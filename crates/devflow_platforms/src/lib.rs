pub mod android;
pub mod apple;
pub mod desktop;
pub mod runner;

pub use android::AndroidPlatformRunner;
pub use apple::ApplePlatformRunner;
pub use desktop::DesktopPlatformRunner;
pub use runner::PlatformRunner;
use devflow_protocol::Platform;
use std::sync::Arc;

pub struct PlatformRegistry;

impl PlatformRegistry {
    pub fn get_runner(platform: Platform) -> Arc<dyn PlatformRunner> {
        match platform {
            Platform::Android => Arc::new(AndroidPlatformRunner::new()),
            Platform::Apple | Platform::Ios | Platform::Macos => Arc::new(ApplePlatformRunner::new()),
            Platform::Desktop | Platform::Generic => Arc::new(DesktopPlatformRunner::new()),
        }
    }
}
