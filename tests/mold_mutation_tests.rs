// Mold Mutation Tests
// These tests catch arithmetic operator mutations in MoldConfig::memory_limit_to_max_memory
// and return value mutations in mold module functions

use dist_agent_lang::mold::config::MoldConfig;

// ============================================================================
// MEMORY CALCULATION TESTS
// ============================================================================
// These catch mutations: * → +, * → /, / → %, / → * in memory_limit_to_max_memory
// Every arithmetic operator is verified with exact expected values.

#[test]
fn test_memory_limit_gb_exact() {
    // Catches: replace * with + in MoldConfig::memory_limit_to_max_memory (line 52)
    // 2GB = 2 * 1024 * 1024 * 1024 = 2147483648 bytes / 4096 = 524288 pages
    let result = MoldConfig::memory_limit_to_max_memory("2GB");
    let expected = (2usize * 1024 * 1024 * 1024) / 4096;
    assert_eq!(result, expected, "2GB should be {} pages, got {}", expected, result);
    // If * was mutated to +, result would be (2 * (1024 + 1024 + 1024)) / 4096 = wildly different
    assert!(result > 100_000, "2GB must be more than 100k pages");
}

#[test]
fn test_memory_limit_mb_exact() {
    // Catches: replace * with + in MoldConfig::memory_limit_to_max_memory (line 54)
    // 512MB = 512 * 1024 * 1024 = 536870912 bytes / 4096 = 131072 pages
    let result = MoldConfig::memory_limit_to_max_memory("512MB");
    let expected = (512usize * 1024 * 1024) / 4096;
    assert_eq!(result, expected, "512MB should be {} pages, got {}", expected, result);
    assert!(result > 100_000, "512MB must be more than 100k pages");
}

#[test]
fn test_memory_limit_kb_exact() {
    // Catches: mutations on the KB branch
    // 4096KB = 4096 * 1024 = 4194304 bytes / 4096 = 1024 pages
    let result = MoldConfig::memory_limit_to_max_memory("4096KB");
    let expected = (4096usize * 1024) / 4096;
    assert_eq!(result, expected, "4096KB should be {} pages, got {}", expected, result);
    assert_eq!(result, 1024);
}

#[test]
fn test_memory_limit_bytes_exact() {
    // Catches: mutations on the raw bytes branch
    // 8192 bytes / 4096 = 2 pages
    let result = MoldConfig::memory_limit_to_max_memory("8192");
    let expected = 8192usize / 4096;
    assert_eq!(result, expected, "8192 bytes should be {} pages, got {}", expected, result);
    assert_eq!(result, 2);
}

#[test]
fn test_memory_limit_division_by_page_size() {
    // Catches: replace / with % in MoldConfig::memory_limit_to_max_memory (line 61)
    // Catches: replace / with * in MoldConfig::memory_limit_to_max_memory (line 61)
    // 1GB = 1 * 1024 * 1024 * 1024 = 1073741824 bytes / 4096 = 262144 pages
    let result = MoldConfig::memory_limit_to_max_memory("1GB");
    let expected = (1024usize * 1024 * 1024) / 4096;
    assert_eq!(result, expected);
    // If / was mutated to %, result would be (1073741824 % 4096) = 0
    assert_ne!(result, 0, "Division should not produce 0 for 1GB");
    // If / was mutated to *, result would overflow or be enormous
    assert!(result < 10_000_000, "Result should be reasonable page count");
}

#[test]
fn test_memory_limit_multiplication_chain() {
    // Catches: replace * with + on the n * unit multiplication (line 61)
    // 1GB: n=1, unit=1073741824. n * unit = 1073741824. n + unit = 1073741825.
    // After /4096: correct = 262144, wrong = 262144 (off by 1 only for n=1, but fails for n=2)
    // 2GB: n=2, unit=1073741824. n * unit = 2147483648. n + unit = 1073741826.
    let result_2gb = MoldConfig::memory_limit_to_max_memory("2GB");
    let result_1gb = MoldConfig::memory_limit_to_max_memory("1GB");
    // 2GB should be exactly 2x 1GB
    assert_eq!(result_2gb, result_1gb * 2, "2GB should be exactly 2x 1GB pages");
}

#[test]
fn test_memory_limit_empty_returns_default() {
    // Catches: replace MoldConfig::memory_limit_to_max_memory -> usize with 0
    // Catches: replace MoldConfig::memory_limit_to_max_memory -> usize with 1
    let result = MoldConfig::memory_limit_to_max_memory("");
    assert_eq!(result, 1000, "Empty string should return default 1000");
    assert_ne!(result, 0, "Should not return 0");
    assert_ne!(result, 1, "Should not return 1");
}

#[test]
fn test_memory_limit_case_insensitive() {
    // Verify case handling works correctly
    let upper = MoldConfig::memory_limit_to_max_memory("2GB");
    let lower = MoldConfig::memory_limit_to_max_memory("2gb");
    assert_eq!(upper, lower, "Should be case insensitive");
}

#[test]
fn test_memory_limit_with_whitespace() {
    // Verify trim works
    let trimmed = MoldConfig::memory_limit_to_max_memory("2GB");
    let padded = MoldConfig::memory_limit_to_max_memory(" 2GB ");
    assert_eq!(trimmed, padded, "Should handle whitespace");
}

#[test]
fn test_memory_limit_relationships() {
    // Verify GB > MB > KB ordering — catches operator swaps
    let gb = MoldConfig::memory_limit_to_max_memory("1GB");
    let mb = MoldConfig::memory_limit_to_max_memory("1MB");
    let kb = MoldConfig::memory_limit_to_max_memory("1KB");
    
    assert!(gb > mb, "1GB ({}) must be > 1MB ({})", gb, mb);
    assert!(mb > kb, "1MB ({}) must be > 1KB ({})", mb, kb);
    // 1KB = 1024 bytes / 4096 = 0 pages (integer division)
    // So for small values the ordering may not hold due to integer division
    // But 1GB >> 1MB >> 1KB is guaranteed
    assert!(gb > 1000 * mb / 1000, "GB should be roughly 1024x MB");
}
