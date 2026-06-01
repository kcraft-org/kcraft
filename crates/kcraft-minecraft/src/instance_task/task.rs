use super::name::InstanceName;

#[derive(Debug, Clone)]
pub struct InstanceTask {
    staging_path: String,
    inst_icon: String,
    inst_group: String,
    instance_name: InstanceName,
    override_existing: bool,
}

impl InstanceTask {
    pub fn new(name: &str, version: &str) -> Self {
        InstanceTask {
            staging_path: String::new(),
            inst_icon: "default".to_string(),
            inst_group: String::new(),
            instance_name: InstanceName::new(name, version),
            override_existing: false,
        }
    }

    pub fn set_staging_path(&mut self, path: &str) {
        self.staging_path = path.to_string();
    }
    pub fn staging_path(&self) -> &str {
        &self.staging_path
    }
    pub fn set_icon(&mut self, icon: &str) {
        self.inst_icon = icon.to_string();
    }
    pub fn icon(&self) -> &str {
        &self.inst_icon
    }
    pub fn set_group(&mut self, group: &str) {
        self.inst_group = group.to_string();
    }
    pub fn group(&self) -> &str {
        &self.inst_group
    }
    pub fn should_override(&self) -> bool {
        self.override_existing
    }
    pub fn set_override(&mut self, override_: bool) {
        self.override_existing = override_;
    }
    pub fn name(&self) -> String {
        self.instance_name.modified_name()
    }
    pub fn set_instance_name(&mut self, name: &str) {
        self.instance_name.set_name(name);
    }

    pub fn execute(&mut self) -> Result<(), String> {
        Err("Not implemented".to_string())
    }
}
