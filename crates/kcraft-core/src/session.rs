use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub client_token: String,
    pub username: String,
    pub session: String,
    pub access_token: String,
    pub player_name: String,
    pub uuid: String,
    pub user_type: String,
    pub authlib_injector_base_url: String,
    pub auth_server_online: bool,
    pub wants_online: bool,
    pub demo: bool,
}

impl AuthSession {
    pub fn new_offline(player_name: &str, uuid: &str) -> Self {
        let session = "-".to_string();
        AuthSession {
            client_token: String::new(),
            username: player_name.to_string(),
            session,
            access_token: String::new(),
            player_name: player_name.to_string(),
            uuid: uuid.to_string(),
            user_type: "offline".to_string(),
            authlib_injector_base_url: String::new(),
            auth_server_online: false,
            wants_online: false,
            demo: false,
        }
    }

    pub fn new_online(
        client_token: String,
        username: String,
        access_token: String,
        player_name: String,
        uuid: String,
        user_type: String,
    ) -> Self {
        let session = format!("token:{}:{}", access_token, uuid);
        AuthSession {
            client_token,
            username,
            session,
            access_token,
            player_name,
            uuid,
            user_type,
            authlib_injector_base_url: String::new(),
            auth_server_online: true,
            wants_online: true,
            demo: false,
        }
    }

    pub fn serialize_user_properties(&self) -> String {
        serde_json::json!({
            "user_properties": serde_json::Value::Object(serde_json::Map::new()),
        })
        .to_string()
    }
}

impl Default for AuthSession {
    fn default() -> Self {
        AuthSession {
            client_token: String::new(),
            username: String::new(),
            session: "-".to_string(),
            access_token: String::new(),
            player_name: String::new(),
            uuid: String::new(),
            user_type: "offline".to_string(),
            authlib_injector_base_url: String::new(),
            auth_server_online: false,
            wants_online: false,
            demo: false,
        }
    }
}
