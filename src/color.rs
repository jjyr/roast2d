#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

    pub fn with_a(mut self, a: u8) -> Self {
        self.a = a;
        self
    }
}

pub const WHITE: Color = Color::rgb(0xff, 0xff, 0xff);
pub const BLACK: Color = Color::rgb(0, 0, 0);
pub const BLUE: Color = Color::rgb(0, 0, 0xff);
pub const GRAY: Color = Color::rgb(0x80, 0x80, 0x80);
pub const GREEN: Color = Color::rgb(0, 0x80, 0);
pub const PURPLE: Color = Color::rgb(0x80, 0, 0x80);
pub const RED: Color = Color::rgb(0xff, 0, 0);
pub const SILVER: Color = Color::rgb(0xc0, 0xc0, 0xc0);
pub const YELLOW: Color = Color::rgb(0xff, 0xff, 0);
pub const NAVY: Color = Color::rgb(0x00, 0x00, 0x80);
pub const DARK_BLUE: Color = Color::rgb(0x00, 0x00, 0x8b);
pub const MEDIUM_BLUE: Color = Color::rgb(0x00, 0x00, 0xcd);
pub const DARK_GREEN: Color = Color::rgb(0x00, 0x64, 0x00);
pub const TEAL: Color = Color::rgb(0x00, 0x80, 0x80);
pub const DARK_CYAN: Color = Color::rgb(0x00, 0x8B, 0x8B);
pub const DEEP_SKY_BLUE: Color = Color::rgb(0x00, 0xBF, 0xFF);
pub const DARK_TURQUOISE: Color = Color::rgb(0x00, 0xce, 0xd1);
pub const MEDIUM_SPRING_GREEN: Color = Color::rgb(0x00, 0xfa, 0x9a);
pub const LIME: Color = Color::rgb(0x00, 0xff, 0x00);
pub const SPRING_GREEN: Color = Color::rgb(0x00, 0xff, 0x7f);
pub const AQUA: Color = Color::rgb(0x00, 0xff, 0xff);
pub const MIDNIGHT_BLUE: Color = Color::rgb(0x19, 0x19, 0x70);
pub const DODGER_BLUE: Color = Color::rgb(0x1e, 0x90, 0xff);
pub const LIGHT_SEA_GREEN: Color = Color::rgb(0x20, 0xb2, 0xaa);
pub const FOREST_GREEN: Color = Color::rgb(0x22, 0x8b, 0x22);
pub const SEA_GREEN: Color = Color::rgb(0x2e, 0x8b, 0x57);
pub const DARK_SLATE_GRAY: Color = Color::rgb(0x2f, 0x4f, 0x4f);
pub const LIME_GREEN: Color = Color::rgb(0x32, 0xcd, 0x32);
pub const MEDIUM_SEA_GREEN: Color = Color::rgb(0x3c, 0xb3, 0x71);
pub const TURQUOISE: Color = Color::rgb(0x40, 0xe0, 0xd0);
pub const ROYAL_BLUE: Color = Color::rgb(0x41, 0x69, 0xe1);
pub const STEEL_BLUE: Color = Color::rgb(0x46, 0x82, 0xb4);
pub const DARK_SLATE_BLUE: Color = Color::rgb(0x48, 0x3d, 0x8b);
pub const MEDIUM_TORQUOISE: Color = Color::rgb(0x48, 0xd1, 0xcc);
pub const INDIGO: Color = Color::rgb(0x4b, 0x00, 0x82);
pub const DARK_OLIVE_GREEN: Color = Color::rgb(0x55, 0x6b, 0x2f);
pub const CADET_BLUE: Color = Color::rgb(0x5f, 0x9e, 0xa0);
pub const CORN_FLOWER_BLUE: Color = Color::rgb(0x64, 0x95, 0xed);
pub const REBECCA_PURPLE: Color = Color::rgb(0x66, 0x33, 0x99);
pub const MEDIUM_AQUA_MARINE: Color = Color::rgb(0x66, 0xcd, 0xaa);
pub const DIM_GRAY: Color = Color::rgb(0x69, 0x69, 0x69);
pub const SLATE_BLUE: Color = Color::rgb(0x6a, 0x5a, 0xcd);
pub const OLIVE_DRAB: Color = Color::rgb(0x6b, 0x8e, 0x23);
pub const SLATE_GRAY: Color = Color::rgb(0x70, 0x80, 0x90);
pub const LIGHT_SLATE_GRAY: Color = Color::rgb(0x77, 0x88, 0x99);
pub const MEDIUM_SLATE_BLUE: Color = Color::rgb(0x7b, 0x68, 0xee);
pub const LAWN_GREEN: Color = Color::rgb(0x7c, 0xfc, 0x00);
pub const CHARTREUSE: Color = Color::rgb(0x7f, 0xff, 0x00);
pub const AUQAMARINE: Color = Color::rgb(0x7f, 0xff, 0xd4);
pub const MAROON: Color = Color::rgb(0x80, 0x00, 0x00);
pub const OLIVE: Color = Color::rgb(0x80, 0x80, 0x00);
pub const SKY_BLUE: Color = Color::rgb(0x87, 0xce, 0xeb);
pub const LIGHT_SKY_BLUE: Color = Color::rgb(0x87, 0xce, 0xfa);
pub const BLUE_VIOLET: Color = Color::rgb(0x8a, 0x2b, 0xe2);
pub const DARK_RED: Color = Color::rgb(0x8b, 0x00, 0x00);
pub const DARK_MAGENTA: Color = Color::rgb(0x8b, 0x00, 0x8b);
pub const SADDLE_BROWN: Color = Color::rgb(0x8b, 0x45, 0x13);
pub const DARK_SEA_GREEN: Color = Color::rgb(0x8f, 0xbc, 0x8f);
pub const LIGHT_GREEN: Color = Color::rgb(0x90, 0xee, 0x90);
pub const MEDIUM_PURPLE: Color = Color::rgb(0x93, 0x70, 0xdb);
pub const DARK_VIOLET: Color = Color::rgb(0x94, 0x00, 0xd3);
pub const PALE_GREEN: Color = Color::rgb(0x98, 0xfb, 0x98);
pub const DARK_ORCHID: Color = Color::rgb(0x99, 0x32, 0xcc);
pub const YELLOW_GREEN: Color = Color::rgb(0x9a, 0xcd, 0x32);
pub const SIENNA: Color = Color::rgb(0xa0, 0x52, 0x2d);
pub const BROWN: Color = Color::rgb(0xa5, 0x2a, 0x2a);
pub const DARK_GRAY: Color = Color::rgb(0xa9, 0xa9, 0xa9);
pub const LIGHT_BLUE: Color = Color::rgb(0xad, 0xd8, 0xe6);
pub const PALE_TURQUOISE: Color = Color::rgb(0xaf, 0xee, 0xee);
pub const FIRE_BRICK: Color = Color::rgb(0xb2, 0x22, 0x22);
pub const DARK_GOLDEN_ROD: Color = Color::rgb(0xb8, 0x86, 0x0b);
pub const MEDIUM_ORCHID: Color = Color::rgb(0xba, 0x55, 0xd3);
pub const ROSY_BROWN: Color = Color::rgb(0xbc, 0x8f, 0x8f);
pub const DARK_KHAKI: Color = Color::rgb(0xbd, 0xb7, 0x6b);
pub const MEDIUM_VIOLET_RED: Color = Color::rgb(0xc7, 0x15, 0x85);
pub const INDIAN_RED: Color = Color::rgb(0xcd, 0x5c, 0x5c);
pub const PERU: Color = Color::rgb(0xcd, 0x85, 0x3f);
pub const CHOCOLATE: Color = Color::rgb(0xd2, 0x69, 0x1e);
pub const TAN: Color = Color::rgb(0xd2, 0xb4, 0x8c);
pub const LIGHT_GRAY: Color = Color::rgb(0xd3, 0xd3, 0xd3);
pub const THISTLE: Color = Color::rgb(0xd8, 0xbf, 0xd8);
pub const ORCHID: Color = Color::rgb(0xda, 0x70, 0xd6);
pub const GOLDEN_ROD: Color = Color::rgb(0xda, 0xa5, 0x20);
pub const PALE_VIOLET_RED: Color = Color::rgb(0xdb, 0x70, 0x93);
pub const CRIMSON: Color = Color::rgb(0xdc, 0x14, 0x3c);
pub const GAINSBORO: Color = Color::rgb(0xdc, 0xdc, 0xdc);
pub const PLUM: Color = Color::rgb(0xdd, 0xa0, 0xdd);
pub const BURLY_WOOD: Color = Color::rgb(0xde, 0xb8, 0x87);
pub const LIGHT_CYAN: Color = Color::rgb(0xe0, 0xff, 0xff);
pub const LAVENDER: Color = Color::rgb(0xe6, 0xe6, 0xfa);
pub const DARK_SALMON: Color = Color::rgb(0xe9, 0x96, 0x7a);
pub const VIOLET: Color = Color::rgb(0xee, 0x82, 0xee);
pub const PALE_GOLDEN_ROD: Color = Color::rgb(0xee, 0xe8, 0xaa);
pub const LIGHT_CORAL: Color = Color::rgb(0xf0, 0x80, 0x80);
pub const KHAKI: Color = Color::rgb(0xf0, 0xe6, 0x8c);
pub const ALICE_BLUE: Color = Color::rgb(0xf0, 0xf8, 0xff);
pub const HONEY_DEW: Color = Color::rgb(0xf0, 0xff, 0xf0);
pub const AZURE: Color = Color::rgb(0xf0, 0xff, 0xff);
pub const SANDY_BROWN: Color = Color::rgb(0xf4, 0xa4, 0x60);
pub const WHEAT: Color = Color::rgb(0xf5, 0xde, 0xb3);
pub const BEIGE: Color = Color::rgb(0xf5, 0xf5, 0xdc);
pub const WHITE_SMOKE: Color = Color::rgb(0xf5, 0xf5, 0xf5);
pub const MINT_CREAM: Color = Color::rgb(0xf5, 0xff, 0xfa);
pub const GHOST_WHITE: Color = Color::rgb(0xf8, 0xf8, 0xff);
pub const SALMON: Color = Color::rgb(0xfa, 0x80, 0x72);
pub const ANTIQUE_WHITE: Color = Color::rgb(0xfa, 0xeb, 0xd7);
pub const LINEN: Color = Color::rgb(0xfa, 0xf0, 0xe6);
pub const LIGHT_GOLDEN_ROD_YELLOW: Color = Color::rgb(0xfa, 0xfa, 0xd2);
pub const OLD_LACE: Color = Color::rgb(0xfd, 0xf5, 0xe6);
pub const FUCHSIA: Color = Color::rgb(0xff, 0x00, 0xff);
pub const DEEP_PINK: Color = Color::rgb(0xff, 0x14, 0x93);
pub const ORANGE_RED: Color = Color::rgb(0xff, 0x45, 0x00);
pub const TOMATO: Color = Color::rgb(0xff, 0x63, 0x47);
pub const HOT_PINK: Color = Color::rgb(0xff, 0x69, 0xb4);
pub const CORAL: Color = Color::rgb(0xff, 0x7f, 0x50);
pub const DARK_ORANGE: Color = Color::rgb(0xff, 0x8c, 0x00);
pub const LIGHT_SALMON: Color = Color::rgb(0xff, 0xa0, 0x7a);
pub const ORANGE: Color = Color::rgb(0xff, 0xa5, 0x00);
pub const LIGHT_PINK: Color = Color::rgb(0xff, 0xb6, 0xc1);
pub const PINK: Color = Color::rgb(0xff, 0xc0, 0xcb);
pub const GOLD: Color = Color::rgb(0xff, 0xd7, 0x00);
pub const PEACH_PUFF: Color = Color::rgb(0xff, 0xda, 0xb9);
pub const NAVAJO_WHITE: Color = Color::rgb(0xff, 0xde, 0xad);
pub const MOCCASIN: Color = Color::rgb(0xff, 0xe4, 0xb5);
pub const BISQUE: Color = Color::rgb(0xff, 0xe4, 0xc4);
pub const MISTY_ROSE: Color = Color::rgb(0xff, 0xe4, 0xe1);
pub const BLANCHED_ALMOND: Color = Color::rgb(0xff, 0xeb, 0xcd);
pub const PAPAYA_WHIP: Color = Color::rgb(0xff, 0xef, 0xd5);
pub const LAVENDER_BLUSH: Color = Color::rgb(0xff, 0xf0, 0xf5);
pub const SEA_SHELL: Color = Color::rgb(0xff, 0xf5, 0xee);
pub const CORNSILK: Color = Color::rgb(0xff, 0xf8, 0xdc);
pub const LEMON_CHIFFON: Color = Color::rgb(0xff, 0xfa, 0xcd);
pub const FLORAL_WHITE: Color = Color::rgb(0xff, 0xfa, 0xf0);
pub const SNOW: Color = Color::rgb(0xff, 0xfa, 0xfa);
pub const LIGHT_YELLOW: Color = Color::rgb(0xff, 0xff, 0xe0);
pub const IVORY: Color = Color::rgb(0xff, 0xff, 0xf0);
