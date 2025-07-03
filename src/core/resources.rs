use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct GameConfig {
    pub _tick_rate: f32,
}