use iced::widget::svg::{Handle, Svg};

const LOGO: &[u8] = include_bytes!("../../static/images/revault-colored-logo.svg");

pub fn revault_colored_logo() -> Svg {
    let h = Handle::from_memory(LOGO.to_vec());
    Svg::new(h)
}
