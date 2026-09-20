use crate::adapter::FrameworkAdapter;
use crate::flutter::FlutterFrameworkAdapter;
use crate::generic::GenericFrameworkAdapter;
use crate::kotlin::KotlinFrameworkAdapter;
use crate::react_native::ReactNativeFrameworkAdapter;
use crate::swift::SwiftFrameworkAdapter;
use crate::tauri::TauriFrameworkAdapter;
use crate::xcode::XcodeAdapter;
use devflow_core::Project;
use std::sync::Arc;

pub struct FrameworkRegistry {
    adapters: Vec<Arc<dyn FrameworkAdapter>>,
}

impl FrameworkRegistry {
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Arc::new(GenericFrameworkAdapter::new()),
                Arc::new(XcodeAdapter::new()),
                Arc::new(SwiftFrameworkAdapter::new()),
                Arc::new(KotlinFrameworkAdapter::new()),
                Arc::new(ReactNativeFrameworkAdapter::new()),
                Arc::new(FlutterFrameworkAdapter::new()),
                Arc::new(TauriFrameworkAdapter::new()),
            ],
        }
    }

    pub fn select_adapter(&self, project: &Project) -> Arc<dyn FrameworkAdapter> {
        // 1. If devflow.toml has a specific framework configured
        if let Some(ref cfg) = project.config {
            let fw = cfg.project.framework.to_lowercase();
            if fw == "xcode" || fw == "ios" || fw == "apple" {
                return Arc::new(XcodeAdapter::new());
            } else if fw == "swift" || fw == "swiftpm" {
                return Arc::new(SwiftFrameworkAdapter::new());
            } else if fw == "kotlin" || fw == "android" {
                return Arc::new(KotlinFrameworkAdapter::new());
            } else if fw == "react-native" || fw == "rn" {
                return Arc::new(ReactNativeFrameworkAdapter::new());
            } else if fw == "flutter" {
                return Arc::new(FlutterFrameworkAdapter::new());
            } else if fw == "tauri" {
                return Arc::new(TauriFrameworkAdapter::new());
            } else {
                return Arc::new(GenericFrameworkAdapter::new());
            }
        }

        // 2. Auto-detection on directory (order: specialized frameworks first)
        for adapter in self.adapters.iter().rev() {
            if adapter.name() != "generic" && adapter.detect(&project.root_dir) {
                return adapter.clone();
            }
        }

        // Default to generic
        Arc::new(GenericFrameworkAdapter::new())
    }
}

impl Default for FrameworkRegistry {
    fn default() -> Self {
        Self::new()
    }
}
