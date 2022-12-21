use bevy::prelude::*;

pub mod bitmap;
pub mod camera;
pub mod collision;
pub mod config;

#[derive(Debug)]
pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(config::ConfigPlugin)
            .add_plugin(camera::CameraPlugin)
            .add_plugin(collision::CollisionPlugin);
    }
}
