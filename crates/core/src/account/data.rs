use serde::{Deserialize, Serialize};

use super::profile::{MinecraftEntitlement, MinecraftProfile};
use super::token::Token;
use super::types::{AccountState, AccountType, Validity};

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
