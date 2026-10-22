use std::{io::ErrorKind, path::PathBuf};

use serde_json::from_str;

use crate::{AttackDefinition, AttackId, Store, StoreError};

pub struct JsonStore {
    root: PathBuf,
}

impl JsonStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn attack_path(&self, id: &AttackId) -> PathBuf {
        self.root.join("attacks").join(format!("{}.json", id.0))
    }
}
impl Store for JsonStore {
    fn get_attack(&self, id: &AttackId) -> Result<AttackDefinition, StoreError> {
        let path = self.attack_path(id);
        let text = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => StoreError::NotFound {
                kind: ("attack"),
                id: (id.0.clone()),
            },
            _ => StoreError::Io {
                path: path.clone(),
                source: e,
            },
        })?;
        from_str(&text).map_err(|source| StoreError::Parse { path, source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> JsonStore {
        JsonStore::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"))
    }

    #[test]
    fn loads_fireball() {
        let fireball = store()
            .get_attack(&AttackId("fireball".into()))
            .expect("fireball.json should load");

        assert_eq!(fireball.name, "fireball");
        assert_eq!(fireball.residue, Some(AttackId("fire_patch".into())));
    }

    #[test]
    fn load_firepatch() {
        let fire_patch = store()
            .get_attack(&AttackId("fire_patch".into()))
            .expect("fireball.json should load");

        assert_eq!(fire_patch.name, "fire_patch")
    }

    #[test]
    fn missing_attack_gives_not_found() {
        let err = store()
            .get_attack(&AttackId("does_not_exist".into()))
            .unwrap_err();

        assert!(matches!(err, StoreError::NotFound { .. }));
    }
}
