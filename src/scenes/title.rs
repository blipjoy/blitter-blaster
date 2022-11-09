use super::GameState;
use crate::bitmap::Bitmap;
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetIo;
use bevy_kira_audio::prelude::*;
use bevy_pixels::*;
use std::path::Path;

pub struct TitlePlugin;

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct Background;

struct Motion {
    angle: f32,
    magnitude: f32,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            angle: 0.0,
            magnitude: 64.0,
        }
    }
}

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Motion>()
            .add_system_set(SystemSet::on_enter(GameState::Title).with_system(Self::enter))
            .add_system_set(SystemSet::on_update(GameState::Title).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(GameState::Title).with_system(Self::exit));
    }
}

impl TitlePlugin {
    fn enter(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        options: Res<PixelsOptions>,
        audio: Res<Audio>,
    ) {
        audio
            .play(asset_server.load("music/getting-started.ogg"))
            .looped();

        let io = asset_server
            .asset_io()
            .downcast_ref::<EmbeddedAssetIo>()
            .unwrap();

        // Spawn the background
        let transform = Transform::from_xyz(0.0, 0.0, 1.0);
        let transform_bundle = TransformBundle::from_transform(transform);
        let image = io.load_path_sync(Path::new("images/bg1.png")).unwrap();
        commands
            .spawn()
            .insert(Bitmap::new(&image).tiled(true))
            .insert_bundle(transform_bundle)
            .insert(Background)
            .insert(TitleScreen);

        // Spawn the title logo
        let x = (options.width / 2) as f32;
        let transform = Transform::from_xyz(x - 120.0, 65.0, 2.0);
        let transform_bundle = TransformBundle::from_transform(transform);
        let image = io.load_path_sync(Path::new("images/odonata.png")).unwrap();
        commands
            .spawn()
            .insert(Bitmap::new(&image))
            .insert_bundle(transform_bundle)
            .insert(TitleScreen);
    }

    fn update(
        time: Res<Time>,
        mut query: Query<&mut Transform, With<Background>>,
        mut motion: ResMut<Motion>,
    ) {
        let delta = time.delta().as_secs_f32();

        let velocity = Quat::from_rotation_z(motion.angle) * Vec3::X * motion.magnitude;

        for mut transform in &mut query {
            let z = transform.translation.z;
            transform.translation -= velocity * delta * z;
        }

        motion.angle += 0.000033;
    }

    fn exit(mut commands: Commands, entities: Query<Entity, With<TitleScreen>>) {
        for entity in &entities {
            commands.entity(entity).despawn_recursive();
        }
    }
}
