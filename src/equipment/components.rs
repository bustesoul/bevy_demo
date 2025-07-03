use bevy::prelude::*;
use crate::inventory::components::ItemStack;

/// 简化为只有 weapon 一格
#[derive(Resource, Default)]
pub struct Equipment {
    pub weapon: Option<ItemStack>,
}
