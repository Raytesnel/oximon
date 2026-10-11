use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonsterId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonsterDef {
    pub id: MonsterId,
    pub name: String,
}
