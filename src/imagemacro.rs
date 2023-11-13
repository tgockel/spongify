//! Image Macro Generation
//! ======================
//!
//! An ["image macro"](https://en.wikipedia.org/wiki/Image_macro) is what a cultured person calls what the unwashed
//! masses would refer to as a "meme." This module generates image macros for Mocking Spongebob. It could be useful to
//! expand the capabilities of this module into a library all its own, but that would require fewer magic numbers.

use std::{cell::RefCell, collections::HashMap};
use bytes::Bytes;
use fontdue::{Font, layout::{self, Layout}};
use image::Pixel;

const ANTON_REGULAR_SOURCE: &'static [u8] = include_bytes!("Anton-Regular.ttf");
const MOCKING_SPONGEBOB_SOURCE: &'static [u8] = include_bytes!("mocking-spongebob.jpg");

type Color = image::Rgba<u8>;

struct GlyphGenerator<'a> {
    font: &'a Font,
    cache: RefCell<HashMap<layout::GlyphRasterConfig, (fontdue::Metrics, Bytes)>>,
}

impl<'a> GlyphGenerator<'a> {
    pub fn with_capacity(font: &'a Font, capacity: usize) -> Self {
        Self {
            font,
            cache: RefCell::new(HashMap::with_capacity(capacity))
        }
    }

    /// Get the glyph named `key`.
    ///
    /// # Return
    /// The returned value is a tuple of `Metrics` and data. The `Metrics` covers positioning metadata like initial
    /// position as well as width and height. The data is covering information, where 0 represents no coverage and 255
    /// represents full coverage.
    pub fn glyph(&self, key: layout::GlyphRasterConfig) -> (fontdue::Metrics, Bytes) {
        self.cache
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| {
                let (metrics, coverage) = self.font.rasterize_config(key);
                let coverage = Bytes::from(coverage);
                (metrics, coverage)
            })
            .to_owned()
    }
}

#[derive(Clone, Copy, Debug)]
struct SizeDim(u32, u32);

impl SizeDim {
    pub fn width(&self) -> u32 {
        self.0
    }

    pub fn height(&self) -> u32 {
        self.1
    }

    pub fn map_height(&self, f: impl FnOnce(u32) -> u32) -> Self {
        Self(self.width(), f(self.height()))
    }

    pub fn area(&self) -> usize {
        (self.width() as usize) * (self.height() as usize)
    }
}

#[derive(Clone, Copy, Debug)]
struct Vec2<T>(T, T);

impl<T> Vec2<T> where T: Clone + Copy {
    pub fn new(x: T, y: T) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> T {
        self.0
    }

    pub fn y(&self) -> T {
        self.1
    }
}

/// Create an overlay image for the rendered `text`.
fn render_text(
    renderer: &GlyphGenerator,
    layout: &mut Layout,
    font: &Font,
    font_size: f32,
    size: SizeDim,
    text: &str,
) -> image::GrayImage {
    let mut gray_image =
        image::GrayImage::from_vec(size.width(), size.height(), vec![0; size.area()]).unwrap();

    let glyphs = get_filling_glyphs(size, &font, layout, font_size, text);

    render_glyphs(glyphs, renderer, |x, y, coverage| {
        gray_image.put_pixel(x, y, image::Luma([coverage]));
    });

    gray_image
}

fn get_filling_glyphs<'a>(
    size: SizeDim,
    font: &Font,
    layout: &'a mut Layout,
    font_size: f32,
    text: &str,
) -> &'a [layout::GlyphPosition] {
    let max_width = size.width() as f32;
    let max_height = size.height() as f32;

    layout.reset(&layout::LayoutSettings {
        max_height: Some(max_height),
        max_width: Some(max_width),
        horizontal_align: layout::HorizontalAlign::Center,
        vertical_align: layout::VerticalAlign::Top,
        wrap_style: layout::WrapStyle::Word,
        wrap_hard_breaks: true,
        ..Default::default()
    });
    layout.append(
        &[font],
        &layout::TextStyle {
            text,
            px: font_size,
            font_index: 0,
            user_data: (),
        },
    );

    layout.glyphs()
}

fn render_glyphs(
    glyphs: &[layout::GlyphPosition],
    renderer: &GlyphGenerator,
    mut put_pixel: impl FnMut(u32, u32, u8),
) {
    for glyph in glyphs.iter().filter(|x| !x.char_data.is_control()) {
        let (ref metrics, ref bytes) = renderer.glyph(glyph.key);

        for x in 0..metrics.width {
            for y in 0..metrics.height {
                let coverage = bytes[x + y * metrics.width];
                let x = x as u32 + glyph.x as u32;
                let y = y as u32 + glyph.y as u32;
                put_pixel(x, y, coverage);
            }
        }
    }
}

fn merge_image(
    image: &mut image::RgbaImage,
    mask: &image::GrayImage,
    color: Color,
    base_position: Vec2<u32>
) {
    for mask_x in 0..mask.width() {
        for mask_y in 0..mask.height() {
            let x = base_position.x() + mask_x;
            let y = base_position.y() + mask_y;

            if (0..image.width()).contains(&x) && (0..image.height()).contains(&y) {
                let mask = mask.get_pixel(mask_x, mask_y).0[0];
                let mask = Color::from([color.0[0], color.0[1], color.0[2], mask]);

                let mut pixel = image.get_pixel(x, y).clone();
                pixel.blend(&mask);

                image.put_pixel(x, y, pixel);
            }
        }
    }
}

pub fn generate_image(
    top_text: Option<&str>,
    bottom_text: Option<&str>,
) -> image::RgbaImage {
    let mut image = image::load_from_memory_with_format(MOCKING_SPONGEBOB_SOURCE, image::ImageFormat::Jpeg)
        .expect("Failed to load built-in image")
        .into_rgba8();

    let font = fontdue::Font::from_bytes(ANTON_REGULAR_SOURCE, fontdue::FontSettings::default())
        .expect("Failed to load built-in font");
    let mut font_layout = fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);

    let rasterer = GlyphGenerator::with_capacity(&font, 1024);

    let font_size = image.height() as f32 / 8.;
    let size = SizeDim(image.width(), image.height());
    let text_color = Color::from([255, 255, 255, 255]);

    if let Some(text) = top_text {
        let mask = render_text(
            &rasterer,
            &mut font_layout,
            &font,
            font_size,
            size.map_height(|h| h / 4),
            text,
        );

        merge_image(
            &mut image,
            &mask,
            text_color,
            Vec2::new(0, 0),
        );
    }

    if let Some(text) = bottom_text {
        let mask = render_text(
            &rasterer,
            &mut font_layout,
            &font,
            font_size,
            size.map_height(|h| h / 4),
            text,
        );

        let text_y = image.height() - font_layout.height() as u32;
        merge_image(
            &mut image,
            &mask,
            text_color,
            Vec2::new(0, text_y),
        );
    }

    image
}
