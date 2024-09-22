use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use image::{DynamicImage, ImageBuffer, Rgba};
use rusttype::{point, Scale};

use crate::color::Color;

#[derive(Clone)]
pub struct Font {
    inner: rusttype::Font<'static>,
}

impl Font {
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let inner = rusttype::Font::try_from_vec(bytes)?;
        Some(Self { inner })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = fs::read(path)?;
        let font = Self::from_bytes(bytes).ok_or_else(|| anyhow!("Invalid font"))?;
        Ok(font)
    }

    pub(crate) fn render_text_texture(
        &self,
        text: &str,
        scale: f32,
        color: Color,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        // layout the glyphs in a line with 20 pixels padding
        let scale = Scale::uniform(scale);
        let v_metrics = self.inner.v_metrics(scale);
        let glyphs: Vec<_> = self
            .inner
            .layout(text, scale, point(20.0, 20.0 + v_metrics.ascent))
            .collect();
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
        let glyphs_width = {
            let min_x = glyphs
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap_or_default();
            let max_x = glyphs
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap_or_default();
            (max_x - min_x) as u32
        };
        let mut image = DynamicImage::new_rgba8(glyphs_width + 40, glyphs_height + 40).to_rgba8();

        // Loop through the glyphs in the text, positing each one on a line
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, v| {
                    image.put_pixel(
                        // Offset the position by the glyph bounding box
                        x + bounding_box.min.x as u32,
                        y + bounding_box.min.y as u32,
                        // Turn the coverage into an alpha value
                        Rgba([color.r, color.g, color.b, (v * 255.0) as u8]),
                    )
                });
            }
        }
        image
    }
}

pub struct Text {
    pub text: String,
    pub font: Font,
    pub scale: f32,
    pub color: Color,
}

impl Text {
    pub fn new(text: String, font: Font, scale: f32, color: Color) -> Self {
        Self {
            text,
            font,
            scale,
            color,
        }
    }
}
