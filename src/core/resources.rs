use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameConfig {
    pub tick_rate: f32,
}