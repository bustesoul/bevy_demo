pub mod components;
pub mod events;
pub mod systems;

use crate::core::states::AppState;
use bevy::prelude::*;
use events::*;
use systems::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册事件
            .add_event::<EquipmentChanged>()
            .add_event::<GainExp>()
            .add_event::<TakeDamage>()
            .add_event::<Death>()
            .add_event::<LevelUp>()
            .add_event::<Heal>()
            .add_event::<RecalculateStats>()
            .add_event::<ShowStats>()
            // 在游戏开始时生成玩家
            .add_systems(OnEnter(AppState::InGame), spawn_player)
            // 游戏中的系统
            .add_systems(
                Update,
                (
                    handle_gain_exp,
                    handle_take_damage,
                    handle_heal,
                    handle_equipment_changed,
                    recalculate_stats,
                    show_stats,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
