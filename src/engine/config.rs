use crate::consts::{APP_NAME, HEIGHT, WIDTH_STANDARD, WIDTH_ULTRAWIDE, WIDTH_WIDE};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::LogSettings,
    prelude::*,
    utils::tracing::Level,
};
use directories::ProjectDirs;

#[derive(Debug)]
pub struct ConfigPlugin;

#[derive(Debug)]
pub struct ConfigState {
    dirs: ProjectDirs,
    ar: AspectRatio,
    log_config: LogConfig,
    fps: bool,
}

#[derive(Debug)]
pub enum SaveEvent {
    /// Screen aspect ratio.
    AspectRatio(AspectRatio),
}

#[derive(Copy, Clone, Debug)]
pub enum AspectRatio {
    Standard,
    Wide,
    Ultrawide,
}

#[derive(Clone, Debug)]
struct LogConfig {
    level: Level,
    filter: String,
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = ConfigState::default();
        let fps = config.fps;

        app.insert_resource(config)
            .add_event::<SaveEvent>()
            .add_system(save_config);

        if fps {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}

fn save_config(mut state: ResMut<ConfigState>, mut events: EventReader<SaveEvent>) {
    for event in events.iter() {
        match event {
            SaveEvent::AspectRatio(ar) => {
                state.ar = *ar;
            }
        }
    }
}

impl Default for ConfigState {
    fn default() -> Self {
        let dirs =
            ProjectDirs::from("com", "BlipJoy", APP_NAME).expect("Could not find home directory");

        // TODO: Load state from the config file
        let ar = AspectRatio::Standard;

        let fps = std::env::var("FPS")
            .ok()
            .map(|fps| fps == "1")
            .unwrap_or_default();

        #[cfg(not(feature = "optimize"))]
        let level = Level::INFO;
        #[cfg(feature = "optimize")]
        let level = if fps { Level::INFO } else { Level::ERROR };

        // TODO: Load state from the config file
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
        let filter = std::env::var("LOG_FILTER")
            .unwrap_or_else(|_| "wgpu=error,symphonia=error".to_string());

        let log_config = LogConfig { level, filter };

        Self {
            dirs,
            ar,
            log_config,
            fps,
        }
    }
}

impl ConfigState {
    pub fn aspect_ratio(&self) -> AspectRatio {
        self.ar
    }

    pub fn screen_resolution(&self) -> (u32, u32) {
        let width = match self.aspect_ratio() {
            AspectRatio::Standard => WIDTH_STANDARD,
            AspectRatio::Wide => WIDTH_WIDE,
            AspectRatio::Ultrawide => WIDTH_ULTRAWIDE,
        };

        (width, HEIGHT)
    }

    pub fn log_settings(&self) -> LogSettings {
        LogSettings {
            level: self.log_config.level,
            filter: self.log_config.filter.clone(),
        }
    }
}
