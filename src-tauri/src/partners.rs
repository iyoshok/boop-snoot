use serde::{
    Deserialize,
    Serialize
};

pub const PARTNERS_FILE: &str = "../boop.partners.json";  //TODO: change to same directory later

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BoopPartner {
    nickname: String,
    user_key: String
}

impl BoopPartner {
    pub fn nickname(&self) -> String {
        self.nickname.clone()
    }

    pub fn user_key(&self) -> String {
        self.user_key.clone()
    }
}
