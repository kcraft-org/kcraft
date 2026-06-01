#[derive(Debug, Clone)]
pub struct InstanceName {
    original_name: String,
    original_version: String,
    modified_name: Option<String>,
}

impl InstanceName {
    pub fn new(name: &str, version: &str) -> Self {
        InstanceName {
            original_name: name.to_string(),
            original_version: version.to_string(),
            modified_name: None,
        }
    }

    pub fn modified_name(&self) -> String {
        self.modified_name
            .clone()
            .unwrap_or_else(|| self.original_name.clone())
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn name(&self) -> String {
        match &self.modified_name {
            Some(m) => format!("{} {}", m, self.original_version),
            None => format!("{} {}", self.original_name, self.original_version),
        }
    }

    pub fn version(&self) -> &str {
        &self.original_version
    }

    pub fn set_name(&mut self, name: &str) {
        self.modified_name = Some(name.to_string());
    }
}
