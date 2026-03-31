//! Edge tests for `cli_design` (version, help text). Catches mutants that blank `version()` / `help_content()`.

use dist_agent_lang::cli_design::{help_content, print_banner, version, TAGLINE, TAGLINE_FULL};

#[test]
fn version_is_non_empty_semver_like() {
    let v = version();
    assert!(
        !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'),
        "version should look like a semver from CARGO_PKG_VERSION, got {:?}",
        v
    );
}

#[test]
fn help_content_includes_bin_and_key_sections() {
    let h = help_content("dal");
    assert!(
        h.contains("GET STARTED") && h.contains("AI & CODE ASSISTANCE"),
        "help layout must preserve discoverability sections"
    );
    assert!(
        h.contains("dal bond") || h.contains("dal run"),
        "quick examples should mention the binary name; got len {}",
        h.len()
    );
    assert!(
        h.len() > 400,
        "help_content must be substantial (catches replace with String::new()); len={}",
        h.len()
    );
}

#[test]
fn tagline_constants_non_empty() {
    assert!(!TAGLINE.is_empty() && !TAGLINE_FULL.is_empty());
    assert!(TAGLINE_FULL.len() > TAGLINE.len());
}

#[test]
fn print_banner_no_banner_is_noop() {
    print_banner("dal", true, false);
    print_banner("dal", true, true);
}

#[test]
fn print_banner_quiet_one_line_does_not_panic() {
    print_banner("dal", false, true);
}
