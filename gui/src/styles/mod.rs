mod colors;
mod dark;
mod light;
mod theme;

pub use theme::Theme;

pub mod fonts {
    pub const SARASA_UI_NERD: &[u8] = include_bytes!("../../../static/fonts/Sarasa UI SC Nerd Font Plus Material Design Icons Windows Compatible.ttf");
}

pub mod spacings {
    pub const _1: u16 = 4;
    pub const _2: u16 = 4 * 2;
    pub const _3: u16 = 4 * 3;
    pub const _4: u16 = 4 * 4;
    pub const _6: u16 = 4 * 6;
    pub const _8: u16 = 4 * 8;
    pub const _10: u16 = 4 * 10;
    pub const _11: u16 = 4 * 11;
    pub const _12: u16 = 4 * 12;
    pub const _16: u16 = 4 * 16;
    pub const _24: u16 = 4 * 24;
    pub const _32: u16 = 4 * 32;
    pub const _64: u16 = 4 * 64;
    pub const _128: u16 = 4 * 128;
}
