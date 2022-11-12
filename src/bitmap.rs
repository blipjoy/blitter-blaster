use crate::camera::{Camera, ScreenSpace};
use ahash::RandomState;
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetIo;
use bevy_pixels::prelude::*;
use pix::{ops::SrcOver, rgb::Rgba8p, Raster};
use std::{collections::HashMap, io::Cursor, path::Path, sync::Arc};

#[derive(Debug)]
pub struct BitmapPlugin;

#[derive(Clone, Component)]
pub struct Bitmap {
    raster: Arc<Raster<Rgba8p>>,
}

/// Adding this component to a `Bitmap` will cause it to be treated as an infinitely tiled
/// (repeated) background.
#[derive(Component, Debug)]
pub struct Tiled;

#[derive(Debug)]
struct TileIter {
    current: i32,
    step: i32,
}

#[derive(Default)]
pub struct BitmapCache {
    map: HashMap<String, Bitmap, RandomState>,
}

impl Plugin for BitmapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BitmapCache>()
            .add_system_to_stage(PixelsStage::Draw, Self::update);
    }
}

impl BitmapPlugin {
    /// Rasterizes all [`Bitmap`]s in the world.
    ///
    /// Each [`Bitmap`] requires a [`Transform`] (to position it), and may optionally include a
    /// [`ScreenSpace`] component to control whether the position is affected by the viewport
    /// position. The [`Camera`] resource provides the viewport.
    fn update(
        mut pixels_res: ResMut<PixelsResource>,
        mut camera: ResMut<Camera>,
        query: Query<(&Bitmap, &Transform, Option<&Tiled>, Option<&ScreenSpace>)>,
    ) {
        let camera_transform = camera.transform();
        let raster = camera.raster_mut();
        raster.clear();

        // Sort by Z coordinate
        let mut bitmaps: Vec<_> = query.iter().collect();
        bitmaps.sort_unstable_by_key(|(_, t, _, _)| (t.translation.z * 1000.0) as i64);

        for (bitmap, transform, tiled, screen_space) in bitmaps {
            let (x, y) = if screen_space.is_some() {
                // In screen space, the destination region is relative to the origin.
                let translation = transform.translation;

                (translation.x as i32, translation.y as i32)
            } else {
                // In world space, the destination region is relative to the camera viewport.
                let z = transform.translation.z;
                let z = if z.is_finite() { z } else { 1.0 };
                let camera_translation = transform.translation - camera_transform.translation * z;

                (camera_translation.x as i32, camera_translation.y as i32)
            };

            if tiled.is_some() {
                // Iterate over all ranges required to fill the frame with the bitmap.
                for y in bitmap.tile_cols(y) {
                    for x in bitmap.tile_rows(x) {
                        raster.composite_raster((x, y), &bitmap.raster, (), SrcOver);
                    }
                }
            } else {
                raster.composite_raster((x, y), &bitmap.raster, (), SrcOver);
            }
        }

        pixels_res
            .pixels
            .get_frame_mut()
            .copy_from_slice(raster.as_u8_slice());
    }
}

impl Bitmap {
    pub fn clear(width: u32, height: u32) -> Self {
        let raster = Arc::new(Raster::with_clear(width, height));

        Self { raster }
    }

    pub fn clear_color(width: u32, height: u32, color: Rgba8p) -> Self {
        let raster = Arc::new(Raster::with_color(width, height, color));

        Self { raster }
    }

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

        Self { raster }
    }

    fn tile_rows(&self, start: i32) -> impl Iterator<Item = i32> {
        let step = self.raster.height().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };

        TileIter { current, step }
    }

    fn tile_cols(&self, start: i32) -> impl Iterator<Item = i32> {
        let step = self.raster.width().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };

        TileIter { current, step }
    }

    pub fn raster_mut(&mut self) -> &mut Arc<Raster<Rgba8p>> {
        &mut self.raster
    }
}

impl Iterator for TileIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let last = self.current;
        self.current += self.step;

        if last < self.step {
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
