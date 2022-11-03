use bevy::prelude::*;
use bevy_pixels::prelude::*;
use pix::{ops::SrcOver, rgb::Rgba8p, Raster};
use std::io::Cursor;

pub struct BitmapPlugin;

#[derive(Component)]
pub struct Bitmap(Raster<Rgba8p>);

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

        Self(raster)
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
        let to = (
            transform.translation.x as i32,
            transform.translation.y as i32,
        );

        raster.composite_raster(to, &bitmap.0, (0, 0), SrcOver);
    }

    pixels_res
        .pixels
        .get_frame_mut()
        .copy_from_slice(raster.as_u8_slice());
}
