use glam::{UVec2, Vec2};

use crate::{
    color::Color, font::Text, handle::Handle, platform::Platform, sprite::Sprite,
    text_cache::TextCache, types::Rect,
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ScaleMode {
    #[default]
    None,
    Discrete,
    Exact,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct ResizeMode {
    pub width: bool,
    pub height: bool,
}

/// Render subsystem
pub(crate) struct Render {
    draw_calls: u32,
    screen_scale: f32,
    inv_screen_scale: f32,
    pub(crate) screen_size: Vec2,
    logical_size: Vec2,
    scale_mode: ScaleMode,
    resize_mode: ResizeMode,
    view_size: Vec2,
    pub(crate) platform: Box<dyn Platform + 'static>,
    pub(crate) default_font: Option<Handle>,
}

impl Render {
    pub(crate) fn new(platform: Box<dyn Platform + 'static>) -> Self {
        Self {
            draw_calls: 0,
            screen_scale: 1.0,
            inv_screen_scale: 1.0,
            screen_size: Vec2::default(),
            logical_size: Vec2::default(),
            scale_mode: ScaleMode::default(),
            view_size: Vec2::new(1280.0, 720.0),
            resize_mode: ResizeMode::default(),
            platform,
            default_font: None,
        }
    }

    pub(crate) fn set_default_font(&mut self, handle: Handle) {
        self.default_font.replace(handle);
    }

    pub(crate) fn snap_px(&self, pos: Vec2) -> Vec2 {
        let sp = pos * self.screen_scale;
        sp * self.inv_screen_scale
    }

    pub(crate) fn draw(
        &mut self,
        handle: &Handle,
        color: Color,
        src: Option<Rect>,
        dst: Rect,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    ) {
        if dst.min.x > self.logical_size.x
            || dst.min.y > self.logical_size.y
            || dst.max.x < 0.
            || dst.max.y < 0.
        {
            return;
        }

        // screen scale
        let dst = Rect {
            min: dst.min * self.screen_scale,
            max: dst.max * self.screen_scale,
        };

        self.draw_calls += 1;

        self.platform
            .draw(handle, color, src, dst, angle, flip_x, flip_y);
    }

    /// Draw image
    pub fn draw_image(
        &mut self,
        image: &Sprite,
        pos: Vec2,
        scale: Option<Vec2>,
        angle: Option<f32>,
    ) {
        let size = scale
            .map(|s| image.sizef() * s)
            .unwrap_or_else(|| image.sizef());

        let dst = Rect {
            min: pos - size * image.anchor,
            max: pos + size * (Vec2::splat(1.0) - image.anchor),
        };
        self.draw(
            &image.texture,
            image.color,
            None,
            dst,
            angle,
            image.flip_x,
            image.flip_y,
        );
    }

    /// Draw image as tile
    pub fn draw_tile(
        &mut self,
        image: &Sprite,
        tile: u16,
        tile_size: Vec2,
        dst_pos: Vec2,
        scale: Option<Vec2>,
        angle: Option<f32>,
        flip_x: bool,
        flip_y: bool,
    ) {
        let cols =
            ((image.size().x as f32 - image.padding) / (tile_size.x + image.spacing)).ceil() as u32;
        let row = tile as u32 / cols;
        let col = tile as u32 % cols;
        let src_pos = Vec2::new(
            col as f32 * (tile_size.x + image.spacing) + image.padding,
            row as f32 * (tile_size.y + image.spacing) + image.padding,
        );
        let src_size = Vec2::new(tile_size.x, tile_size.y);
        let half_dst_size = scale.map(|s| s * src_size).unwrap_or_else(|| src_size) * 0.5;

        // color
        let flip_x = flip_x || image.flip_x;
        let flip_y = flip_y || image.flip_y;

        // Fix texture bleeding by offset half pixel on source
        // see https://github.com/jjyr/roast2d/issues/6 for details
        // NOTICE the offset is 1 pixel instead of 0.5 pixel on some backends
        let src = Rect {
            min: src_pos + Vec2::splat(0.5),
            max: src_pos + src_size - Vec2::splat(0.5),
        };
        let dst = Rect {
            min: dst_pos - half_dst_size,
            max: dst_pos + half_dst_size,
        };
        self.draw(
            &image.texture,
            image.color,
            Some(src),
            dst,
            angle,
            flip_x,
            flip_y,
        );
    }

    pub(crate) fn resize(&mut self, size: UVec2) {
        // Determine Zoom
        if self.scale_mode == ScaleMode::None {
            self.screen_scale = 1.0;
        } else {
            self.screen_scale =
                (size.x as f32 / self.view_size.x).min(size.y as f32 / self.view_size.y);
            if self.scale_mode == ScaleMode::Discrete {
                self.screen_scale = self.screen_scale.floor().max(0.5);
            }
        }
        // Determine size
        if self.resize_mode.width {
            self.screen_size.x = (size.x as f32).max(self.view_size.x);
        } else {
            self.screen_size.x = self.view_size.x * self.screen_scale;
        }

        if self.resize_mode.height {
            self.screen_size.y = (size.y as f32).max(self.view_size.y);
        } else {
            self.screen_size.y = self.view_size.y * self.screen_scale;
        }

        self.logical_size.x = (self.screen_size.x / self.screen_scale).ceil();
        self.logical_size.y = (self.screen_size.y / self.screen_scale).ceil();
        self.inv_screen_scale = 1.0 / self.screen_scale;
    }

    pub(crate) fn create_text_texture(
        &mut self,
        text_cache: &mut TextCache,
        handle: Handle,
        text: &Text,
    ) -> UVec2 {
        let Text {
            text,
            font,
            scale,
            color,
        } = text;
        let font = text_cache
            .get_font(
                font.as_ref()
                    .or(self.default_font.as_ref())
                    .expect("no default font")
                    .id(),
            )
            .expect("can't find font by handle id");
        let buffer = font.render_text_texture(text, *scale, *color);
        let width = buffer.width();
        let height = buffer.height();
        let size = UVec2::new(width, height);
        self.platform
            .create_texture(handle, buffer.into_vec(), size);
        size
    }

    pub(crate) fn scale_mode(&self) -> ScaleMode {
        self.scale_mode
    }

    pub(crate) fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
    }

    pub(crate) fn resize_mode(&self) -> ResizeMode {
        self.resize_mode
    }

    pub(crate) fn set_resize_mode(&mut self, mode: ResizeMode) {
        self.resize_mode = mode;
    }

    pub(crate) fn view_size(&self) -> Vec2 {
        self.view_size
    }

    pub(crate) fn logical_size(&self) -> Vec2 {
        self.logical_size
    }

    pub(crate) fn set_view_size(&mut self, size: Vec2) {
        self.view_size = size;
    }
}
