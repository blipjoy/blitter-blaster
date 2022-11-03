use bevy::prelude::*;
use pix::{rgb::Rgba8p, Raster};
use std::io::Cursor;

#[derive(Component)]
pub struct Bitmap {
    pub pos: (i32, i32),
    pub raster: Raster<Rgba8p>,
}

impl Bitmap {
    pub fn new(pos: (i32, i32), bytes: &[u8]) -> Self {
        let decoder = png::Decoder::new(Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let raster = Raster::with_u8_buffer(info.width, info.height, &buf[..info.buffer_size()]);

        Self { pos, raster }
    }
}
