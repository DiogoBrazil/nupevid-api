use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCity {
    pub name: String,
    pub state: String,
    pub battalion: String,
}
