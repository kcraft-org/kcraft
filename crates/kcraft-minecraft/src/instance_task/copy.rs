use std::path::Path;

use super::import::copy_dir;
use super::task::InstanceTask;
use crate::instance::Instance;

#[derive(Debug, Clone)]
pub struct InstanceCopyTask {
    base: InstanceTask,
    instance_root: String,
    matcher: Option<String>,
    keep_playtime: bool,
}

impl InstanceCopyTask {
    pub fn new(instance_root: &str, copy_saves: bool, keep_playtime: bool) -> Self {
        let matcher = if !copy_saves {
            Some("[.]?minecraft/saves".to_string())
        } else {
            None
        };
        InstanceCopyTask {
            base: InstanceTask::new("", ""),
            instance_root: instance_root.to_string(),
            matcher,
            keep_playtime,
        }
    }

    pub fn execute(&mut self) -> Result<(), String> {
        let src = Path::new(&self.instance_root);
        let dst = Path::new(self.base.staging_path());

        copy_dir(src, dst, self.matcher.as_deref())?;

        let mut instance = Instance::new(dst.to_string_lossy().as_ref(), "");
        instance.load_specific_settings();

        self.base.set_instance_name(&instance.name);
        self.base.set_icon(&instance.icon_key);

        if !self.keep_playtime {
            instance.total_time_played = 0;
            instance.last_launch_time = 0;
        }

        instance.save_now();
        Ok(())
    }
}
