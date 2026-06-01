#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchState {
    NotStarted,
    Running,
    Waiting,
    Failed,
    Aborted,
    Finished,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    StdOut,
    StdErr,
    Warning,
    Error,
    Fatal,
    Launcher,
    Minecraft,
    Unknown,
}
