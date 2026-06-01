#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Inactive,
    Running,
    Succeeded,
    Failed,
    AbortedByUser,
}
