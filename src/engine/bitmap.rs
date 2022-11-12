use crate::engine::{
    camera::{Camera, ScreenSpace},
    collision::BvhResource,
};
use ahash::{HashSet, HashSetExt as _, RandomState};
use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetIo;
use bevy_pixels::prelude::*;
use bvh_arena::volumes::Aabb;
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
    end: i32,
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

impl Camera {
    fn to_aabb(&self) -> Aabb<2> {
        let pos = self.transform().translation.truncate();
        let size = *self.size();

        Aabb::from_min_max(pos, pos + size)
    }
}

type DrawableBitmap<'a> = (
    Entity,
    &'a Bitmap,
    &'a Transform,
    Option<&'a Tiled>,
    Option<&'a ScreenSpace>,
);

impl BitmapPlugin {
    /// Rasterizes all [`Bitmap`]s in the world.
    ///
    /// Each [`Bitmap`] requires a [`Transform`] (to position it), and may optionally include a
    /// [`ScreenSpace`] component to control whether the position is affected by the viewport
    /// position. The [`Camera`] resource provides the viewport.
    fn update(
        mut pixels_res: ResMut<PixelsResource>,
        mut camera: ResMut<Camera>,
        bvh: Res<BvhResource>,
        bitmaps: Query<DrawableBitmap>,
        tiled_bitmaps: Query<DrawableBitmap, With<Tiled>>,
        screen_bitmaps: Query<DrawableBitmap, With<ScreenSpace>>,
    ) {
        let camera_transform = camera.transform();
        let camera_aabb = camera.to_aabb();
        let camera_raster = camera.raster_mut();

        // Clear the camera.
        camera_raster.clear();

        // Use a HashSet to de-dupe entities.
        let mut entities = HashSet::new();

        // Find all Bitmap entities that are within the viewport.
        bvh.for_each_overlaps(&camera_aabb, |&entity| {
            entities.insert(entity);
        });

        // Tiled and screen-space bitmaps are always selected.
        entities.extend(tiled_bitmaps.into_iter().map(|query| query.0));
        entities.extend(screen_bitmaps.into_iter().map(|query| query.0));

        // Sort by Z coordinate
        let mut bitmaps: Vec<_> = entities
            .into_iter()
            .map(|entity| bitmaps.get(entity).unwrap())
            .collect();
        bitmaps.sort_unstable_by_key(|query| (query.2.translation.z * 1000.0) as i64);

        // Composite each bitmap to the camera.
        for (_, bitmap, transform, tiled, screen_space) in bitmaps {
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
                let width = camera_raster.width();
                let height = camera_raster.height();

                // Iterate over all ranges required to fill the frame with the bitmap.
                for x in bitmap.tile_cols(x, width) {
                    for y in bitmap.tile_rows(y, height) {
                        camera_raster.composite_raster((x, y), &bitmap.raster, (), SrcOver);
                    }
                }
            } else {
                camera_raster.composite_raster((x, y), &bitmap.raster, (), SrcOver);
            }
        }

        // Copy the camera to `Pixels`.
        pixels_res
            .pixels
            .get_frame_mut()
            .copy_from_slice(camera_raster.as_u8_slice());
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

        Self { raster }
    }

    pub fn with_clear(width: u32, height: u32) -> Self {
        let raster = Arc::new(Raster::with_clear(width, height));

        Self { raster }
    }

    pub fn with_color(width: u32, height: u32, color: Rgba8p) -> Self {
        let raster = Arc::new(Raster::with_color(width, height, color));

        Self { raster }
    }

    pub fn clear(&mut self, color: Rgba8p) {
        self.raster = Arc::new(Raster::with_color(self.width(), self.height(), color));
    }

    pub fn width(&self) -> u32 {
        self.raster.width()
    }

    pub fn height(&self) -> u32 {
        self.raster.height()
    }

    fn tile_rows(&self, start: i32, height: u32) -> impl Iterator<Item = i32> {
        let step = self.height().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };
        let end = height.try_into().unwrap();

        TileIter { current, step, end }
    }

    fn tile_cols(&self, start: i32, width: u32) -> impl Iterator<Item = i32> {
        let step = self.width().try_into().unwrap();
        let current = start % step;
        let current = if current > 0 { current - step } else { current };
        let end = width.try_into().unwrap();

        TileIter { current, step, end }
    }
}

impl Iterator for TileIter {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let last = self.current;
            self.current += self.step;

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
