use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::component::PackProfile;
use crate::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    Launch,
    Resolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    Offline,
    Online,
}

pub struct ComponentUpdate {
    mode: UpdateMode,
    abort_flag: Arc<AtomicBool>,
}

impl ComponentUpdate {
    pub fn new(mode: UpdateMode, _netmode: NetMode) -> Self {
        ComponentUpdate {
            mode,
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn execute(&self, components: &mut PackProfile) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for index in 0..components.components().len() {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err(vec!["Update aborted".to_string()]);
            }

            let component = &components.components()[index];
            if component.disabled {
                continue;
            }

            if !component.loaded {
                let result = self.load_component(component);
                if let Err(e) = result {
                    errors.push(format!(
                        "Failed to load component '{}': {}",
                        component.uid, e
                    ));
                    if self.mode == UpdateMode::Launch {
                        return Err(errors);
                    }
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn load_component(&self, _component: &Component) -> Result<(), String> {
        Ok(())
    }

    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
    }

    pub fn can_abort(&self) -> bool {
        true
    }
}
