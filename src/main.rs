use bevy::prelude::*;

mod core;
mod interface;

use core::CorePlugin;
use interface::debug_cli::DebugCliPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                visible: false,
                ..default()
            }),   // visible窗口，实现“无 UI”
            ..default()
        }))
        .add_plugins(CorePlugin)
        .add_plugins(DebugCliPlugin)
        .add_systems(Update, forward_log_event) // 简单打印
        .run();
}

fn forward_log_event(mut reader: EventReader<core::events::LogEvent>) {
    for e in reader.read() {
        println!("> {}", e.0);
    }
}