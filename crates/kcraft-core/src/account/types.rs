use serde::{Deserialize, Serialize};

pub type InstanceId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Msa,
    AuthlibInjector,
    Offline,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Msa => "msa",
            AccountType::AuthlibInjector => "authlib-injector",
            AccountType::Offline => "offline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AccountState {
    #[default]
    Unchecked,
    Offline,
    Working,
    Online,
    Disabled,
    Errored,
    Expired,
    Queued,
    Gone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Validity {
    #[default]
    None,
    Assumed,
    Certain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Idle,
    LoggingIn,
    LoggingOut,
    Refreshing,
    FailedSoft,
    FailedHard,
    FailedGone,
    Succeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountTaskState {
    Created,
    Working,
    Succeeded,
    Disabled,
    FailedSoft,
    FailedHard,
    FailedGone,
    Offline,
}
