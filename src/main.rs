pub mod app;
pub mod revaultd;
pub mod ui;

use iced::{Application, Settings};

use ui::UI;

fn main() {
    UI::run(Settings::default());
}
