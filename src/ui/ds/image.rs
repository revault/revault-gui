use iced::widget::svg::{Handle, Svg};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/images/"]
struct Image;

pub fn revaut_colored_logo() -> Svg {
    let h = Handle::from_memory(Image::get("revault-colored-logo.svg").unwrap());
    Svg::new(h)
}
