use bevy::prelude::*;

#[derive(Event)]
pub struct EquipEvent {
    pub slot: String, // head, body, weapon, accessory
    pub index: usize, // 背包索引
}

#[derive(Event)]
pub struct UnequipEvent {
    pub slot: String, // head, body, weapon, accessory
}
