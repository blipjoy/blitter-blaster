use bevy::{log::LogSettings, prelude::*, utils::tracing::Level, window::WindowResizeConstraints};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_kira_audio::prelude::*;
use bevy_pixels::prelude::*;
use bitmap::BitmapPlugin;

mod bitmap;
mod scenes;

fn main() {
    let (width, height) = screen_resolution();
    let width_x2 = width as f32 * 2.0;
    let height_x2 = height as f32 * 2.0;

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Blitter Blaster".to_string(),
            width: width_x2,
            height: height_x2,
            resize_constraints: WindowResizeConstraints {
                min_width: width_x2,
                min_height: height_x2,
                ..default()
            },
            // mode: bevy::window::WindowMode::BorderlessFullscreen,
            fit_canvas_to_parent: true,
            ..default()
        })
        .insert_resource(PixelsOptions { width, height })
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

fn screen_resolution() -> (u32, u32) {
    // Define screen resolutions for aspect ratios 4:3, 16:9, and 21:9.
    const WIDTH_4X3: u32 = 320;
    const WIDTH_16X9: u32 = 427;
    const WIDTH_21X9: u32 = 560;
    const HEIGHT: u32 = 240;

    // TODO: Get aspect ratio setting from configuration
    let width = match std::env::var("SCREEN_RES")
        .unwrap_or_else(|_| "0".to_string())
        .as_str()
    {
        "0" => WIDTH_4X3,
        "1" => WIDTH_16X9,
        "2" => WIDTH_21X9,
        _ => panic!("Unimplemented screen resolution; valid options are: 0,1,2"),
    };

    (width, HEIGHT)
}
