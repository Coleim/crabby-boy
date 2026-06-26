use image::{DynamicImage, RgbaImage};
use ratatui::Frame;
use ratatui::layout::Size;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Padding},
};
use ratatui_image::{Image, Resize, picker::Picker};

const WIDTH: u32 = 160;
const HEIGHT: u32 = 144;

pub fn render(frame: &mut Frame, area: Rect) {
    // TODO: Replace with real PPU buffer
    let dyn_img = generate_buffer();

    let block = Block::bordered()
        .title(Line::from("Game Playing..."))
        .padding(Padding {
            left: 1,
            right: 0,
            top: 1,
            bottom: 0,
        })
        .style(Style::new().green());

    let inner_area = block.inner(area);

    let size = Size::new(inner_area.width, inner_area.height);

    let picker = Picker::from_query_stdio().expect("impossible de détecter le terminal");
    let image = picker
        .new_protocol(dyn_img, size, Resize::Fit(None))
        .unwrap();

    let image = Image::new(&image);

    frame.render_widget(block, area);
    frame.render_widget(image, inner_area);
}

/// Build a raw RGBA pixel buffer by hand and wrap it in a DynamicImage.
fn generate_buffer() -> DynamicImage {
    let mut raw_pixels: Vec<u8> = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let r = ((x as f32 / WIDTH as f32) * 255.0) as u8;
            let g = ((y as f32 / HEIGHT as f32) * 255.0) as u8;
            let b = 128u8;
            let a = 255u8;
            raw_pixels.push(r);
            raw_pixels.push(g);
            raw_pixels.push(b);
            raw_pixels.push(a);
        }
    }
    let img_buf = RgbaImage::from_raw(WIDTH, HEIGHT, raw_pixels)
        .expect("la taille du buffer doit être width*height*4");
    DynamicImage::ImageRgba8(img_buf)
}
