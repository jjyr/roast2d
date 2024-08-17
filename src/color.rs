#[derive(Debug, Default, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 0xff)
    }
}

pub const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);
pub const BLACK: Color = Color::rgb(0, 0, 0);
pub const BLUE: Color = Color::rgb(0, 0, 0xff);
pub const GRAY: Color = Color::rgb(0x80, 0x80, 0x80);
pub const GREEN: Color = Color::rgb(0, 0x80, 0);
pub const PURPLE: Color = Color::rgb(0x80, 0, 0x80);
pub const RED: Color = Color::rgb(0xff, 0, 0);
pub const SILVER: Color = Color::rgb(0xC0, 0xC0, 0xC0);
pub const YELLOW: Color = Color::rgb(0xff, 0xff, 0);
