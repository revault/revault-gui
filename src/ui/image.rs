use iced::widget::svg::{Handle, Svg};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/images/"]
struct Image;

pub fn revault_colored_logo() -> Svg {
    let h = Handle::from_memory(Image::get("revault-colored-logo.svg").unwrap());
    Svg::new(h)
}

pub fn home_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/home.svg").unwrap());
    Svg::new(h)
}

pub fn home_white_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/home_white.svg").unwrap());
    Svg::new(h)
}

pub fn send_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/send.svg").unwrap());
    Svg::new(h)
}

pub fn history_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/history.svg").unwrap());
    Svg::new(h)
}

pub fn history_white_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/history_white.svg").unwrap());
    Svg::new(h)
}

pub fn settings_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/settings.svg").unwrap());
    Svg::new(h)
}

pub fn block_icon() -> Svg {
    let h = Handle::from_memory(Image::get("icons/block.svg").unwrap());
    Svg::new(h)
}
