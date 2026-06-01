use super::settings_object::INISettingsObject;

pub struct SettingsLock {
    obj: INISettingsObject,
}

impl SettingsLock {
    pub fn new(obj: &INISettingsObject) -> Self {
        obj.suspend_save();
        SettingsLock { obj: obj.clone() }
    }
}

impl Drop for SettingsLock {
    fn drop(&mut self) {
        self.obj.resume_save();
    }
}
