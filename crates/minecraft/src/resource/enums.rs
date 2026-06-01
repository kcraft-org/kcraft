#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Unknown,
    Folder,
    ZipFile,
    Litemod,
    SingleFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableAction {
    Enable,
    Disable,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortType {
    Name,
    Date,
    Enabled,
}
