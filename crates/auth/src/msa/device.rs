use kcraft_core::account::{AccountData, AccountTaskState, Token, Validity};
use tracing::info;

use crate::AuthError;
use crate::Result;

use super::flow::VerificationCallback;

pub(crate) struct MsaDeviceCodeStep {
    pub(crate) client_id: String,
    pub(crate) verification_callback: Option<VerificationCallback>,
}

impl crate::AuthStep for MsaDeviceCodeStep {
    fn describe(&self) -> &str {
        "MSA Login"
    }

    fn perform(&mut self, data: &mut AccountData) -> Result<AccountTaskState> {
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

                data.msa_token = Token {
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

        Ok(AccountTaskState::Working)
    }
}
