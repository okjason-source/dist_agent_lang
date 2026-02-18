// Type system mappings between Solidity and DAL

use std::collections::HashMap;

/// Solidity type to DAL type mapping
pub struct TypeMapper {
    mappings: HashMap<String, String>,
}

impl TypeMapper {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Basic types
        mappings.insert("uint256".to_string(), "int".to_string());
        mappings.insert("uint128".to_string(), "int".to_string());
        mappings.insert("uint64".to_string(), "int".to_string());
        mappings.insert("uint32".to_string(), "int".to_string());
        mappings.insert("uint16".to_string(), "int".to_string());
        mappings.insert("uint8".to_string(), "int".to_string());
        mappings.insert("uint".to_string(), "int".to_string());

        mappings.insert("int256".to_string(), "int".to_string());
        mappings.insert("int128".to_string(), "int".to_string());
        mappings.insert("int64".to_string(), "int".to_string());
        mappings.insert("int32".to_string(), "int".to_string());
        mappings.insert("int16".to_string(), "int".to_string());
        mappings.insert("int8".to_string(), "int".to_string());
        mappings.insert("int".to_string(), "int".to_string());

        mappings.insert("address".to_string(), "string".to_string());
        mappings.insert("bool".to_string(), "bool".to_string());
        mappings.insert("string".to_string(), "string".to_string());

        // Bytes types
        mappings.insert("bytes".to_string(), "vector<u8>".to_string());
        mappings.insert("bytes32".to_string(), "vector<u8>".to_string());
        mappings.insert("bytes16".to_string(), "vector<u8>".to_string());
        mappings.insert("bytes8".to_string(), "vector<u8>".to_string());

        Self { mappings }
    }

    /// Convert Solidity type to DAL type
    pub fn convert_type(&self, solidity_type: &str) -> String {
        // Handle arrays
        if solidity_type.ends_with("[]") {
            let base_type = &solidity_type[..solidity_type.len() - 2];
            let dal_base = self.convert_type(base_type);
            return format!("vector<{}>", dal_base);
        }

        // Handle fixed-size arrays
        if let Some(start) = solidity_type.find('[') {
            let base_type = &solidity_type[..start];
            let dal_base = self.convert_type(base_type);
            return format!("vector<{}>", dal_base);
        }

        // Handle mappings
        if solidity_type.starts_with("mapping(") {
            return self.convert_mapping(solidity_type);
        }

        // Direct mapping
        self.mappings
            .get(solidity_type)
            .cloned()
            .unwrap_or_else(|| {
                // Unknown type - try to convert as-is or use string
                if solidity_type.contains("struct") || solidity_type.contains("enum") {
                    solidity_type.to_string()
                } else {
                    format!("string") // Fallback to string for unknown types
                }
            })
    }

    /// Convert Solidity mapping to DAL map
    fn convert_mapping(&self, mapping_type: &str) -> String {
        // Parse mapping(keyType => valueType)
        if let Some(start) = mapping_type.find('(') {
            if let Some(end) = mapping_type.rfind(')') {
                let inner = &mapping_type[start + 1..end];
                if let Some(arrow) = inner.find("=>") {
                    let key_type = inner[..arrow].trim();
                    let value_type = inner[arrow + 2..].trim();
                    let dal_key = self.convert_type(key_type);
                    let dal_value = self.convert_type(value_type);
                    return format!("map<{}, {}>", dal_key, dal_value);
                }
            }
        }
        format!("map<string, any>") // Fallback
    }

    /// Check if type is supported
    pub fn is_supported(&self, solidity_type: &str) -> bool {
        // Remove array brackets for checking
        let base_type = solidity_type
            .trim_end_matches("[]")
            .split('[')
            .next()
            .unwrap_or(solidity_type);

        self.mappings.contains_key(base_type)
            || base_type.starts_with("mapping(")
            || base_type.contains("struct")
            || base_type.contains("enum")
    }
}

impl Default for TypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_conversion() {
        let mapper = TypeMapper::new();
        assert_eq!(mapper.convert_type("uint256"), "int");
        assert_eq!(mapper.convert_type("address"), "string");
        assert_eq!(mapper.convert_type("bool"), "bool");
        assert_eq!(mapper.convert_type("string"), "string");
    }

    #[test]
    fn test_array_conversion() {
        let mapper = TypeMapper::new();
        assert_eq!(mapper.convert_type("uint256[]"), "vector<int>");
        assert_eq!(mapper.convert_type("address[]"), "vector<string>");
    }

    #[test]
    fn test_mapping_conversion() {
        let mapper = TypeMapper::new();
        let result = mapper.convert_type("mapping(address => uint256)");
        assert!(result.contains("map"));
        assert!(result.contains("string")); // address -> string
        assert!(result.contains("int")); // uint256 -> int
    }
}
