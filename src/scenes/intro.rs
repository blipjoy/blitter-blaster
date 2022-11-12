use super::GameState;
use crate::{
    bitmap::{Bitmap, BitmapCache},
    camera::Camera,
};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_pixels::*;
use pix::rgb::Rgba8p;

pub struct IntroPlugin;

#[derive(Component)]
struct IntroScreen;

struct IntroState {
    anim: Vec<Anim>,
    timer: Timer,
    fading: bool,
}

struct Anim {
    duration: f32,
    pos: Transform,
    image: Bitmap,
    sfx: Option<Handle<AudioSource>>,
}

struct AnimLoader<'a> {
    asset_server: Res<'a, AssetServer>,
    cache: ResMut<'a, BitmapCache>,
}

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Intro).with_system(Self::enter))
            .add_system_set(SystemSet::on_update(GameState::Intro).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(GameState::Intro).with_system(Self::exit));
    }
}

impl IntroPlugin {
    fn enter(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        cache: ResMut<BitmapCache>,
        options: Res<PixelsOptions>,
    ) {
        commands.insert_resource(IntroState::new(asset_server, cache, options.width));
    }

    fn update(
        mut commands: Commands,
        mut game_state: ResMut<State<GameState>>,
        mut state: ResMut<IntroState>,
        time: Res<Time>,
        audio: Res<Audio>,
        options: Res<PixelsOptions>,
    ) {
        if state.timer.tick(time.delta()).finished() {
            if let Some(anim) = state.anim.pop() {
                commands
                    .spawn()
                    .insert(anim.image)
                    .insert(anim.pos)
                    .insert(IntroScreen);

                if let Some(sfx) = anim.sfx {
                    audio.play(sfx);
                }

                state.timer = Timer::from_seconds(anim.duration, false);
            } else {
                game_state.set(GameState::Title).unwrap();
            }
        } else if !state.fading && state.anim.is_empty() && state.timer.percent() >= 0.5 {
            state.fading = true;

            let color = Rgba8p::new(0.0, 0.0, 0.0, 1.0);
            let fade_bundle = Camera::fade_out(1.0, options.width, options.height, color);
            commands
                .spawn()
                .insert_bundle(fade_bundle)
                .insert(IntroScreen);
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
    fn new(asset_server: Res<AssetServer>, cache: ResMut<BitmapCache>, width: u32) -> Self {
        let mut loader = AnimLoader::new(asset_server, cache);
        let hw = width as i32 / 2;

        Self {
            anim: vec![
                loader.load(2.0, (hw + 90, 140), "logo-y.png", Some("blip7.ogg")),
                loader.load(0.5, (hw + 50, 140), "logo-o.png", Some("blip6.ogg")),
                loader.load(0.25, (hw + 10, 140), "logo-j.png", Some("blip5.ogg")),
                loader.load(0.15, (hw - 20, 140), "logo-p.png", Some("blip4.ogg")),
                loader.load(0.15, (hw - 40, 140), "logo-i.png", Some("blip3.ogg")),
                loader.load(0.5, (hw - 80, 140), "logo-l.png", Some("blip2.ogg")),
                loader.load(0.2, (hw - 120, 140), "logo-b.png", Some("blip1.ogg")),
                loader.load(0.5, (hw - 40, 50), "logo.png", None),
            ],
            timer: Timer::from_seconds(0.0, false),
            fading: false,
        }
    }
}

impl<'a> AnimLoader<'a> {
    fn new(asset_server: Res<'a, AssetServer>, cache: ResMut<'a, BitmapCache>) -> Self {
        Self {
            asset_server,
            cache,
        }
    }

    fn load(&mut self, duration: f32, pos: (i32, i32), image: &str, sfx: Option<&str>) -> Anim {
        let pos = Transform::from_xyz(pos.0 as f32, pos.1 as f32, 1.0);
        let image = self
            .cache
            .get_or_create(&format!("images/{image}"), &self.asset_server);
        let sfx = sfx.map(|path| self.asset_server.load(&format!("sfx/{path}")));

        Anim {
            duration,
            pos,
            image,
            sfx,
        }
    }
}
