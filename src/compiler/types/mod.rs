use std::fmt::Display;

pub mod mark;
pub mod reagents;
pub mod strand;
pub mod tapestry;
pub mod weaves;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Secret,
}

impl Visibility {
    pub fn default() -> Self {
        Visibility::Public
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Public => write!(f, "Public"),
            Visibility::Secret => write!(f, "Secret"),
        }
    }
}
