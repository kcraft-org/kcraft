use crate::{AuthFlow, Result};
use app_core::account::{
    AccountData, AccountState, AccountTaskState, AccountType, MinecraftEntitlement,
    MinecraftProfile, Skin, Token, Validity,
};

pub struct OfflineFlow {
    username: String,
}

impl OfflineFlow {
    pub fn new(username: String) -> Self {
        OfflineFlow { username }
    }
}

impl AuthFlow for OfflineFlow {
    fn name(&self) -> &str {
        "offline"
    }

    fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        data.account_type = AccountType::Offline;
        data.yggdrasil_token = Token {
            token: Some("offline".to_string()),
            validity: Validity::Certain,
            issue_instant: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0),
            ),
            extra: {
                let mut m = std::collections::HashMap::new();
                m.insert("userName".to_string(), self.username.clone());
                m.insert(
                    "clientToken".to_string(),
                    uuid::Uuid::new_v4().to_string().replace('-', ""),
                );
                m
            },
            ..Default::default()
        };

        data.minecraft_entitlement = MinecraftEntitlement {
            owns_minecraft: true,
            can_play_minecraft: true,
            validity: Validity::Certain,
        };

        let uuid = crate::generate_offline_uuid(&self.username);
        data.minecraft_profile = MinecraftProfile {
            id: uuid,
            name: self.username.clone(),
            skin: Skin {
                id: String::new(),
                url: String::new(),
                variant: "classic".to_string(),
                data: None,
            },
            current_cape: String::new(),
            capes: Vec::new(),
            validity: Validity::Certain,
        };

        data.validity = Validity::Certain;
        data.account_state = AccountState::Online;

        Ok(AccountTaskState::Succeeded)
    }
}
