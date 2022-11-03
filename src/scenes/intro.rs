use super::GameState;
use crate::bitmap::Bitmap;
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetIo;
use bevy_kira_audio::prelude::*;
use std::path::Path;

pub struct ScenePlugin;

#[derive(Component)]
struct IntroScreen;

struct IntroState {
    anim: Vec<Anim>,
    timer: Timer,
}

struct Anim {
    duration: f32,
    pos: (i32, i32),
    image: Vec<u8>,
    sfx: Option<Handle<AudioSource>>,
}

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Intro).with_system(Self::enter))
            .add_system_set(SystemSet::on_update(GameState::Intro).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(GameState::Intro).with_system(Self::exit));
    }
}

impl ScenePlugin {
    fn enter(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.insert_resource(IntroState::new(&asset_server));
    }

    fn update(
        mut commands: Commands,
        mut game_state: ResMut<State<GameState>>,
        time: Res<Time>,
        mut state: ResMut<IntroState>,
        audio: Res<Audio>,
    ) {
        if state.timer.tick(time.delta()).finished() {
            if let Some(anim) = state.anim.pop() {
                let Anim {
                    duration,
                    pos,
                    image,
                    sfx,
                } = anim;

                commands
                    .spawn()
                    .insert(Bitmap::new(pos, &image))
                    .insert(IntroScreen);

                if let Some(sfx) = sfx {
                    audio.play(sfx);
                }

                state.timer = Timer::from_seconds(duration, false);
            } else {
                game_state.set(GameState::Title).unwrap();
            }
        }
    }

    fn exit(mut commands: Commands, entities: Query<Entity, With<IntroScreen>>) {
        commands.remove_resource::<IntroState>();
        for entity in &entities {
            commands.entity(entity).despawn_recursive();
        }
    }
}

impl IntroState {
    fn new(asset_server: &Res<AssetServer>) -> Self {
        Self {
            anim: vec![
                Anim::load(
                    asset_server,
                    2.0,
                    (250, 140),
                    "images/logo-y.png",
                    Some("sfx/blip7.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.5,
                    (210, 140),
                    "images/logo-o.png",
                    Some("sfx/blip6.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.25,
                    (170, 140),
                    "images/logo-j.png",
                    Some("sfx/blip5.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.15,
                    (140, 140),
                    "images/logo-p.png",
                    Some("sfx/blip4.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.15,
                    (120, 140),
                    "images/logo-i.png",
                    Some("sfx/blip3.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.5,
                    (80, 140),
                    "images/logo-l.png",
                    Some("sfx/blip2.ogg"),
                ),
                Anim::load(
                    asset_server,
                    0.2,
                    (40, 140),
                    "images/logo-b.png",
                    Some("sfx/blip1.ogg"),
                ),
                Anim::load(asset_server, 0.5, (120, 50), "images/logo.png", None),
            ],
            timer: Timer::from_seconds(0.0, false),
        }
    }
}

impl Anim {
    fn load(
        asset_server: &Res<AssetServer>,
        duration: f32,
        pos: (i32, i32),
        image: &str,
        sfx: Option<&str>,
    ) -> Self {
        let io = asset_server
            .asset_io()
            .downcast_ref::<EmbeddedAssetIo>()
            .unwrap();

        Self {
            duration,
            pos,
            image: io.load_path_sync(Path::new(image)).unwrap(),
            sfx: sfx.map(|path| asset_server.load(path)),
        }
    }
}
