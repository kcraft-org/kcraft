use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::instance::Instance;

pub type InstanceId = String;
pub type GroupId = String;
pub type InstanceLocator = (InstancePtr, usize);

#[derive(Debug, Clone)]
pub struct InstancePtr {
    inner: Arc<RwLock<Instance>>,
}

impl InstancePtr {
    pub fn new(instance: Instance) -> Self {
        InstancePtr {
            inner: Arc::new(RwLock::new(instance)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Instance> {
        self.inner.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Instance> {
        self.inner.write().unwrap()
    }
}
