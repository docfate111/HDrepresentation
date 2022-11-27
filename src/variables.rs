pub use crate::types::*;
pub use std::ffi::CString;
pub use std::fmt;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: Box<VariableType>,
    pub kind: FileType,
}

impl Variable {
    pub fn new(name: &str, var_type: VariableType, kind: FileType) -> Self {
        Self {
            name: String::from(name),
            var_type: Box::new(var_type),
            kind,
        }
    }

    pub fn is_pointer(self) -> bool {
        match *self.var_type {
            VariableType::UCharPtr(_, _) | VariableType::Str(_) | VariableType::VoidPtr => true,
            _ => false,
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self.var_type {
            VariableType::Long(value) => write!(f, "\tlong {} = {};\n", self.name, value),
            VariableType::Str(value) => {
                let cstr = format!(
                    "{:?}",
                    CString::new(&**value).expect("invalid c string in fmt::Display Variable")
                );
                let mut var = String::from(&cstr[1..cstr.len() - 1]);
                var.push_str("\\x00");
                write!(f, "\tchar {}[] = \"{}\";\n", self.name, var)
            }
            VariableType::UCharPtr(value, size) => {
                let mut line = format!("\tunsigned char {}[{}];\n", self.name, size);
                match value {
                    None => {
                        line.push_str(&format!("\tmemset({}, 0,", self.name));
                        //print_binstr(value, size); <- was in janus idk if i need it
                        line.push_str(&format!("{});\n", size));
                    }
                    Some(v) => {
                        line.push_str(&format!(
                            "\tmemcpy({},{:?}",
                            self.name,
                            CString::new(&*v.clone())
                                .expect("invalid c string in fmt::Display Veriable UCharPtr")
                        ));
                        line.push_str(&format!(",{});", size));
                    }
                }
                return write!(f, "{}", line);
            }
            _ => Err(fmt::Error),
        }
    }
}
