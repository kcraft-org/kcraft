use serde::{Deserialize, Serialize};
use crate::Library;

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
