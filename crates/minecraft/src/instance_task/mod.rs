pub mod copy;
pub mod creation;
pub mod import;
pub mod name;
pub mod task;

pub use copy::InstanceCopyTask;
pub use creation::{InstanceCreationTask, VanillaInstanceCreationTask};
pub use import::InstanceImportTask;
pub use name::InstanceName;
pub use task::InstanceTask;
