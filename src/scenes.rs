use bevy::prelude::*;

pub mod intro;
pub mod title;

#[derive(Debug)]
pub struct ScenePlugin;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Intro,
    Title,
    Game,
}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(intro::IntroPlugin)
            .add_plugin(title::TitlePlugin);
    }
}
