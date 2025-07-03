use bevy::prelude::*;

pub mod states;
pub mod events;
pub mod resources;

/// 核心插件：注册全局资源 / 事件 / 状态
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        use states::AppState;

        // 插件首次载入时，插入初始 State
        app
            .init_state::<AppState>()
            .add_event::<events::LogEvent>()          // 示例事件
            .init_resource::<resources::GameConfig>() // 示例资源
            .add_systems(Startup, events::hello_world);
    }
}