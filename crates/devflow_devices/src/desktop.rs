use devflow_protocol::Device;

pub struct DesktopDiscoverer;

impl DesktopDiscoverer {
    pub async fn discover() -> Vec<Device> {
        vec![Device::host_desktop()]
    }
}
