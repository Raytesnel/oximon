use std::collections::HashMap;

use bevy::prelude::*;
use game_data::{AttackDefinition, AttackId, JsonStore, Store, StoreError};

#[derive(Resource, Default)]
pub struct Attacks(HashMap<AttackId, AttackDefinition>);

impl Attacks {
    pub fn load(store: &impl Store, ids: &[&str]) -> Result<Self, StoreError> {
        let mut map = HashMap::new();
        let mut queue: Vec<AttackId> = ids.iter().map(|s| AttackId(s.to_string())).collect();

        while let Some(id) = queue.pop() {
            if map.contains_key(&id) {
                continue;
            }
            let def = store.get_attack(&id)?;
            if let Some(r) = &def.residue {
                queue.push(r.clone());
            }
            map.insert(id, def);
        }
        Ok(Self(map))
    }

    pub fn get(&self, id: &str) -> Option<&AttackDefinition> {
        self.0.get(&AttackId(id.to_string()))
    }

    pub fn residue_of(&self, def: &AttackDefinition) -> Option<&AttackDefinition> {
        def.residue.as_ref().and_then(|r| self.0.get(r))
    }
}

pub fn asset_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

pub fn load_attacks(mut commands: Commands) {
    let store = JsonStore::new(asset_root());
    let attacks = Attacks::load(
        &store,
        &[
            "quick_attack",
            "speedo",
            "shoot_square",
            "fireball",
            "stone_throw",
        ],
    )
    .expect("failed to load attacks");
    commands.insert_resource(attacks);
}
