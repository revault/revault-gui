use revault_hwi::{
    dummysigner::{DummySigner, DUMMYSIGNER_DEFAULT_ADDRESS},
    specter::{Specter, SPECTER_SIMULATOR_DEFAULT_ADDRESS},
    HWIError, RevaultHWI,
};

pub async fn connect() -> Result<Box<dyn RevaultHWI + Send>, HWIError> {
    if let Ok(device) = DummySigner::try_connect(DUMMYSIGNER_DEFAULT_ADDRESS).await {
        return Ok(device.into());
    }
    if let Ok(device) = Specter::try_connect_simulator(SPECTER_SIMULATOR_DEFAULT_ADDRESS).await {
        return Ok(device.into());
    }

    if let Ok(device) = Specter::try_connect_serial() {
        return Ok(device.into());
    }

    Err(HWIError::DeviceDisconnected)
}
