use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::Deserialize;
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ItemEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub atk: i32,
    #[serde(default)]
    pub def: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default)]
    pub rng: i32,
    #[serde(default)]
    pub heal: i32,
    #[serde(default)]
    pub item_type: String, // "weapon", "armor", "potion", "scroll", "key", etc.
}

#[derive(Asset, TypePath, Deserialize, Debug)]
pub struct ItemList {
    pub items: Vec<ItemEntry>,
}

// 在顶层模块中定义常量句柄
// pub const ITEM_LIST_HANDLE: Handle<ItemList> =
//     weak_handle!("bddb7d8c-1e02-4b56-ba3e-47779fba3992");
