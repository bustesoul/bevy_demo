pub mod components;
pub mod events;
mod systems;

use bevy::prelude::*;
use crate::core::states::AppState;
use components::*;
use events::*;
use systems::*;

pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Equipment::default())
            .add_event::<EquipEvent>()
            .add_systems(
                Update,
                equip_item.run_if(in_state(AppState::InGame)),
            );
    }
}
