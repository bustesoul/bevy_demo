use bevy::prelude::*;

mod character;
mod core;
mod data;
mod equipment;
mod interface;
mod inventory;

use crate::character::CharacterPlugin;
use crate::core::CorePlugin;
use crate::core::states;
use crate::data::DataPlugin;
use crate::equipment::EquipmentPlugin;
use crate::interface::debug_cli::DebugCliPlugin;
use crate::inventory::InventoryPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                visible: false,
                ..default()
            }), // visible窗口，实现“无 UI”
            ..default()
        }))
        // 核心插件
        .add_plugins(CorePlugin)
        // 功能插件
        .add_plugins(CharacterPlugin)
        .add_plugins(DataPlugin)
        .add_plugins(InventoryPlugin)
        .add_plugins(EquipmentPlugin)
        // 交互插件
        .add_plugins(DebugCliPlugin)
        // 全局系统
        .add_systems(Update, forward_log_event) // 简单打印
        .add_systems(Startup, |mut next: ResMut<NextState<states::AppState>>| {
            next.set(states::AppState::Loading);
        })
        .run();
}

fn forward_log_event(mut reader: EventReader<core::events::LogEvent>) {
    use crate::interface::debug_cli::{queue_log_message};
    
    for e in reader.read() {
        queue_log_message(e.0.clone());
    }
}
