use kcraft_core::account::{AccountData, AccountState, AccountTaskState, AccountType};
use tracing::info;

use crate::parsers;
use crate::yggdrasil::{get_yggdrasil_base_url, YggdrasilFlow};
use crate::{AuthError, AuthFlow, Result};

pub struct AuthlibInjectorFlow {
    username: String,
    password: Option<String>,
    base_url: String,
}

impl AuthlibInjectorFlow {
    pub fn new_refresh(username: String, base_url: String) -> Self {
        AuthlibInjectorFlow {
            username,
            password: None,
            base_url,
        }
    }

    pub fn new_login(username: String, password: String, base_url: String) -> Self {
        AuthlibInjectorFlow {
            username,
            password: Some(password),
            base_url,
        }
    }
}

impl AuthFlow for AuthlibInjectorFlow {
    fn name(&self) -> &str {
        if self.password.is_some() {
            "authlib_injector_login"
        } else {
            "authlib_injector_refresh"
        }
    }

    fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        data.account_type = AccountType::AuthlibInjector;
        data.authlib_injector_base_url = self.base_url.clone();

        let api_location = get_yggdrasil_base_url(&self.base_url)?;
        data.authlib_injector_api_location = api_location.clone();

        let base = api_location.trim_end_matches("/authserver").to_string();
        let session_server = format!("{}/sessionserver", base);

        let mut ygg_flow = if let Some(ref password) = self.password {
            let mut flow = YggdrasilFlow::new_login(password.clone());
            flow.set_base_url(api_location.clone());
            data.yggdrasil_token
                .extra
                .insert("userName".to_string(), self.username.clone());
            flow
        } else {
            let mut flow = YggdrasilFlow::new_refresh();
            flow.set_base_url(api_location.clone());
            flow
        };

        let result = ygg_flow.execute(data)?;

        if result != AccountTaskState::Succeeded {
            return Ok(result);
        }

        let profile_id = data.profile_id().to_string();
        if !profile_id.is_empty() {
            let url = format!(
                "{}/session/minecraft/profile/{}",
                session_server, profile_id
            );
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&url)
                .send()
                .map_err(|e| AuthError::Network(e.to_string()))?;

            if response.status().is_success() {
                let bytes = response
                    .bytes()
                    .map_err(|e| AuthError::Network(e.to_string()))?;
                if let Some(profile) = parsers::parse_minecraft_profile_mojang(&bytes) {
                    data.minecraft_profile = profile;
                }
            }
        }

        info!(
            "Authlib-Injector auth successful for {} at {}",
            self.username, self.base_url
        );
        data.account_state = AccountState::Online;
        Ok(AccountTaskState::Succeeded)
    }
}
