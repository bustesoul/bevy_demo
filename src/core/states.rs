use bevy::prelude::*;

/// 游戏运行的大状态
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Startup,
    Loading,
    InGame,
    Shutdown,
}