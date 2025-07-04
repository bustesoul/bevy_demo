use bevy::prelude::*;

#[derive(Event)]
pub struct GiveItemEvent {
    pub id: String,
    pub count: u32,
}

#[derive(Event)]
pub struct ListInventoryEvent; // 让 CLI 请求打印背包

#[derive(Event)]
pub struct UseItemEvent {
    pub index: usize, // 背包索引
}
