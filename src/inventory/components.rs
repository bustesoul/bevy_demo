use bevy::prelude::*;
use crate::data::schema::ItemEntry;

/// 玩家背包（挂在 Resource）
#[derive(Resource, Default)]
pub struct Backpack {
    pub slots: Vec<ItemStack>,   // 固定容量，空位用 count=0 占位
    pub capacity: usize,
}

/// 运行时物品实例
#[derive(Clone)]
pub struct ItemStack {
    pub proto: ItemEntry,  // 直接复制静态表条目即可
    pub count: u32,
}
