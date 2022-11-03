use bevy::{log::LogSettings, prelude::*, utils::tracing::Level, window::WindowResizeConstraints};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_kira_audio::prelude::*;
use bevy_pixels::prelude::*;
use bitmap::BitmapPlugin;

mod bitmap;
mod scenes;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn main() {
    let width = WIDTH as f32 * 2.0;
    let height = HEIGHT as f32 * 2.0;

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Blitter Blaster".to_string(),
            width,
            height,
            resize_constraints: WindowResizeConstraints {
                min_width: width,
                min_height: height,
                ..default()
            },
            // mode: bevy::window::WindowMode::BorderlessFullscreen,
            fit_canvas_to_parent: true,
            ..default()
        })
        .insert_resource(PixelsOptions {
            width: WIDTH,
            height: HEIGHT,
        })
        .insert_resource(log_settings())
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
        })
        .add_plugin(AudioPlugin)
        .add_plugin(PixelsPlugin)
        .add_plugin(BitmapPlugin)
        .add_plugin(scenes::intro::ScenePlugin)
        // .add_plugin(scenes::title::ScenePlugin)
        .add_state(scenes::GameState::Intro)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn log_settings() -> LogSettings {
    #[cfg(not(feature = "optimize"))]
    let level = Level::INFO;
    #[cfg(feature = "optimize")]
    let level = Level::ERROR;

    let level = std::env::var("LOG_LEVEL")
        .map(|level| match level.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            level => {
                eprintln!("Unknown log level: {level}");
                Level::INFO
            }
        })
        .unwrap_or(level);
    let filter =
        std::env::var("LOG_FILTER").unwrap_or_else(|_| "wgpu=error,symphonia=error".to_string());

    LogSettings { level, filter }
}
