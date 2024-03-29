pub mod fixtures;
pub mod mock;
pub mod sandbox;

use revault_hwi::{app::revault::RevaultHWI, HWIError};

pub async fn no_hardware_wallet() -> Result<Box<dyn RevaultHWI + Send>, HWIError> {
    Err(HWIError::DeviceNotFound)
}
