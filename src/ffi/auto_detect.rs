// Auto-detection logic for interface selection
// When @api attribute is not specified, intelligently chooses HTTP or FFI

use crate::ffi::interface::InterfaceType;
use crate::runtime::values::Value;

/// Service metadata for auto-detection
#[derive(Debug, Clone)]
pub struct ServiceMetadata {
    pub name: String,
    pub function_names: Vec<String>,
    pub has_network_operations: bool,
    pub has_compute_operations: bool,
    pub estimated_call_frequency: CallFrequency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallFrequency {
    Low,    // < 10 calls/sec
    Medium, // 10-1000 calls/sec
    High,   // > 1000 calls/sec
}

impl ServiceMetadata {
    /// Auto-detect interface type based on service characteristics
    pub fn detect_interface_type(&self) -> InterfaceType {
        // Rule 1: High-frequency operations prefer FFI
        if self.estimated_call_frequency == CallFrequency::High {
            return InterfaceType::FFI;
        }

        // Rule 2: Network operations prefer HTTP
        if self.has_network_operations && !self.has_compute_operations {
            return InterfaceType::HTTP;
        }

        // Rule 3: Compute-intensive operations prefer FFI
        if self.has_compute_operations && !self.has_network_operations {
            return InterfaceType::FFI;
        }

        // Rule 4: Mixed operations use both (auto-select per call)
        InterfaceType::Both
    }

    /// Analyze function name to determine operation type
    pub fn analyze_function(function_name: &str) -> (bool, bool) {
        let network_patterns = [
            "chain::",
            "database::",
            "network_",
            "remote_",
            "fetch",
            "request",
            "api_",
            "http_",
            "web_",
        ];

        let compute_patterns = [
            "hash",
            "sign",
            "verify",
            "encrypt",
            "decrypt",
            "compute",
            "calculate",
            "process",
            "transform",
            "batch_",
            "parallel_",
            "fast_",
            "crypto::",
        ];

        let has_network = network_patterns
            .iter()
            .any(|pattern| function_name.contains(pattern));

        let has_compute = compute_patterns
            .iter()
            .any(|pattern| function_name.contains(pattern));

        (has_network, has_compute)
    }
}

/// Runtime interface selector
pub struct InterfaceSelector {
    default_interface: InterfaceType,
    service_metadata: std::collections::HashMap<String, ServiceMetadata>,
}

impl InterfaceSelector {
    pub fn new() -> Self {
        Self {
            default_interface: InterfaceType::Both, // Default: auto-select
            service_metadata: std::collections::HashMap::new(),
        }
    }

    /// Register service metadata for better auto-detection
    pub fn register_service(&mut self, metadata: ServiceMetadata) {
        self.service_metadata
            .insert(metadata.name.clone(), metadata);
    }

    /// Select interface for a service call
    pub fn select_interface(
        &self,
        service_name: &str,
        function_name: &str,
        _args: &[Value],
    ) -> InterfaceType {
        // Check if we have metadata for this service
        if let Some(metadata) = self.service_metadata.get(service_name) {
            // Use service-level detection
            let service_interface = metadata.detect_interface_type();

            // Override with function-level analysis if needed
            let (has_network, has_compute) = ServiceMetadata::analyze_function(function_name);

            if has_network && !has_compute {
                return InterfaceType::HTTP;
            }
            if has_compute && !has_network {
                return InterfaceType::FFI;
            }

            return service_interface;
        }

        // Fallback: analyze function name
        let (has_network, has_compute) = ServiceMetadata::analyze_function(function_name);

        if has_network && !has_compute {
            InterfaceType::HTTP
        } else if has_compute && !has_network {
            InterfaceType::FFI
        } else {
            self.default_interface
        }
    }

    /// Set default interface
    pub fn set_default(&mut self, interface: InterfaceType) {
        self.default_interface = interface;
    }

    /// Get the default interface
    pub fn default_interface(&self) -> InterfaceType {
        self.default_interface
    }

    /// Get the number of registered services
    pub fn service_count(&self) -> usize {
        self.service_metadata.len()
    }

    /// Check if a service is registered
    pub fn has_service(&self, service_name: &str) -> bool {
        self.service_metadata.contains_key(service_name)
    }
}

impl Default for InterfaceSelector {
    fn default() -> Self {
        Self::new()
    }
}
