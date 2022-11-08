use bevy::prelude::*;
use bevy_pixels::prelude::*;
use pix::{ops::SrcOver, rgb::Rgba8p, Raster};
use std::io::Cursor;

#[derive(Debug)]
pub struct BitmapPlugin;

#[derive(Component)]
pub struct Bitmap {
    tiled: bool,
    raster: Raster<Rgba8p>,
}

struct TileIter {
    current: i32,
    step: i32,
    end: i32,
}

impl Plugin for BitmapPlugin {
    fn build(&self, app: &mut App) {
        let PixelsOptions { width, height } = *app.world.resource::<PixelsOptions>();

        app.insert_resource(Raster::<Rgba8p>::with_clear(width, height))
            .add_system_to_stage(PixelsStage::Draw, blit);
    }
}

impl Bitmap {
    pub fn new(bytes: &[u8]) -> Self {
        let decoder = png::Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let raster = Raster::with_u8_buffer(info.width, info.height, &buf[..info.buffer_size()]);

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

fn blit(
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
