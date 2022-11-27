pub use crate::arg::*;
mod arg;
pub use crate::fileobject::*;
mod fileobject;
pub use crate::progconstants::*;
mod progconstants;
pub use crate::syscall::*;
mod syscall;
pub use crate::types::*;
mod types;
pub use crate::variables::*;
mod variables;
use serde_with::serde_as;
use std::fmt::Write;
use std::fs::{read_to_string, write};
use std::path::Path;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    // To make Program more extensible, we need a vector of indices.
    pub variables: Vec<Variable>,
    pub syscalls: Vec<Syscall>,
    // the list of various variable indices.
    //variable_indices: Vec<Vec<i64>>
    // these are indexes for variables vector:
    pub active_fds: Vec<i64>,
    pub active_file_fds: Vec<i64>,
    pub active_dir_fds: Vec<i64>,
    // bases of mmap'd memory
    pub active_map_base_idx: Vec<i64>,
    // active_map_size: Vec<i64>; // should match with above
    pub avail_files: Vec<FileObject>,
    pub avail_dirs: Vec<FileObject>,
    pub avail_non_dirs: Vec<FileObject>,
    // store variables for path of all file objects
    //#[serde(with = "tuple_vec_map")]
}

impl Program {
    pub const PAGE_SIZE: u32 = 4096;
    pub const SRC8192: i64 = 0;
    pub const DEST8192: i64 = 1;
    // v_2 .. v_n are variables for path
    pub const PATHSTART: i64 = 2;
    pub fn new() -> Self {
        Self {
            variables: Vec::<Variable>::new(),
            syscalls: Vec::<Syscall>::new(),
            active_fds: Vec::<i64>::new(),
            active_file_fds: Vec::<i64>::new(),
            active_dir_fds: Vec::<i64>::new(),
            active_map_base_idx: Vec::<i64>::new(),
            avail_files: Vec::<FileObject>::new(),
            avail_dirs: Vec::<FileObject>::new(),
            avail_non_dirs: Vec::<FileObject>::new(),
        }
    }

    // create large buffers to store data
    pub fn prepare_buffers(&mut self) {
        assert!(
            self.create_variable(VariableType::UCharPtr(None, Program::PAGE_SIZE * 2))
                == Program::SRC8192
        );
        assert!(
            self.create_variable(VariableType::UCharPtr(None, Program::PAGE_SIZE * 2))
                == Program::DEST8192
        );
    }

    pub fn create_str(&mut self, s: &str) -> i64 {
        self.create_variable(VariableType::Str(String::from(s).clone()))
    }

    // create variable of non-file type
    pub fn create_variable(&mut self, var_type: VariableType) -> i64 {
        self.create_file_variable(var_type, FileType::Unknown)
    }

    // create variable of file type(i.e. file, dir, symlink for file descriptors, mmap address)
    pub fn create_file_variable(&mut self, var_type: VariableType, kind: FileType) -> i64 {
        let var_count = self.variables.len();
        let mut name = String::from("v");
        name.push_str(&var_count.to_string());
        let v = Variable::new(&name, var_type, kind);
        self.add_variable(v);
        var_count as i64
    }
    // add file descriptor
    pub fn add_fd(&mut self, fd_index: i64) {
        self.active_fds.push(fd_index);
        // get the variable using the index and if its a directory put it with directory fds
        let var = self.variables.get(fd_index as usize).unwrap();
        match var.kind {
            FileType::Dir => {
                self.active_dir_fds.push(fd_index);
            }
            FileType::File | FileType::Symlink | FileType::Fifo => {
                self.active_file_fds.push(fd_index);
            }
            _ => {}
        }
    }

    pub fn add_variable(&mut self, v: Variable) {
        self.variables.push(v.clone());
        // if the variable is a file descriptor and its index for variables into active fds
        match v.kind {
            FileType::File | FileType::Dir | FileType::Symlink | FileType::Fifo => {
                self.add_fd((self.variables.len() - 1) as i64);
            }
            _ => {}
        }
    }

    pub fn remove_last_variable(&mut self) {
        if self.variables.len() < 1 {
            eprintln!("remove_last_variable: no more variables");
            return;
        }
        let last = self.variables.pop().unwrap();
        let last_index = (self.variables.len() - 1) as i64;
        match last.kind {
            FileType::File | FileType::Dir | FileType::Symlink | FileType::Fifo => {
                self.remove_fd(last_index);
            }
            FileType::Mmap => {
                self.mark_base_unmapped(last_index);
            }
            _ => {}
        }
    }

    pub fn add_syscall(&mut self, v: Syscall) {
        self.syscalls.push(v);
    }

    pub fn remove_last_syscall(&mut self) {
        if self.syscalls.len() < 1 {
            eprintln!("remove_last_syscall: no more syscalls");
            return;
        }
        self.syscalls.pop();
    }

    pub fn remove_last_syscall_if_same(&mut self, syscall: Syscall) {
        if self.syscalls.pop().expect("remove_last_syscall_if_same") == syscall {
            self.remove_last_syscall();
        }
    }

    pub fn remove_syscall(&mut self, syscall: Syscall) {
        self.syscalls.retain(|x| *x != syscall);
    }

    // add file object
    pub fn add_file(&mut self, fobj: FileObject, var_index: i64) {
        let mut f = fobj.clone();
        f.fd_index = var_index;
        match f.ftype {
            FileType::Dir => {
                self.avail_dirs.push(fobj);
            }
            FileType::File | FileType::Fifo | FileType::Symlink => {
                self.avail_non_dirs.push(fobj);
            }
            _ => {
                eprintln!("add_file: invalid file object added to avail_files");
            }
        }
        self.avail_files.push(f.clone());
    }

    pub fn remove_last_file(&mut self) {
        let fobj = self
            .avail_files
            .pop()
            .expect("remove_last_file: avail_files has no more to remove");
        match fobj.ftype {
            FileType::Dir => {
                self.avail_dirs
                    .pop()
                    .expect("remove_last_file: avail_dirs is empty");
            }
            FileType::File | FileType::Fifo | FileType::Symlink => {
                self.avail_non_dirs
                    .pop()
                    .expect("remove_last_file: avail_non_dirs is empty");
            }
            _ => {
                eprintln!("remove_last_file: invalid file object added to remove_last_file");
            }
        }
    }
    pub fn remove_file(&mut self, fobj: FileObject) {
        self.avail_files.retain(|x| *x != fobj);
        match fobj.ftype {
            FileType::Dir => {
                self.remove_dir(fobj.clone());
            }
            FileType::File | FileType::Fifo | FileType::Symlink => {
                self.remove_non_dir(fobj.clone());
            }
            _ => {
                eprintln!("remove_file: invalid file object added to remove_last_file");
            }
        }
    }

    pub fn remove_dir(&mut self, fobj: FileObject) {
        self.avail_dirs.retain(|x| *x != fobj);
    }

    pub fn remove_non_dir(&mut self, fobj: FileObject) {
        self.avail_non_dirs.retain(|x| *x != fobj);
    }

    pub fn remove_fd(&mut self, fd_index: i64) {
        let var = self
            .variables
            .get(fd_index as usize)
            .expect("remove_fd: invalid index");
        match var.kind {
            FileType::Dir => {
                self.remove_dir_fd(fd_index);
            }
            FileType::File | FileType::Fifo | FileType::Symlink => {
                self.remove_file_fd(fd_index);
            }
            _ => {
                eprintln!("Invalid type for fd");
            }
        }
        self.active_fds.retain(|&x| x != fd_index);
    }

    pub fn remove_file_fd(&mut self, fd_index: i64) {
        self.active_file_fds.retain(|&x| x != fd_index);
    }

    pub fn remove_dir_fd(&mut self, fd_index: i64) {
        self.active_dir_fds.retain(|&x| x != fd_index);
    }

    pub fn mark_base_unmapped(&mut self, map_index: i64) {
        let index = self
            .active_map_base_idx
            .iter()
            .position(|x| *x == map_index)
            .expect("mark_base_unmapped: index does not exist");
        if let Some(var) = self.variables.get_mut(index) {
            var.kind = FileType::None;
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let file = read_to_string(path).unwrap();
        serde_json::from_str(&file).unwrap()
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let serialized_str = serde_json::to_string(&self)?;
        write(path, serialized_str)?;
        Ok(())
    }

    pub fn cprogram_to_file<P: AsRef<Path>>(&self, path: &mut P) -> std::io::Result<()> {
        write(path, format!("{}", self))
    }
}

pub fn get_headers() -> String {
    String::from(
        "#define _GNU_SOURCE\n\
#include <sys/types.h>\n\
#include <sys/mount.h>\n\
#include <sys/mman.h>\n\
#include <sys/stat.h>\n\
#include <sys/xattr.h>\n\
#include <sys/syscall.h>\n\n\
#include <dirent.h>\n\
#include <errno.h>\n\
//#include <error.h>\n\
#include <fcntl.h>\n\
#include <stdio.h>\n\
#include <stdlib.h>\n\
#include <string.h>\n\
#include <unistd.h>\n",
    )
}

pub fn get_syscall(prog: &Program, syscall: &Syscall) -> String {
    let mut output = String::from("");
    // why -1
    let sys_name = num_to_name(syscall.nr);
    if syscall.ret_index != -1 {
        let index = syscall.ret_index as usize;
        let name = &prog.variables[index].name;
        write!(&mut output, "\t{name} = syscall({sys_name}").unwrap();
    } else {
        write!(&mut output, "\tsyscall({sys_name}").unwrap();
    }
    for arg in syscall.args.iter() {
        if arg.is_variable {
            let index = arg
                .index
                .expect("get_syscall found a value when expecting an index");
            let name = &prog.variables[index].name;
            write!(&mut output, ", (long){name}").unwrap();
        } else {
            let value = arg
                .value
                .expect("get_syscall found an index when expecting a value");
            write!(&mut output, ", {value}").unwrap();
        }
    }
    output.push_str(");\n");
    output
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = get_headers();
        output.push_str("\nint main(int argc, char* argv[])\n{\n");
        for v in self.variables.iter() {
            write!(&mut output, "{}\n", *v).unwrap();
        }
        for s in self.syscalls.iter() {
            let syscall_print_out = get_syscall(self, s);
            write!(&mut output, "{} ", syscall_print_out).unwrap();
        }
        output.push_str("\n");
        for fd_index in self.active_fds.iter() {
            let var = self.variables.get(*fd_index as usize).unwrap();
            write!(&mut output, "\tclose({});\n", var.name).unwrap();
        }
        output.push_str("\treturn 0;\n");
        output.push_str("}\n");
        output.push_str("/* Active fds: ");
        for fd_index in self.active_fds.iter() {
            let var = self.variables.get(*fd_index as usize).unwrap();
            write!(&mut output, "{} ", var.name).unwrap();
        }
        output.push_str("*/\n/*Files\n");
        for fobj in self.avail_files.iter() {
            let index = fobj.fd_index;
            let var = match &*self
                .variables
                .get(index as usize)
                .expect("no such index in variable vector fmt::Display avail_files.iter()")
                .var_type
            {
                VariableType::Str(s) => s.clone(),
                _ => String::from(
                    "wrong index provided to variable vector fmt::Display avail_files.iter()",
                ),
            };
            write!(
                &mut output,
                "{:?}\n",
                CString::new(var.clone()).expect("converting file name for C printing failed")
            )
            .unwrap();
        }
        write!(f, "{}*/\n", output)
    }
}
