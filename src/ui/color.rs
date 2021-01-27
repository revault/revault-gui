use iced::Color;

pub const BACKGROUND: Color = Color::from_rgb(
    0xF6 as f32 / 255.0,
    0xF7 as f32 / 255.0,
    0xF8 as f32 / 255.0,
);

pub const BACKGROUND_LIGHT: Color = Color::from_rgb(
    0xFA as f32 / 255.0,
    0xFA as f32 / 255.0,
    0xFA as f32 / 255.0,
);

pub const FOREGROUND: Color = Color::WHITE;

pub const SECONDARY: Color = Color::from_rgb(
    0xe1 as f32 / 255.0,
    0xe4 as f32 / 255.0,
    0xe8 as f32 / 255.0,
);

pub const PRIMARY: Color = Color::from_rgb(
    0xF0 as f32 / 255.0,
    0x43 as f32 / 255.0,
    0x59 as f32 / 255.0,
);

pub const PRIMARY_LIGHT: Color = Color::from_rgba(
    0xF0 as f32 / 255.0,
    0x43 as f32 / 255.0,
    0x59 as f32 / 255.0,
    0.5f32,
);

pub const SUCCESS: Color = Color::from_rgb(
    0x29 as f32 / 255.0,
    0xBC as f32 / 255.0,
    0x97 as f32 / 255.0,
);

#[allow(dead_code)]
pub const SUCCESS_LIGHT: Color = Color::from_rgba(
    0x29 as f32 / 255.0,
    0xBC as f32 / 255.0,
    0x97 as f32 / 255.0,
    0.5f32,
);

pub const WARNING: Color = Color::from_rgb(
    0xF0 as f32 / 255.0,
    0x43 as f32 / 255.0,
    0x59 as f32 / 255.0,
);

pub const WARNING_LIGHT: Color = Color::from_rgba(
    0xF0 as f32 / 255.0,
    0x43 as f32 / 255.0,
    0x59 as f32 / 255.0,
    0.5f32,
);

pub const CANCEL: Color = Color::from_rgb(
    0x34 as f32 / 255.0,
    0x37 as f32 / 255.0,
    0x3D as f32 / 255.0,
);

pub const INFO: Color = Color::from_rgb(
    0x2A as f32 / 255.0,
    0x98 as f32 / 255.0,
    0xBD as f32 / 255.0,
);

pub const INFO_LIGHT: Color = Color::from_rgba(
    0x2A as f32 / 255.0,
    0x98 as f32 / 255.0,
    0xBD as f32 / 255.0,
    0.5f32,
);
