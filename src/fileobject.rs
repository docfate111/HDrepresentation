pub use crate::types::*;
pub use std::collections::HashMap;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Debug, Eq, Clone, Hash, PartialEq)]
pub struct Xattr(pub String, pub String, pub i64);

/*#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(into = "Vec<(K, V)>")]
#[serde(from = "Vec<(K, V)>")]
pub struct ObjectMap<K: Eq + Hash + Clone, V: Clone>(pub HashMap<K, V>);

impl<K: Eq + Hash + Clone, V: Clone> From<ObjectMap<K, V>> for Vec<(K, V)> {
        fn from(map: ObjectMap<K, V>) -> Self {
                    map.0.into_iter().collect()
                            }
}

impl<K: Eq + Hash + Clone, V: Clone> From<Vec<(K, V)>> for ObjectMap<K, V>{
        fn from(vec: Vec<(K, V)>) -> Self {
                    Self(vec.into_iter().collect())
                            }
}*/

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub struct FileObject {
    pub rel_path: String,
    pub ftype: FileType,
    pub xattrs: Vec<Xattr>,
    pub fd_index: i64,
}

impl FileObject {
    pub fn new(path: &str, ftype: FileType, fd_index: i64) -> Self {
        Self {
            rel_path: String::from(path),
            ftype,
            xattrs: Vec::<Xattr>::new(),
            fd_index,
        }
    }
}

impl fmt::Display for FileObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output = format!("Path {}\n Type: ", self.rel_path);
        let ftype = match self.ftype {
            FileType::Symlink => String::from("symlink"),
            FileType::File => String::from("file"),
            FileType::Dir => String::from("dir"),
            FileType::Fifo => String::from("fifo"),
            _ => String::from("other"),
        };
        output.push_str(&ftype);
        output.push_str("\nXattrs:\n");
        for xattr in self.xattrs.iter() {
            output.push_str(&format!("\t{}:{}\n", xattr.0, xattr.1));
        }
        write!(f, "{}\n", output)
    }
}
