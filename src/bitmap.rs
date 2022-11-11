use ahash::RandomState;
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetIo;
use bevy_pixels::prelude::*;
use pix::{
    chan::{Ch8, Channel as _},
    el::Pixel,
    ops::SrcOver,
    rgb::Rgba8p,
    Raster,
};
use std::{collections::HashMap, io::Cursor, path::Path, sync::Arc};

#[derive(Debug)]
pub struct BitmapPlugin;

#[derive(Clone, Component)]
pub struct Bitmap {
    tiled: bool,
    raster: Arc<Raster<Rgba8p>>,
}

struct TileIter {
    current: i32,
    step: i32,
    end: i32,
}

#[derive(Default)]
pub struct BitmapCache {
    map: HashMap<String, Bitmap, RandomState>,
}

#[derive(Debug)]
pub struct FadePlugin;

#[derive(Component, Debug)]
pub struct Fade {
    timer: Timer,
    from: f32,
    to: f32,
    base_color: Rgba8p,
}

impl Plugin for BitmapPlugin {
    fn build(&self, app: &mut App) {
        let PixelsOptions { width, height } = *app.world.resource::<PixelsOptions>();

        app.insert_resource(Raster::<Rgba8p>::with_clear(width, height))
            .init_resource::<BitmapCache>()
            .add_system_to_stage(PixelsStage::Draw, Self::update);
    }
}

impl BitmapPlugin {
    fn update(
        mut pixels_res: ResMut<PixelsResource>,
        mut raster: ResMut<Raster<Rgba8p>>,
        query: Query<(&Bitmap, &Transform)>,
    ) {
        raster.clear();

        // Sort by Z coordinate
        let mut bitmaps: Vec<_> = query.iter().collect();
        bitmaps.sort_unstable_by_key(|(_, t)| (t.translation.z * 1000.0) as i64);

        for (bitmap, transform) in &bitmaps {
            if bitmap.tiled {
                // Iterate over all ranges required to fill the frame with the bitmap.

                let x_start = transform.translation.x as i32;
                let x_end = raster.width().try_into().unwrap();
                let y_start = transform.translation.y as i32;
                let y_end = raster.height().try_into().unwrap();

                for y in bitmap.tile_cols(y_start, y_end) {
                    for x in bitmap.tile_rows(x_start, x_end) {
                        raster.composite_raster((x, y), &bitmap.raster, (), SrcOver);
                    }
                }
            } else {
                let to = (
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                );

                raster.composite_raster(to, &bitmap.raster, (), SrcOver);
            }
        }

        pixels_res
            .pixels
            .get_frame_mut()
            .copy_from_slice(raster.as_u8_slice());
    }
}

impl Bitmap {
    fn new(bytes: &[u8]) -> Self {
        let decoder = png::Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let raster = Arc::new(Raster::with_u8_buffer(
            info.width,
            info.height,
            &buf[..info.buffer_size()],
        ));

        Self {
            tiled: false,
            raster,
        }
    }

    pub fn tiled(mut self, tiled: bool) -> Self {
        self.tiled = tiled;

        self
    }

    fn tile_rows(&self, start: i32, end: i32) -> impl Iterator<Item = i32> {
        let step = self.raster.width().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };

        assert!(current <= end);

        TileIter { current, step, end }
    }

    fn tile_cols(&self, start: i32, end: i32) -> impl Iterator<Item = i32> {
        let step = self.raster.height().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };

        assert!(current <= end);

        TileIter { current, step, end }
    }
}

impl Iterator for TileIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let last = self.current;
        self.current += self.step;

        if last <= self.end {
            Some(last)
        } else {
            None
        }
    }
}

impl BitmapCache {
    pub fn get_or_create(&mut self, key: &str, asset_server: &Res<AssetServer>) -> Bitmap {
        self.map
            .entry(key.to_string())
            .or_insert_with(|| {
                let io = asset_server
                    .asset_io()
                    .downcast_ref::<EmbeddedAssetIo>()
                    .unwrap();

                // TODO: This should probably return the Result.
                let image = io.load_path_sync(Path::new(key)).unwrap();

                Bitmap::new(&image)
            })
            .clone()
    }
}

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::update);
    }
}

impl FadePlugin {
    fn update(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Bitmap, &mut Fade)>,
        time: Res<Time>,
    ) {
        for (entity, mut bitmap, mut fade) in query.iter_mut() {
            if fade.timer.tick(time.delta()).finished() {
                commands.entity(entity).despawn_recursive();
            } else {
                let mut color = fade.base_color;

                // Apply the fade to the color (pre-multiplied alpha).
                let alpha =
                    Ch8::from(fade.from).lerp(Ch8::from(fade.to), Ch8::from(fade.timer.percent()));
                for chan in color.channels_mut() {
                    *chan = *chan * alpha;
                }

                // Force-update the bitmap raster.
                bitmap.raster = Arc::new(Raster::with_color(
                    bitmap.raster.width(),
                    bitmap.raster.height(),
                    color,
                ));
            }
        }
    }
}

impl Fade {
    pub fn fade_in(
        time_seconds: f32,
        width: u32,
        height: u32,
        base_color: Rgba8p,
    ) -> (Bitmap, Self, TransformBundle) {
        let raster = Arc::new(Raster::with_color(width, height, base_color));
        let bitmap = Bitmap {
            tiled: false,
            raster,
        };

        let timer = Timer::from_seconds(time_seconds, false);
        let from = 1.0;
        let to = 0.0;
        let fade = Self {
            timer,
            from,
            to,
            base_color,
        };

        let transform = Transform::from_xyz(0.0, 0.0, std::f32::INFINITY);
        let transform_bundle = TransformBundle::from_transform(transform);

        (bitmap, fade, transform_bundle)
    }

    pub fn fade_out(
        time_seconds: f32,
        width: u32,
        height: u32,
        base_color: Rgba8p,
    ) -> (Bitmap, Self, TransformBundle) {
        let raster = Arc::new(Raster::with_clear(width, height));
        let bitmap = Bitmap {
            tiled: false,
            raster,
        };

        let timer = Timer::from_seconds(time_seconds, false);
        let from = 0.0;
        let to = 1.0;
        let fade = Self {
            timer,
            from,
            to,
            base_color,
        };

        let transform = Transform::from_xyz(0.0, 0.0, std::f32::INFINITY);
        let transform_bundle = TransformBundle::from_transform(transform);

        (bitmap, fade, transform_bundle)
    }
}
