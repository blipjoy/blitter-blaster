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
    pos: TransformBundle,
    image: Vec<u8>,
    sfx: Option<Handle<AudioSource>>,
}

struct AnimLoader<'a> {
    asset_server: Res<'a, AssetServer>,
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
        commands.insert_resource(IntroState::new(asset_server));
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
                    .insert(Bitmap::new(&image))
                    .insert_bundle(pos)
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
    fn new(asset_server: Res<AssetServer>) -> Self {
        let loader = AnimLoader::new(asset_server);

        Self {
            anim: vec![
                loader.load(2.0, (250, 140), "logo-y.png", Some("blip7.ogg")),
                loader.load(0.5, (210, 140), "logo-o.png", Some("blip6.ogg")),
                loader.load(0.25, (170, 140), "logo-j.png", Some("blip5.ogg")),
                loader.load(0.15, (140, 140), "logo-p.png", Some("blip4.ogg")),
                loader.load(0.15, (120, 140), "logo-i.png", Some("blip3.ogg")),
                loader.load(0.5, (80, 140), "logo-l.png", Some("blip2.ogg")),
                loader.load(0.2, (40, 140), "logo-b.png", Some("blip1.ogg")),
                loader.load(0.5, (120, 50), "logo.png", None),
            ],
            timer: Timer::from_seconds(0.0, false),
        }
    }
}

impl<'a> AnimLoader<'a> {
    fn new(asset_server: Res<'a, AssetServer>) -> Self {
        Self { asset_server }
    }

    fn load(&self, duration: f32, pos: (i32, i32), image: &str, sfx: Option<&str>) -> Anim {
        let io = self
            .asset_server
            .asset_io()
            .downcast_ref::<EmbeddedAssetIo>()
            .unwrap();

        let transform = Transform::from_xyz(pos.0 as f32, pos.1 as f32, 0.0);
        let pos = TransformBundle::from_transform(transform);
        let image = io
            .load_path_sync(Path::new(&format!("images/{image}")))
            .unwrap();
        let sfx = sfx.map(|path| self.asset_server.load(&format!("sfx/{path}")));

        Anim {
            duration,
            pos,
            image,
            sfx,
        }
    }
}
