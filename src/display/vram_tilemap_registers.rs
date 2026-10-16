use crate::cpu::cpu::CPU;
use cli_log::*;
use image::error::UnsupportedError;
use image::{DynamicImage, RgbaImage};
use palette::Srgb;
use ratatui::style::{Color, Stylize};
use ratatui::{
    Frame,
    layout::{Constraint, Rect, Size},
    style::Style,
    text::Line,
    widgets::{Block, Cell, Padding, Paragraph, Row, Table},
};
use ratatui_image::{Image, Resize, picker::Picker};

pub fn render(frame: &mut Frame, area: Rect, tilemap: &[u8], tiles: &[u8]) {
    let block = Block::bordered()
        .title(Line::from("VRAM-Tiles"))
        .padding(Padding {
            left: 1,
            right: 0,
            top: 1,
            bottom: 0,
        })
        .style(Style::new().light_magenta());

    let inner_area = block.inner(area);

    let dyn_img = generate_buffer(tilemap, tiles);
    let size = Size::new(inner_area.width, inner_area.height);

    let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
    let image = picker
        .new_protocol(dyn_img, size, Resize::Scale(None))
        .unwrap();

    // for each pair of bytes
    // combine each bit each other -> get a color -> put it in the array
    // draw a 8x8 square based on this color
    let image = Image::new(&image);

    frame.render_widget(block, area);
    frame.render_widget(image, inner_area);
}

const COLORS: [Srgb; 4] = [
    Srgb::new(1.0, 1.0, 1.0),
    Srgb::new(0.83, 0.83, 0.83),
    Srgb::new(0.5, 0.5, 0.5),
    Srgb::new(0.0, 0.0, 0.0),
];

fn generate_buffer(tilemap: &[u8], tiles: &[u8]) -> DynamicImage {
    const TILES_PER_ROW: usize = 32;
    let width_px = TILES_PER_ROW * 8;
    let height_px = TILES_PER_ROW * 8;

    let mut raw_pixels: Vec<u8> = Vec::with_capacity(width_px * height_px * 4);
    for y in 0..height_px {
        for x in 0..width_px {
            let tile_idx = (y / 8) * TILES_PER_ROW + (x / 8);
            let tile_id = tilemap[tile_idx];
            // get tile from ID
            let start_tile = tile_id as usize * 16;

            let row = y % 8; // which line inside the tile

            let lo = tiles[start_tile + row * 2];
            let hi = tiles[start_tile + row * 2 + 1];

            let pixel_idx_x = 7 - (x % 8);
            let color_idx: u8 = (((hi >> pixel_idx_x) & 1) << 1) | ((lo >> pixel_idx_x) & 1);
            let color: Srgb<u8> = COLORS[color_idx as usize].into_format();

            let a = 255u8;
            raw_pixels.push(color.red);
            raw_pixels.push(color.green);
            raw_pixels.push(color.blue);
            raw_pixels.push(a);
        }
    }
    let img_buf = RgbaImage::from_raw(width_px as u32, height_px as u32, raw_pixels).unwrap();
    DynamicImage::ImageRgba8(img_buf)
}
