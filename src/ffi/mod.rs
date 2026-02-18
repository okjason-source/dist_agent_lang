// FFI (Foreign Function Interface) Module
// Provides both HTTP/REST and native FFI bindings for dist_agent_lang
// Users can choose the interface based on their performance and deployment needs

pub mod auto_detect;
pub mod c;
pub mod interface;
pub mod python;
pub mod rust;
pub mod security;

pub use auto_detect::{CallFrequency, InterfaceSelector, ServiceMetadata};
pub use interface::{FFIInterface, InterfaceType, ServiceInterface};

/// FFI Configuration
#[derive(Debug, Clone)]
pub struct FFIConfig {
    pub interface_type: InterfaceType,
    pub enable_http: bool,
    pub enable_ffi: bool,
    pub python_enabled: bool,
    pub rust_enabled: bool,
    pub c_enabled: bool,
}

impl Default for FFIConfig {
    fn default() -> Self {
        Self {
            interface_type: InterfaceType::Both,
            enable_http: true,
            enable_ffi: true,
            python_enabled: cfg!(feature = "python-ffi"),
            rust_enabled: true,
            c_enabled: cfg!(feature = "c-ffi"),
        }
    }
}

impl FFIConfig {
    pub fn http_only() -> Self {
        Self {
            interface_type: InterfaceType::HTTP,
            enable_http: true,
            enable_ffi: false,
            python_enabled: false,
            rust_enabled: false,
            c_enabled: false,
        }
    }

    pub fn ffi_only() -> Self {
        Self {
            interface_type: InterfaceType::FFI,
            enable_http: false,
            enable_ffi: true,
            python_enabled: cfg!(feature = "python-ffi"),
            rust_enabled: true,
            c_enabled: cfg!(feature = "c-ffi"),
        }
    }

    pub fn both() -> Self {
        Self::default()
    }

    /// Auto-detect configuration based on environment
    pub fn auto_detect() -> Self {
        // Check if we're in a local context (same process)
        // If both are available, default to "both" with auto-selection
        Self {
            interface_type: InterfaceType::Both,
            enable_http: true, // Always enable HTTP as fallback
            enable_ffi: true,  // Enable FFI if available
            python_enabled: cfg!(feature = "python-ffi"),
            rust_enabled: true,
            c_enabled: cfg!(feature = "c-ffi"),
        }
    }
}
