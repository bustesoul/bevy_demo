pub mod components;
pub mod events;
mod systems;

use crate::core::states::AppState;
use bevy::prelude::*;
use components::*;
use events::*;
use systems::*;

pub struct EquipmentPlugin;
impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Equipment::default())
            .add_event::<EquipEvent>()
            .add_event::<UnequipEvent>()
            .add_systems(
                Update,
                (equip_item, unequip_item).run_if(in_state(AppState::InGame)),
            );
    }
}
