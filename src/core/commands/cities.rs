use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CreateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UpdateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}
