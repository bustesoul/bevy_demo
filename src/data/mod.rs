pub mod schema;
pub mod loader;

use bevy::prelude::*;
use schema::ItemList;
use crate::core::states::AppState;

// --------------------------- 资源 ---------------------------
#[derive(Resource, Default)]
pub struct ItemAssets {
    handle: Option<Handle<ItemList>>,
}

// --------------------------- 插件 ---------------------------
pub struct DataPlugin;
impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册资产类型 & Loader
            .init_asset::<ItemList>()
            .register_asset_loader(loader::RonItemLoader::default())
            // 注册资源
            .init_resource::<ItemAssets>()
            // Loading 流程
            .add_systems(OnEnter(AppState::Loading), start_loading)
            .add_systems(
                Update,
                check_loaded.run_if(in_state(AppState::Loading)),
            );
    }
}

// --------------------------- 系统 ---------------------------
fn start_loading(
    mut item_assets: ResMut<ItemAssets>,
    asset_server: Res<AssetServer>,
) {
    let handle: Handle<ItemList> = asset_server.load("data/items.ron");
    item_assets.handle = Some(handle);
}

fn check_loaded(
    mut next: ResMut<NextState<AppState>>,
    item_assets: Res<ItemAssets>,
    lists: Res<Assets<ItemList>>,
) {
    if let Some(h) = &item_assets.handle {
        if let Some(list) = lists.get(h) {
            println!("✔ Items loaded: {}", list.items.len());
            next.set(AppState::InGame);
        }
    }
}