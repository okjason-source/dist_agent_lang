//! Tests for `FFIConfig::both` and `FFIConfig::auto_detect` (must match intentional defaults).

use dist_agent_lang::{FFIConfig, InterfaceType};

#[test]
fn ffi_config_both_matches_default() {
    let a = FFIConfig::default();
    let b = FFIConfig::both();
    assert_eq!(a.interface_type, InterfaceType::Both);
    assert_eq!(b.interface_type, InterfaceType::Both);
    assert_eq!(a.enable_http, b.enable_http);
    assert_eq!(a.enable_ffi, b.enable_ffi);
    assert_eq!(a.rust_enabled, b.rust_enabled);
}

#[test]
fn ffi_config_auto_detect_enables_http_and_ffi() {
    let c = FFIConfig::auto_detect();
    assert_eq!(c.interface_type, InterfaceType::Both);
    assert!(c.enable_http);
    assert!(c.enable_ffi);
    assert!(c.rust_enabled);
}

#[test]
fn ffi_config_http_only_disables_ffi_paths() {
    let c = FFIConfig::http_only();
    assert_eq!(c.interface_type, InterfaceType::HTTP);
    assert!(c.enable_http);
    assert!(!c.enable_ffi);
}

#[test]
fn ffi_config_ffi_only_disables_http() {
    let c = FFIConfig::ffi_only();
    assert_eq!(c.interface_type, InterfaceType::FFI);
    assert!(!c.enable_http);
    assert!(c.enable_ffi);
}
