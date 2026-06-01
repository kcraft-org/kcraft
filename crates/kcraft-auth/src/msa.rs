use kcraft_core::account::{AccountData, AccountState, AccountTaskState, AccountType, Validity};
use tracing::{debug, info};
use url::Url;

use crate::parsers;
use crate::AuthError;
use crate::{AuthFlow, Result};

pub type VerificationCallback = Box<dyn Fn(&str, &str, i32) + Send>;

pub struct MsaFlow {
    interactive: bool,
    client_id: String,
    verification_callback: Option<VerificationCallback>,
}

impl MsaFlow {
    pub fn new_silent() -> Self {
        MsaFlow {
            interactive: false,
            client_id: String::new(),
            verification_callback: None,
        }
    }

    pub fn new_interactive(client_id: String) -> Self {
        MsaFlow {
            interactive: true,
            client_id,
            verification_callback: None,
        }
    }

    pub fn set_verification_callback<F: Fn(&str, &str, i32) + Send + 'static>(&mut self, cb: F) {
        self.verification_callback = Some(Box::new(cb));
    }

    fn build_microsoft_steps(&mut self, _data: &mut AccountData) -> Vec<Box<dyn crate::AuthStep>> {
        let cb = self.verification_callback.take();
        vec![
            Box::new(MsaTokenStep {
                client_id: self.client_id.clone(),
                interactive: self.interactive,
                verification_callback: cb,
            }),
            Box::new(XboxUserStep),
            Box::new(XboxAuthorizationStep {
                relying_party: "http://xboxlive.com".to_string(),
                target: "xboxApi".to_string(),
            }),
            Box::new(XboxAuthorizationStep {
                relying_party: "rp://api.minecraftservices.com/".to_string(),
                target: "mojangservices".to_string(),
            }),
            Box::new(LauncherLoginStep),
            Box::new(XboxProfileStep),
            Box::new(EntitlementsStep),
            Box::new(MinecraftProfileStep),
        ]
    }
}

impl AuthFlow for MsaFlow {
    fn name(&self) -> &str {
        if self.interactive {
            "msa_interactive"
        } else {
            "msa_silent"
        }
    }

    fn execute(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        data.account_type = AccountType::Msa;

        let steps = self.build_microsoft_steps(data);
        for mut step in steps {
            let result = step.perform(data)?;
            match result {
                AccountTaskState::Working | AccountTaskState::Created => continue,
                AccountTaskState::Succeeded => {
                    data.validity = Validity::Certain;
                    data.account_state = AccountState::Online;
                    return Ok(AccountTaskState::Succeeded);
                }
                other => {
                    data.account_state = AccountState::Errored;
                    return Ok(other);
                }
            }
        }

        data.validity = Validity::Certain;
        data.account_state = AccountState::Online;
        Ok(AccountTaskState::Succeeded)
    }
}

struct MsaTokenStep {
    client_id: String,
    interactive: bool,
    verification_callback: Option<VerificationCallback>,
}

impl crate::AuthStep for MsaTokenStep {
    fn describe(&self) -> &str {
        if self.interactive {
            "MSA Login"
        } else {
            "MSA Refresh"
        }
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        if self.interactive {
            info!("Starting interactive MSA login");
            if !self.client_id.is_empty() {
                data.msa_client_id = self.client_id.clone();
            }
            let client_id = if data.msa_client_id.is_empty() {
                kcraft_core::build_config::BUILD_CONFIG
                    .msa_client_id
                    .clone()
            } else {
                data.msa_client_id.clone()
            };

            if client_id.is_empty() {
                return Err(AuthError::Auth("MSA client ID not configured".to_string()));
            }

            let client = reqwest::blocking::Client::new();

            let device_code_body = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("client_id", &client_id)
                .append_pair("scope", "XboxLive.signin offline_access")
                .finish();

            let resp = client
                .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(device_code_body)
                .send()
                .map_err(|e| AuthError::Network(e.to_string()))?;

            let resp_json: serde_json::Value = resp
                .json()
                .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

            let user_code = resp_json["user_code"].as_str().unwrap_or("");
            let device_code = resp_json["device_code"].as_str().unwrap_or("");
            let verification_uri = resp_json["verification_uri"].as_str().unwrap_or("");
            let expires_in = resp_json["expires_in"].as_i64().unwrap_or(900) as i32;

            if let Some(ref cb) = self.verification_callback {
                cb(verification_uri, user_code, expires_in);
            } else {
                info!(
                    "Please open {} and enter code: {}",
                    verification_uri, user_code
                );
            }

            let start = std::time::Instant::now();
            loop {
                if start.elapsed().as_secs() > expires_in as u64 {
                    return Err(AuthError::Auth("Device code expired".to_string()));
                }
                std::thread::sleep(std::time::Duration::from_secs(5));

                let token_body = url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
                    .append_pair("client_id", &client_id)
                    .append_pair("device_code", device_code)
                    .finish();

                let token_resp = client
                    .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(token_body)
                    .send()
                    .map_err(|e| AuthError::Network(e.to_string()))?;

                if token_resp.status().is_success() {
                    let token_json: serde_json::Value = token_resp
                        .json()
                        .map_err(|e| AuthError::InvalidResponse(e.to_string()))?;
                    let access_token = token_json["access_token"].as_str().unwrap_or("");
                    let refresh_token = token_json["refresh_token"].as_str().unwrap_or("");

                    data.msa_token = kcraft_core::account::Token {
                        token: Some(access_token.to_string()),
                        refresh_token: Some(refresh_token.to_string()),
                        validity: Validity::Certain,
                        ..Default::default()
                    };
                    break;
                } else {
                    let err_json: serde_json::Value =
                        token_resp.json().unwrap_or(serde_json::Value::Null);
                    let err_str = err_json["error"].as_str().unwrap_or("");
                    if err_str != "authorization_pending" {
                        return Err(AuthError::Auth(format!(
                            "Token polling failed: {}",
                            err_str
                        )));
                    }
                }
            }
        } else {
            info!("Starting silent MSA refresh");
            if data.msa_token.token.is_none() || data.msa_token.refresh_token.is_none() {
                return Err(AuthError::TokenExpired);
            }
            // Implement silent refresh here if needed
            debug!("Refresh token exists, would refresh MSA token");
        }

        Ok(AccountTaskState::Working)
    }
}

struct XboxUserStep;

impl crate::AuthStep for XboxUserStep {
    fn describe(&self) -> &str {
        "Xbox User Authentication"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let msa_token = data.msa_token.token.as_deref().ok_or_else(|| {
            AuthError::Auth("No MSA token available for Xbox user auth".to_string())
        })?;

        let body = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", msa_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .json(&body)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Xbox user auth failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AuthError::Network(e.to_string()))?;
        let token = parsers::parse_x_token_response(&bytes).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse Xbox user token response".to_string())
        })?;

        data.user_token = token;

        let uhs = data
            .user_token
            .extra
            .get("uhs")
            .cloned()
            .unwrap_or_default();
        debug!("Xbox user auth successful. uhs={}", uhs);

        Ok(AccountTaskState::Working)
    }
}

struct XboxAuthorizationStep {
    relying_party: String,
    target: String,
}

impl crate::AuthStep for XboxAuthorizationStep {
    fn describe(&self) -> &str {
        "Xbox Authorization"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let user_token =
            data.user_token.token.as_deref().ok_or_else(|| {
                AuthError::Auth("No user token for Xbox authorization".to_string())
            })?;

        let body = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [user_token]
            },
            "RelyingParty": self.relying_party,
            "TokenType": "JWT"
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://xsts.auth.xboxlive.com/xsts/authorize")
            .json(&body)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "XSTS auth failed for {}: {}",
                self.relying_party,
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AuthError::Network(e.to_string()))?;
        let token = parsers::parse_x_token_response(&bytes).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse XSTS token response".to_string())
        })?;

        match self.target.as_str() {
            "xboxApi" => data.xbox_api_token = token,
            "mojangservices" => data.mojangservices_token = token,
            _ => return Err(AuthError::Auth(format!("Unknown target: {}", self.target))),
        }

        Ok(AccountTaskState::Working)
    }
}

struct LauncherLoginStep;

impl crate::AuthStep for LauncherLoginStep {
    fn describe(&self) -> &str {
        "Minecraft Launcher Login"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let uhs = data
            .user_token
            .extra
            .get("uhs")
            .cloned()
            .unwrap_or_default();
        let mc_token = data
            .mojangservices_token
            .token
            .as_deref()
            .ok_or_else(|| AuthError::Auth("No Mojang services token".to_string()))?;

        let xtoken = format!("XBL3.0 x={};{}", uhs, mc_token);

        let body = serde_json::json!({
            "xtoken": xtoken,
            "platform": "PC_LAUNCHER"
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.minecraftservices.com/launcher/login")
            .json(&body)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Launcher login failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AuthError::Network(e.to_string()))?;
        let token = parsers::parse_mojang_response(&bytes).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse launcher login response".to_string())
        })?;

        data.yggdrasil_token = token;

        let username = data
            .yggdrasil_token
            .extra
            .get("userName")
            .cloned()
            .unwrap_or_default();
        debug!("Launcher login successful. username={}", username);

        Ok(AccountTaskState::Working)
    }
}

struct XboxProfileStep;

impl crate::AuthStep for XboxProfileStep {
    fn describe(&self) -> &str {
        "Xbox Profile"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let uhs = data
            .user_token
            .extra
            .get("uhs")
            .cloned()
            .unwrap_or_default();
        let xbox_token = data
            .xbox_api_token
            .token
            .as_deref()
            .ok_or_else(|| AuthError::Auth("No Xbox API token".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let mut profile_url = Url::parse("https://profile.xboxlive.com/users/me/profile/settings")
            .map_err(|e| AuthError::Network(e.to_string()))?;
        profile_url.query_pairs_mut().append_pair(
            "settings",
            "GameDisplayName,PublicGamerpic,Gamerscore,Gamertag",
        );
        let response = client
            .get(profile_url)
            .header("Authorization", format!("XBL3.0 x={};{}", uhs, xbox_token))
            .header("x-xbl-contract-version", "3")
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Xbox profile fetch failed: {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response.json().map_err(|e| {
            AuthError::InvalidResponse(format!("Failed to parse Xbox profile: {}", e))
        })?;

        if let Some(settings) = json.get("profileUsers").and_then(|v| v.as_array()) {
            if let Some(first) = settings.first().and_then(|v| v.as_object()) {
                if let Some(settings_arr) = first.get("settings").and_then(|v| v.as_array()) {
                    for setting in settings_arr {
                        if let Some(s_obj) = setting.as_object() {
                            if s_obj.get("id").and_then(|v| v.as_str()) == Some("Gamertag") {
                                if let Some(gtg) = s_obj.get("value").and_then(|v| v.as_str()) {
                                    data.xbox_api_token
                                        .extra
                                        .insert("gtg".to_string(), gtg.to_string());
                                    debug!("Xbox gamertag: {}", gtg);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(AccountTaskState::Working)
    }
}

struct EntitlementsStep;

impl crate::AuthStep for EntitlementsStep {
    fn describe(&self) -> &str {
        "Minecraft Entitlements"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let ygg_token =
            data.yggdrasil_token.token.as_deref().ok_or_else(|| {
                AuthError::Auth("No Yggdrasil token for entitlements".to_string())
            })?;

        let client = reqwest::blocking::Client::new();
        let mut entitlements_url =
            Url::parse("https://api.minecraftservices.com/entitlements/license")
                .map_err(|e| AuthError::Network(e.to_string()))?;
        entitlements_url
            .query_pairs_mut()
            .append_pair("requestId", &uuid::Uuid::new_v4().to_string());
        let response = client
            .get(entitlements_url)
            .header("Authorization", format!("Bearer {}", ygg_token))
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Entitlements check failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AuthError::Network(e.to_string()))?;
        let entitlement = parsers::parse_minecraft_entitlements(&bytes).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse entitlements".to_string())
        })?;

        data.minecraft_entitlement = entitlement;
        debug!(
            "Entitlements: owns={}, canPlay={}",
            data.minecraft_entitlement.owns_minecraft,
            data.minecraft_entitlement.can_play_minecraft
        );

        Ok(AccountTaskState::Working)
    }
}

struct MinecraftProfileStep;

impl crate::AuthStep for MinecraftProfileStep {
    fn describe(&self) -> &str {
        "Minecraft Profile"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
        let ygg_token = data
            .yggdrasil_token
            .token
            .as_deref()
            .ok_or_else(|| AuthError::Auth("No Yggdrasil token for profile".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .header("Authorization", format!("Bearer {}", ygg_token))
            .send()
            .map_err(|e| AuthError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::Auth(format!(
                "Profile fetch failed: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .map_err(|e| AuthError::Network(e.to_string()))?;
        let profile = parsers::parse_minecraft_profile(&bytes).ok_or_else(|| {
            AuthError::InvalidResponse("Failed to parse Minecraft profile".to_string())
        })?;

        data.minecraft_profile = profile;
        debug!(
            "Profile: name={}, id={}",
            data.minecraft_profile.name, data.minecraft_profile.id
        );

        Ok(AccountTaskState::Succeeded)
    }
}
