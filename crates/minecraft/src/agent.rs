use crate::Library;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub library: Library,
    pub argument: String,
}

impl Agent {
    pub fn new(library: Library, argument: String) -> Self {
        Agent { library, argument }
    }
}
