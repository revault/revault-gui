use std::path::PathBuf;

use iced::{Container, Element, Length};

use super::{Message, State};
use crate::ui::ds::image::revaut_colored_logo;

#[derive(Debug, Clone)]
pub struct StateCharging {
    revaultd_config_path: Option<PathBuf>,
    debug: bool,
    step: ChargingStep,
}

impl StateCharging {
    pub fn new(revaultd_config_path: Option<PathBuf>, debug: bool) -> Self {
        StateCharging {
            revaultd_config_path,
            debug,
            step: ChargingStep::Connecting,
        }
    }
}

impl State for StateCharging {
    fn view(&mut self) -> Element<Message> {
        let svg = revaut_colored_logo()
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(svg)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum ChargingStep {
    Connecting,
    Syncing,
    Error,
}
