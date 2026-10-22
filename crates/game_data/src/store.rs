use crate::{AttackDefinition, AttackId, StoreError};

pub trait Store {
    // fn list_attacks(&self) -> Result<Vec<AttackId>, StoreError>;
    fn get_attack(&self, id: &AttackId) -> Result<AttackDefinition, StoreError>;
    // fn save_attack(&mut self, def: &AttackDefinition) -> Result<(), StoreError>;
    // fn delete_attack(&mut self, id: &AttackId) -> Result<(), StoreError>;
}

// impl Store for JsonStore {}
