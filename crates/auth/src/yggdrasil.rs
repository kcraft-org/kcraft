use kcraft_core::account::{AccountData, AccountTaskState, Validity};
use tracing::info;

use crate::parsers;
use crate::{AuthError, AuthFlow, Result};

pub struct YggdrasilFlow {
    password: Option<String>,
    base_url: String,
}

impl YggdrasilFlow {
    pub fn new_refresh() -> Self {
        YggdrasilFlow {
            password: None,
            base_url: "https://authserver.mojang.com".to_string(),
        }
    }

    pub fn new_login(password: String) -> Self {
        YggdrasilFlow {
            password: Some(password),
            base_url: "https://authserver.mojang.com".to_string(),
        }
    }

    pub fn set_base_url(&mut self, url: String) {
        self.base_url = url;
    }

    fn authenticate(&self, data: &mut AccountData) -> Result<AccountTaskState> {
        let password = self
            .password
            .as_deref()
            .ok_or_else(|| AuthError::Auth("Password required for Yggdrasil login".to_string()))?;

        let username = data
            .yggdrasil_token
            .extra
            .get("userName")
            .cloned()
            .unwrap_or_default();
        let client_token = data
            .yggdrasil_token
            .extra
            .get("clientToken")
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string().replace('-', ""));

        let body = serde_json::json!({
            "agent": {
                "name": "Minecraft",
                "version": 1
            },
            "username": username,
            "password": password,
            "requestUser": false,
            "clientToken": client_token
        });

        let url = format!("{}/authenticate", self.base_url);
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .json(&body)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Yggdrasil authenticate failed: {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().map_err(|e| {
            AuthError::InvalidResponse(format!("Failed to parse Yggdrasil response: {}", e))
        })?;

        let token = parsers::parse_yggdrasil_response(&json, &client_token).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse Yggdrasil token".to_string())
        })?;

        data.yggdrasil_token = token;
        data.validity = Validity::Certain;

        if let Some(profile) = json.get("selectedProfile") {
            if let Some(profile_obj) = profile.as_object() {
                if let Some(name) = profile_obj.get("name").and_then(|v| v.as_str()) {
                    data.minecraft_profile.name = name.to_string();
                }
                if let Some(id) = profile_obj.get("id").and_then(|v| v.as_str()) {
                    data.minecraft_profile.id = id.to_string();
                }
            }
        }

        info!("Yggdrasil login successful for {}", username);
        Ok(AccountTaskState::Succeeded)
    }

    fn refresh(&self, data: &mut AccountData) -> Result<AccountTaskState> {
        let client_token = data.client_token().to_string();
        let access_token = data.access_token().to_string();

        if access_token.is_empty() {
            return Err(AuthError::Auth("No access token to refresh".to_string()));
        }

        let body = serde_json::json!({
            "clientToken": client_token,
            "accessToken": access_token,
            "requestUser": false
        });

        let url = format!("{}/refresh", self.base_url);
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .json(&body)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Yggdrasil refresh failed: {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().map_err(|e| {
            AuthError::InvalidResponse(format!("Failed to parse Yggdrasil response: {}", e))
        })?;

        let ygg_token = data
            .yggdrasil_token
            .extra
            .get("clientToken")
            .cloned()
            .unwrap_or_else(|| client_token.clone());

        let token = parsers::parse_yggdrasil_response(&json, &ygg_token).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse Yggdrasil token".to_string())
        })?;

        data.yggdrasil_token = token;
        data.validity = Validity::Certain;

        if let Some(profile) = json.get("selectedProfile") {
            if let Some(profile_obj) = profile.as_object() {
                if let Some(name) = profile_obj.get("name").and_then(|v| v.as_str()) {
                    data.minecraft_profile.name = name.to_string();
                }
                if let Some(id) = profile_obj.get("id").and_then(|v| v.as_str()) {
                    data.minecraft_profile.id = id.to_string();
                }
            }
        }

        info!("Yggdrasil refresh successful");
        Ok(AccountTaskState::Succeeded)
    }
}

impl AuthFlow for YggdrasilFlow {
    fn name(&self) -> &str {
        if self.password.is_some() {
            "yggdrasil_login"
        } else {
            "yggdrasil_refresh"
        }
    }

    fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        if self.password.is_some() {
            self.authenticate(data)
        } else {
            self.refresh(data)
        }
    }
}

pub fn get_yggdrasil_base_url(authlib_injector_url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AuthError::Network(e.to_string()))?;

    let response = client
        .get(authlib_injector_url)
        .send()
        .map_err(|e| AuthError::Network(e.to_string()))?;

    if let Some(api_location) = response
        .headers()
        .get("x-authlib-injector-api-location")
        .and_then(|v| v.to_str().ok())
    {
        Ok(api_location.trim_end_matches('/').to_string())
    } else {
        Err(AuthError::InvalidResponse(
            "No x-authlib-injector-api-location header in response".to_string(),
        ))
    }
}
