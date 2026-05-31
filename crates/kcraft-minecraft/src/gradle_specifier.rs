use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GradleSpecifier {
    group_id: String,
    artifact_id: String,
    version: String,
    classifier: String,
    extension: String,
    valid: bool,
    invalid_value: String,
}

impl GradleSpecifier {
    pub fn parse(value: &str) -> Self {
        let pattern = regex::Regex::new(
            r"([^:@]+):([^:@]+):([^:@]+)(?::([^:@]+))?(?:@([^:@]+))?"
        ).unwrap();

        if let Some(caps) = pattern.captures(value) {
            let group_id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let artifact_id = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let version = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
            let classifier = caps.get(4).map(|m| m.as_str().to_string()).unwrap_or_default();
            let extension = caps.get(5).map(|m| m.as_str().to_string()).unwrap_or_else(|| "jar".to_string());

            GradleSpecifier {
                group_id,
                artifact_id,
                version,
                classifier,
                extension,
                valid: true,
                invalid_value: String::new(),
            }
        } else {
            GradleSpecifier {
                group_id: String::new(),
                artifact_id: String::new(),
                version: String::new(),
                classifier: String::new(),
                extension: "jar".to_string(),
                valid: false,
                invalid_value: value.to_string(),
            }
        }
    }

    pub fn serialize(&self) -> String {
        if !self.valid {
            return self.invalid_value.clone();
        }
        let mut ret = format!("{}:{}:{}", self.group_id, self.artifact_id, self.version);
        if !self.classifier.is_empty() {
            ret.push(':');
            ret.push_str(&self.classifier);
        }
        if self.extension != "jar" {
            ret.push('@');
            ret.push_str(&self.extension);
        }
        ret
    }

    pub fn get_file_name(&self) -> String {
        if !self.valid {
            return String::new();
        }
        let mut filename = format!("{}-{}", self.artifact_id, self.version);
        if !self.classifier.is_empty() {
            filename.push('-');
            filename.push_str(&self.classifier);
        }
        filename.push('.');
        filename.push_str(&self.extension);
        filename
    }

    pub fn to_path(&self, filename_override: Option<&str>) -> String {
        if !self.valid {
            return String::new();
        }
        let filename = filename_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.get_file_name());
        let path = self.group_id.replace('.', "/");
        format!("{}/{}/{}/{}", path, self.artifact_id, self.version, filename)
    }

    pub fn valid(&self) -> bool { self.valid }
    pub fn version(&self) -> &str { &self.version }
    pub fn group_id(&self) -> &str { &self.group_id }
    pub fn artifact_id(&self) -> &str { &self.artifact_id }
    pub fn classifier(&self) -> &str { &self.classifier }
    pub fn extension(&self) -> &str { &self.extension }
    pub fn artifact_prefix(&self) -> String {
        format!("{}:{}", self.group_id, self.artifact_id)
    }

    pub fn set_classifier(&mut self, classifier: String) {
        self.classifier = classifier;
    }

    pub fn match_name(&self, other: &GradleSpecifier) -> bool {
        self.artifact_id == other.artifact_id
            && self.group_id == other.group_id
            && self.classifier == other.classifier
    }
}

impl From<String> for GradleSpecifier {
    fn from(s: String) -> Self {
        GradleSpecifier::parse(&s)
    }
}

impl From<&str> for GradleSpecifier {
    fn from(s: &str) -> Self {
        GradleSpecifier::parse(s)
    }
}

impl std::fmt::Display for GradleSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize())
    }
}
