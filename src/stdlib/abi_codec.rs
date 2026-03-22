const SELECTOR_ERROR_STRING: &str = "08c379a0";
const SELECTOR_PANIC_UINT256: &str = "4e487b71";

pub fn decode_uint256_word(word_hex: &str) -> Result<u128, String> {
    let normalized = word_hex.trim().trim_start_matches("0x");
    if normalized.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", normalized.len()));
    }
    u128::from_str_radix(normalized, 16).map_err(|e| e.to_string())
}

pub fn decode_bool_word(word_hex: &str) -> Result<bool, String> {
    let value = decode_uint256_word(word_hex)?;
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("invalid ABI bool value {}", value)),
    }
}

pub fn decode_address_word(word_hex: &str) -> Result<String, String> {
    let normalized = word_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() != 64 {
        return Err(format!("expected 64 hex chars, got {}", normalized.len()));
    }
    Ok(format!("0x{}", &normalized[24..]))
}

pub fn decode_abi_string_data(payload_hex: &str) -> Result<String, String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 128 {
        return Err("ABI dynamic string payload too short".to_string());
    }
    let offset = decode_uint256_word(&normalized[0..64])? as usize;
    if offset % 32 != 0 {
        return Err(format!("invalid ABI string offset {}", offset));
    }
    let offset_hex = offset * 2;
    if normalized.len() < offset_hex + 64 {
        return Err("ABI string length word out of bounds".to_string());
    }
    let str_len = decode_uint256_word(&normalized[offset_hex..offset_hex + 64])? as usize;
    let str_data_start = offset_hex + 64;
    let str_data_end = str_data_start + (str_len * 2);
    if normalized.len() < str_data_end {
        return Err("ABI string data out of bounds".to_string());
    }
    let bytes = hex::decode(&normalized[str_data_start..str_data_end])
        .map_err(|e| format!("invalid ABI string hex payload: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("invalid UTF-8 ABI string payload: {}", e))
}

pub fn decode_abi_bytes_data(payload_hex: &str) -> Result<Vec<u8>, String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 128 {
        return Err("ABI dynamic bytes payload too short".to_string());
    }
    let offset = decode_uint256_word(&normalized[0..64])? as usize;
    if offset % 32 != 0 {
        return Err(format!("invalid ABI bytes offset {}", offset));
    }
    let offset_hex = offset * 2;
    if normalized.len() < offset_hex + 64 {
        return Err("ABI bytes length word out of bounds".to_string());
    }
    let data_len = decode_uint256_word(&normalized[offset_hex..offset_hex + 64])? as usize;
    let data_start = offset_hex + 64;
    let data_end = data_start + (data_len * 2);
    if normalized.len() < data_end {
        return Err("ABI bytes data out of bounds".to_string());
    }
    hex::decode(&normalized[data_start..data_end])
        .map_err(|e| format!("invalid ABI bytes hex payload: {}", e))
}

pub fn decode_abi_tuple_string_bytes_payload(
    payload_hex: &str,
) -> Result<(String, Vec<u8>), String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 128 {
        return Err("ABI tuple(string,bytes) payload too short".to_string());
    }

    let string_offset = decode_uint256_word(&normalized[0..64])? as usize;
    let bytes_offset = decode_uint256_word(&normalized[64..128])? as usize;

    if string_offset % 32 != 0 || bytes_offset % 32 != 0 {
        return Err(format!(
            "invalid tuple offsets string={} bytes={}",
            string_offset, bytes_offset
        ));
    }

    let decode_segment = |offset: usize| -> Result<Vec<u8>, String> {
        let offset_hex = offset * 2;
        if normalized.len() < offset_hex + 64 {
            return Err("tuple dynamic length word out of bounds".to_string());
        }
        let len = decode_uint256_word(&normalized[offset_hex..offset_hex + 64])? as usize;
        let data_start = offset_hex + 64;
        let data_end = data_start + (len * 2);
        if normalized.len() < data_end {
            return Err("tuple dynamic data out of bounds".to_string());
        }
        hex::decode(&normalized[data_start..data_end])
            .map_err(|e| format!("invalid tuple dynamic hex payload: {}", e))
    };

    let string_bytes = decode_segment(string_offset)?;
    let bytes = decode_segment(bytes_offset)?;
    let string = String::from_utf8(string_bytes)
        .map_err(|e| format!("invalid UTF-8 tuple string payload: {}", e))?;
    Ok((string, bytes))
}

pub fn decode_custom_error_payload_words(
    payload_hex: &str,
    expected_selector_hex: &str,
) -> Result<Vec<String>, String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 8 {
        return Err("custom error payload too short".to_string());
    }
    let expected = expected_selector_hex
        .trim()
        .trim_start_matches("0x")
        .to_lowercase();
    let actual = &normalized[0..8];
    if actual != expected {
        return Err(format!(
            "unexpected custom error selector {}, expected {}",
            actual, expected
        ));
    }
    let words_hex = &normalized[8..];
    if words_hex.len() % 64 != 0 {
        return Err(format!(
            "custom error payload words not 32-byte aligned (hex len {})",
            words_hex.len()
        ));
    }
    let mut words = Vec::new();
    for idx in (0..words_hex.len()).step_by(64) {
        words.push(words_hex[idx..idx + 64].to_string());
    }
    Ok(words)
}

pub fn decode_static_tuple_address_uint_bool_payload(
    payload_hex: &str,
) -> Result<(String, u128, bool), String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() != 192 {
        return Err(format!(
            "expected static tuple payload of 192 hex chars, got {}",
            normalized.len()
        ));
    }
    let address = decode_address_word(&normalized[0..64])?;
    let amount = decode_uint256_word(&normalized[64..128])?;
    let flag = decode_bool_word(&normalized[128..192])?;
    Ok((address, amount, flag))
}

pub fn decode_revert_error_string_payload(payload_hex: &str) -> Result<String, String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 8 {
        return Err("revert payload too short".to_string());
    }
    let selector = &normalized[0..8];
    if selector != SELECTOR_ERROR_STRING {
        return Err(format!(
            "unexpected revert selector {}, expected {}",
            selector, SELECTOR_ERROR_STRING
        ));
    }
    decode_abi_string_data(&normalized[8..])
}

pub fn decode_revert_panic_code_payload(payload_hex: &str) -> Result<u128, String> {
    let normalized = payload_hex.trim().trim_start_matches("0x").to_lowercase();
    if normalized.len() < 8 + 64 {
        return Err("panic payload too short".to_string());
    }
    let selector = &normalized[0..8];
    if selector != SELECTOR_PANIC_UINT256 {
        return Err(format!(
            "unexpected panic selector {}, expected {}",
            selector, SELECTOR_PANIC_UINT256
        ));
    }
    decode_uint256_word(&normalized[8..72])
}

#[cfg(test)]
mod tests {
    use super::{
        decode_abi_bytes_data, decode_abi_string_data, decode_abi_tuple_string_bytes_payload,
        decode_custom_error_payload_words, decode_revert_error_string_payload,
        decode_revert_panic_code_payload, decode_static_tuple_address_uint_bool_payload,
        decode_uint256_word,
    };

    #[test]
    fn decodes_dynamic_string_and_bytes_payloads() {
        let str_payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000005\
68656c6c6f000000000000000000000000000000000000000000000000000000";
        assert_eq!(decode_abi_string_data(str_payload).unwrap(), "hello");

        let bytes_payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000004\
deadbeef00000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            decode_abi_bytes_data(bytes_payload).unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn decodes_nested_dynamic_tuple_and_revert_payloads() {
        let tuple_payload = "0x0000000000000000000000000000000000000000000000000000000000000040\
0000000000000000000000000000000000000000000000000000000000000080\
0000000000000000000000000000000000000000000000000000000000000005\
68656c6c6f000000000000000000000000000000000000000000000000000000\
0000000000000000000000000000000000000000000000000000000000000004\
deadbeef00000000000000000000000000000000000000000000000000000000";
        let (s, b) = decode_abi_tuple_string_bytes_payload(tuple_payload).unwrap();
        assert_eq!(s, "hello");
        assert_eq!(b, vec![0xde, 0xad, 0xbe, 0xef]);

        let error_payload = "0x08c379a00000000000000000000000000000000000000000000000000000000000000020\
00000000000000000000000000000000000000000000000000000000000000046e6f706500000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            decode_revert_error_string_payload(error_payload).unwrap(),
            "nope"
        );

        let panic_payload =
            "0x4e487b710000000000000000000000000000000000000000000000000000000000000011";
        assert_eq!(
            decode_revert_panic_code_payload(panic_payload).unwrap(),
            0x11
        );
    }

    #[test]
    fn rejects_selector_mismatch_and_malformed_lengths() {
        let payload = "0xdeadbeef0000000000000000000000000000000000000000000000000000000000000001";
        let err = decode_custom_error_payload_words(payload, "0xfeedface").unwrap_err();
        assert!(err.contains("unexpected custom error selector"));

        let malformed = "0xdeadbeef00";
        let err = decode_custom_error_payload_words(malformed, "0xdeadbeef").unwrap_err();
        assert!(err.contains("not 32-byte aligned"));

        let bad_tuple_len = "0x00";
        let err = decode_static_tuple_address_uint_bool_payload(bad_tuple_len).unwrap_err();
        assert!(err.contains("expected static tuple payload"));

        let bad_word = "0x1";
        let err = decode_uint256_word(bad_word).unwrap_err();
        assert!(err.contains("expected 64 hex chars"));
    }
}
