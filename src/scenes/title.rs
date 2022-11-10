use super::GameState;
use crate::bitmap::BitmapCache;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_pixels::*;

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
        mut cache: ResMut<BitmapCache>,
        asset_server: Res<AssetServer>,
        options: Res<PixelsOptions>,
        audio: Res<Audio>,
    ) {
        audio
            .play(asset_server.load("music/getting-started.ogg"))
            .looped();

        // Spawn the background
        let transform = Transform::from_xyz(0.0, 0.0, 1.0);
        let transform_bundle = TransformBundle::from_transform(transform);
        let bitmap = cache.get_or_create("images/bg1.png", &asset_server);
        commands
            .spawn()
            .insert(bitmap.tiled(true))
            .insert_bundle(transform_bundle)
            .insert(Background)
            .insert(TitleScreen);

        // Spawn the title logo
        let x = (options.width / 2) as f32;
        let transform = Transform::from_xyz(x - 120.0, 65.0, 2.0);
        let transform_bundle = TransformBundle::from_transform(transform);
        let bitmap = cache.get_or_create("images/odonata.png", &asset_server);
        commands
            .spawn()
            .insert(bitmap)
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
