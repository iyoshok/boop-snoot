use serde::{
    Deserialize,
    Serialize
};

pub const CONFIG_FILE: &str = "../boop.config.json"; //TODO: change to same directory later

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoopConfig {
    // server config
    server_address: String,

    // login data
    user:     String,
    password: String
}

impl Default for BoopConfig {
    fn default() -> Self {
        BoopConfig {
            server_address: String::new(),
            user:           String::new(),
            password:       String::new()
        }
    }
}

impl BoopConfig {
    pub fn server_address(&self) -> String {
        self.server_address.clone()
    }

    pub fn user_name(&self) -> String {
        self.user.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}
