pub use self::{bitmap::*, camera::*, collision::*, config::*};
use bevy::prelude::*;

mod bitmap;
mod camera;
mod collision;
mod config;

#[derive(Debug)]
pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ConfigPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(CollisionPlugin);
    }
}
