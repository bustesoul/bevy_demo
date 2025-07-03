use bevy::prelude::*;

#[derive(Event)]
pub struct EquipEvent {
    pub slot: String,   // weapon
    pub index: usize,   // 背包索引
}
