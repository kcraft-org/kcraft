use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    #[serde(default)]
    pub issue_instant: Option<i64>,
    #[serde(default)]
    pub not_after: Option<i64>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
    #[serde(default)]
    pub validity: Validity,
    #[serde(default = "default_true")]
    pub persistent: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Token {
    fn default() -> Self {
        Token {
            issue_instant: None,
            not_after: None,
            token: None,
            refresh_token: None,
            extra: HashMap::new(),
            validity: Validity::None,
            persistent: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub variant: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cape {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftEntitlement {
    #[serde(default)]
    pub owns_minecraft: bool,
    #[serde(default)]
    pub can_play_minecraft: bool,
    #[serde(default)]
    pub validity: Validity,
}

impl Default for MinecraftEntitlement {
    fn default() -> Self {
        MinecraftEntitlement {
            owns_minecraft: false,
            can_play_minecraft: false,
            validity: Validity::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub skin: Skin,
    #[serde(default)]
    pub current_cape: String,
    #[serde(default)]
    pub capes: Vec<Cape>,
    #[serde(default)]
    pub validity: Validity,
}

impl Default for MinecraftProfile {
    fn default() -> Self {
        MinecraftProfile {
            id: String::new(),
            name: String::new(),
            skin: Skin::default(),
            current_cape: String::new(),
            capes: Vec::new(),
            validity: Validity::None,
        }
    }
}

impl Skin {
    pub fn new(id: String, url: String, variant: String) -> Self {
        Skin {
            id,
            url,
            variant,
            data: None,
        }
    }
}

impl Default for Skin {
    fn default() -> Self {
        Skin {
            id: String::new(),
            url: String::new(),
            variant: "classic".to_string(),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    #[serde(rename = "type")]
    pub account_type: AccountType,
    #[serde(default)]
    pub authlib_injector_base_url: String,
    #[serde(default)]
    pub authlib_injector_api_location: String,
    #[serde(default)]
    pub can_migrate_to_msa: bool,
    #[serde(default)]
    pub msa_client_id: String,
    #[serde(default)]
    pub msa_token: Token,
    #[serde(default)]
    pub user_token: Token,
    #[serde(default)]
    pub xbox_api_token: Token,
    #[serde(default)]
    pub mojangservices_token: Token,
    #[serde(default)]
    pub yggdrasil_token: Token,
    #[serde(default)]
    pub minecraft_profile: MinecraftProfile,
    #[serde(default)]
    pub minecraft_entitlement: MinecraftEntitlement,
    #[serde(default)]
    pub validity: Validity,
    #[serde(skip)]
    pub internal_id: String,
    #[serde(skip)]
    pub error_string: String,
    #[serde(skip)]
    pub account_state: AccountState,
}

impl AccountData {
    pub fn user_name(&self) -> &str {
        self.yggdrasil_token
            .extra
            .get("userName")
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn client_token(&self) -> &str {
        self.yggdrasil_token
            .extra
            .get("clientToken")
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn access_token(&self) -> &str {
        self.yggdrasil_token.token.as_deref().unwrap_or("")
    }

    pub fn profile_id(&self) -> &str {
        &self.minecraft_profile.id
    }

    pub fn profile_name(&self) -> &str {
        &self.minecraft_profile.name
    }

    pub fn last_error(&self) -> &str {
        &self.error_string
    }

    pub fn account_display_string(&self) -> String {
        match self.account_type {
            AccountType::Msa => self
                .xbox_api_token
                .extra
                .get("gtg")
                .cloned()
                .unwrap_or_else(|| self.profile_name().to_string()),
            _ => self.user_name().to_string(),
        }
    }
}

impl Default for AccountData {
    fn default() -> Self {
        AccountData {
            account_type: AccountType::Offline,
            authlib_injector_base_url: String::new(),
            authlib_injector_api_location: String::new(),
            can_migrate_to_msa: false,
            msa_client_id: String::new(),
            msa_token: Token::default(),
            user_token: Token::default(),
            xbox_api_token: Token::default(),
            mojangservices_token: Token::default(),
            yggdrasil_token: Token::default(),
            minecraft_profile: MinecraftProfile::default(),
            minecraft_entitlement: MinecraftEntitlement::default(),
            validity: Validity::None,
            internal_id: String::new(),
            error_string: String::new(),
            account_state: AccountState::Unchecked,
        }
    }
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
