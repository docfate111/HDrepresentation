pub use crate::types::*;

#[derive(Copy, Eq, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Arg {
    pub value: Option<i64>,
    pub index: Option<usize>,
    pub is_variable: bool,
}

impl Arg {
    pub fn new(x: i64, is_variable: bool) -> Self {
        if is_variable {
            Self {
                value: None,
                index: Some(x as usize),
                is_variable: true,
            }
        } else {
            Self {
                value: Some(x),
                index: None,
                is_variable: false,
            }
        }
    }
}
