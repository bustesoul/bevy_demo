pub mod components;
pub mod events;
mod systems;

use bevy::prelude::*;
use crate::core::states::AppState;
use components::*;
use events::*;
use systems::*;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Backpack {
                slots: vec![ItemStack { proto: Default::default(), count: 0 }; 30],
                capacity: 30,
            })
            .add_event::<GiveItemEvent>()
            .add_event::<ListInventoryEvent>()
            .add_systems(
                Update,
                (
                    give_item,
                    print_inventory,
                ).run_if(in_state(AppState::InGame)),
            );
    }
}
