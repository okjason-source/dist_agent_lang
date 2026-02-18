use crate::runtime::values::Value;
use std::collections::HashMap;
use std::env;

/// Chain namespace for comprehensive blockchain operations
/// Provides multi-chain support with deployment, interaction, and monitoring

#[cfg(feature = "http-interface")]
mod rpc {
    use lazy_static::lazy_static;
    use serde_json::{json, Value as JsonValue};
    use std::sync::{Arc, Mutex};

    // Shared reqwest client to avoid concurrent initialization panics
    // Reqwest's blocking client runtime can only be initialized once per process
    lazy_static! {
        static ref CLIENT: Arc<Mutex<Option<Arc<reqwest::blocking::Client>>>> =
            Arc::new(Mutex::new(None));
    }

    fn get_client() -> Result<Arc<reqwest::blocking::Client>, String> {
        let mut client_guard = CLIENT
            .lock()
            .map_err(|e| format!("Mutex poisoned: {}", e))?;

        if let Some(ref client) = *client_guard {
            // Return a clone of the Arc (cheap, shares the underlying client)
            return Ok(Arc::clone(client));
        }

        // Create new client and store it
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let client_arc = Arc::new(client);
        *client_guard = Some(Arc::clone(&client_arc));
        Ok(client_arc)
    }

    /// Perform a JSON-RPC 2.0 request to the chain RPC endpoint.
    /// Returns the "result" field on success, or an error string.
    pub(super) fn rpc_request(
        rpc_url: &str,
        method: &str,
        params: Vec<JsonValue>,
    ) -> Result<JsonValue, String> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let client = get_client()?;
        let resp = client
            .post(rpc_url)
            .json(&body)
            .send()
            .map_err(|e| e.to_string())?;
        let status = resp.status();
        let json: JsonValue = resp.json().map_err(|e| e.to_string())?;
        if let Some(err) = json.get("error") {
            let msg = err
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("RPC error");
            return Err(msg.to_string());
        }
        if !status.is_success() {
            return Err(format!("RPC HTTP {}", status));
        }
        json.get("result")
            .cloned()
            .ok_or_else(|| "Missing result".to_string())
    }

    /// Parse hex string (with or without 0x) to u128, saturating to i64 for balance/amounts.
    pub(super) fn hex_to_i64(hex_str: &str) -> i64 {
        let s = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let s = s.trim_start_matches('0');
        if s.is_empty() {
            return 0;
        }
        let mut value: u128 = 0;
        for c in s.chars() {
            let d = c.to_digit(16).unwrap_or(0) as u128;
            value = value.saturating_mul(16).saturating_add(d);
        }
        value.min(i64::MAX as u128) as i64
    }

    /// Parse hex gas price to gwei (f64).
    pub(super) fn hex_gas_price_to_gwei(hex_str: &str) -> f64 {
        let raw = hex_to_i64(hex_str);
        if raw <= 0 {
            return 0.0;
        }
        raw as f64 / 1_000_000_000.0
    }
}

// Chain configuration structure
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: i64,
    pub name: String,
    pub rpc_url: String,
    pub explorer: String,
    pub gas_limit: i64,
    pub gas_price: f64,
    pub confirmations: i64,
    pub is_testnet: bool,
}

// Global chain registry
lazy_static::lazy_static! {
    static ref CHAIN_REGISTRY: HashMap<i64, ChainConfig> = {
        let mut m = HashMap::new();

        // Ethereum Mainnet
        m.insert(1, ChainConfig {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            explorer: "https://etherscan.io".to_string(),
            gas_limit: 21000,
            gas_price: 20.0,
            confirmations: 12,
            is_testnet: false,
        });

        // Polygon
        m.insert(137, ChainConfig {
            chain_id: 137,
            name: "Polygon".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            explorer: "https://polygonscan.com".to_string(),
            gas_limit: 21000,
            gas_price: 30.0,
            confirmations: 256,
            is_testnet: false,
        });

        // Binance Smart Chain
        m.insert(56, ChainConfig {
            chain_id: 56,
            name: "Binance Smart Chain".to_string(),
            rpc_url: "https://bsc-dataseed.binance.org".to_string(),
            explorer: "https://bscscan.com".to_string(),
            gas_limit: 21000,
            gas_price: 5.0,
            confirmations: 15,
            is_testnet: false,
        });

        // Arbitrum
        m.insert(42161, ChainConfig {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            explorer: "https://arbiscan.io".to_string(),
            gas_limit: 21000,
            gas_price: 0.1,
            confirmations: 1,
            is_testnet: false,
        });

        // Test Networks
        m.insert(5, ChainConfig {
            chain_id: 5,
            name: "Ethereum Goerli".to_string(),
            rpc_url: "https://goerli.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            explorer: "https://goerli.etherscan.io".to_string(),
            gas_limit: 21000,
            gas_price: 2.0,
            confirmations: 6,
            is_testnet: true,
        });

        m.insert(80001, ChainConfig {
            chain_id: 80001,
            name: "Polygon Mumbai".to_string(),
            rpc_url: "https://rpc-mumbai.maticvigil.com".to_string(),
            explorer: "https://mumbai.polygonscan.com".to_string(),
            gas_limit: 21000,
            gas_price: 1.0,
            confirmations: 6,
            is_testnet: true,
        });

        m
    };
}

/// Get chain configuration by chain ID
pub fn get_chain_config(chain_id: i64) -> Option<ChainConfig> {
    CHAIN_REGISTRY.get(&chain_id).cloned()
}

/// Get all supported chains
pub fn get_supported_chains() -> Vec<ChainConfig> {
    CHAIN_REGISTRY.values().cloned().collect()
}

/// Deploy a contract to a specific chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `contract_name` - Name of the contract to deploy
/// * `constructor_args` - Constructor arguments
///
/// # Returns
/// * `String` - The deployed contract address
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// use std::collections::HashMap;
/// let mut args = HashMap::new();
/// args.insert("name".to_string(), "KEYS".to_string());
/// args.insert("symbol".to_string(), "KEYS".to_string());
/// let address = chain::deploy(1, "KEYS_Token".to_string(), args);
/// ```
pub fn deploy(
    chain_id: i64,
    contract_name: String,
    constructor_args: HashMap<String, String>,
) -> String {
    // Log the deployment operation
    crate::stdlib::log::audit(
        "deploy",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("chain_id".to_string(), Value::Int(chain_id));
            data.insert(
                "contract_name".to_string(),
                Value::String(contract_name.clone()),
            );
            data.insert(
                "constructor_args".to_string(),
                Value::String(format!("{:?}", constructor_args)),
            );
            data
        },
        Some("chain"),
    );

    // Get chain config
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => {
            crate::stdlib::log::error(
                "deploy",
                {
                    let mut data = std::collections::HashMap::new();
                    data.insert("chain_id".to_string(), Value::Int(chain_id));
                    data.insert(
                        "error".to_string(),
                        Value::String(format!("Chain {} not supported", chain_id)),
                    );
                    data
                },
                Some("chain"),
            );
            return String::new();
        }
    };

    #[cfg(feature = "http-interface")]
    if let Some(raw_tx_hex) = constructor_args
        .get("raw_transaction")
        .or_else(|| constructor_args.get("signed_tx"))
    {
        if let Ok(addr) = deploy_via_raw_transaction(&chain_config.rpc_url, raw_tx_hex) {
            crate::stdlib::log::info(
                "deploy_success",
                {
                    let mut data = std::collections::HashMap::new();
                    data.insert("chain_id".to_string(), Value::Int(chain_id));
                    data.insert("contract_name".to_string(), Value::String(contract_name));
                    data.insert("address".to_string(), Value::String(addr.clone()));
                    data
                },
                None,
            );
            return addr;
        }
    }

    // Fallback: mock contract address (use when no raw_transaction or RPC unavailable)
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let address = format!("0x{:040x}", timestamp);

    crate::stdlib::log::info(
        "deploy_success",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("chain_id".to_string(), Value::Int(chain_id));
            data.insert("contract_name".to_string(), Value::String(contract_name));
            data.insert("address".to_string(), Value::String(address.clone()));
            data
        },
        None,
    );

    address
}

#[cfg(feature = "http-interface")]
fn deploy_via_raw_transaction(rpc_url: &str, raw_tx_hex: &str) -> Result<String, String> {
    use serde_json::json;
    let tx_hex = raw_tx_hex.strip_prefix("0x").unwrap_or(raw_tx_hex);
    let result = rpc::rpc_request(
        rpc_url,
        "eth_sendRawTransaction",
        vec![json!(format!("0x{}", tx_hex))],
    )?;
    let tx_hash = result.as_str().ok_or("expected tx hash string")?;
    let receipt = wait_for_receipt(rpc_url, tx_hash, 30)?;
    let addr = receipt
        .get("contractAddress")
        .and_then(|v| v.as_str())
        .ok_or("no contractAddress in receipt")?
        .to_string();
    Ok(addr)
}

#[cfg(feature = "http-interface")]
fn wait_for_receipt(
    rpc_url: &str,
    tx_hash: &str,
    max_attempts: u32,
) -> Result<serde_json::Value, String> {
    use serde_json::json;
    for _ in 0..max_attempts {
        let result = rpc::rpc_request(rpc_url, "eth_getTransactionReceipt", vec![json!(tx_hash)])?;
        if !result.is_null() {
            return Ok(result);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    Err("timeout waiting for transaction receipt".to_string())
}

/// Estimate gas for an operation on a specific chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `operation` - The operation to estimate gas for
///
/// # Returns
/// * `i64` - Estimated gas cost
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// let gas = chain::estimate_gas(1, "transfer".to_string());
/// ```
pub fn estimate_gas(chain_id: i64, operation: String) -> i64 {
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => return 0,
    };

    #[cfg(feature = "http-interface")]
    {
        use serde_json::json;
        let params = vec![json!({ "to": null, "data": "0x" })];
        if let Ok(result) = rpc::rpc_request(&chain_config.rpc_url, "eth_estimateGas", params) {
            if let Some(hex_str) = result.as_str() {
                let gas = rpc::hex_to_i64(hex_str);
                if gas > 0 {
                    return gas;
                }
            }
        }
    }

    // Fallback: mock gas estimation when RPC unavailable or not configured
    let base_gas = match operation.as_str() {
        "transfer" => 21000,
        "mint" => 50000,
        "burn" => 30000,
        "approve" => 46000,
        "deploy" => 200000,
        _ => 21000,
    };
    let adjusted_gas = if chain_config.is_testnet {
        base_gas / 2
    } else {
        base_gas
    };
    adjusted_gas
}

/// Get current gas price for a chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
///
/// # Returns
/// * `f64` - Current gas price in gwei
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// let gas_price = chain::get_gas_price(1);
/// ```
pub fn get_gas_price(chain_id: i64) -> f64 {
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => return 0.0,
    };

    #[cfg(feature = "http-interface")]
    if let Ok(result) = rpc::rpc_request(&chain_config.rpc_url, "eth_gasPrice", vec![]) {
        if let Some(hex_str) = result.as_str() {
            let gwei = rpc::hex_gas_price_to_gwei(hex_str);
            if gwei > 0.0 {
                return gwei;
            }
        }
    }

    // Fallback: config gas price with small variation when RPC unavailable
    let variation = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        % 100) as f64
        / 100.0;
    chain_config.gas_price + variation
}

/// Get block timestamp for a chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
///
/// # Returns
/// * `i64` - Current block timestamp
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// let timestamp = chain::get_block_timestamp(1);
/// ```
pub fn get_block_timestamp(chain_id: i64) -> i64 {
    let _chain_config = get_chain_config(chain_id);

    // Return current timestamp
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Get transaction status
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `tx_hash` - Transaction hash
///
/// # Returns
/// * `String` - Transaction status
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// let status = chain::get_transaction_status(1, "0x1234...".to_string());
/// ```
pub fn get_transaction_status(chain_id: i64, tx_hash: String) -> String {
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => {
            return if tx_hash.starts_with("0x") && tx_hash.len() > 10 {
                "confirmed".to_string()
            } else {
                "pending".to_string()
            };
        }
    };

    #[cfg(feature = "http-interface")]
    {
        use serde_json::json;
        let hash = if tx_hash.starts_with("0x") {
            tx_hash.clone()
        } else {
            format!("0x{}", tx_hash)
        };
        if let Ok(result) = rpc::rpc_request(
            &chain_config.rpc_url,
            "eth_getTransactionReceipt",
            vec![json!(hash)],
        ) {
            if result.is_null() {
                return "pending".to_string();
            }
            if let Some(status) = result.get("status").and_then(|v| v.as_str()) {
                return match status {
                    "0x1" => "confirmed".to_string(),
                    "0x0" => "failed".to_string(),
                    _ => "pending".to_string(),
                };
            }
        }
    }

    if tx_hash.starts_with("0x") && tx_hash.len() > 10 {
        "confirmed".to_string()
    } else {
        "pending".to_string()
    }
}

/// Get balance of an address on a specific chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `address` - The address to check
///
/// # Returns
/// * `i64` - Balance in wei
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// let balance = chain::get_balance(1, "0x1234...".to_string());
/// ```
pub fn get_balance(chain_id: i64, address: String) -> i64 {
    let chain_config = get_chain_config(chain_id);

    #[cfg(feature = "http-interface")]
    if let Some(ref config) = chain_config {
        use serde_json::json;
        let addr = if address.starts_with("0x") {
            address.clone()
        } else {
            format!("0x{}", address)
        };
        if let Ok(result) = rpc::rpc_request(
            &config.rpc_url,
            "eth_getBalance",
            vec![json!(addr), json!("latest")],
        ) {
            if let Some(hex_str) = result.as_str() {
                return rpc::hex_to_i64(hex_str);
            }
        }
    }

    // Fallback: mock balance when RPC unavailable or chain not supported
    if let Some(_) = chain_config {
        if address.starts_with("0x") {
            let hash_sum: i64 = address
                .chars()
                .filter(|c| c.is_ascii_hexdigit())
                .map(|c| c.to_digit(16).unwrap_or(0) as i64)
                .sum();
            let limited_sum = hash_sum % 1000;
            let wei_per_unit: i64 = 1000000000000000;
            return limited_sum.checked_mul(wei_per_unit).unwrap_or(0);
        }
    }
    0
}

/// Call a contract function on a specific chain
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `contract_address` - Contract address
/// * `function_name` - Function to call
/// * `args` - Function arguments
///
/// # Returns
/// * `String` - Function result
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// use std::collections::HashMap;
/// let mut args = HashMap::new();
/// args.insert("to".to_string(), "0x5678...".to_string());
/// args.insert("amount".to_string(), "1000000000000000000".to_string());
/// let result = chain::call(1, "0x1234...".to_string(), "transfer".to_string(), args);
/// ```
pub fn call(
    chain_id: i64,
    contract_address: String,
    function_name: String,
    args: HashMap<String, String>,
) -> String {
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => return "error: chain not supported".to_string(),
    };

    crate::stdlib::log::audit(
        "contract_call",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("chain_id".to_string(), Value::Int(chain_id));
            data.insert(
                "contract_address".to_string(),
                Value::String(contract_address.clone()),
            );
            data.insert(
                "function_name".to_string(),
                Value::String(function_name.clone()),
            );
            data.insert("args".to_string(), Value::String(format!("{:?}", args)));
            data
        },
        Some("chain"),
    );

    #[cfg(feature = "http-interface")]
    if let Some(call_data) = args.get("data").or_else(|| args.get("calldata")) {
        use serde_json::json;
        let to = if contract_address.starts_with("0x") {
            contract_address.clone()
        } else {
            format!("0x{}", contract_address)
        };
        let data_hex = if call_data.starts_with("0x") {
            call_data.clone()
        } else {
            format!("0x{}", call_data)
        };
        let tx = json!({ "to": to, "data": data_hex });
        if let Ok(result) =
            rpc::rpc_request(&chain_config.rpc_url, "eth_call", vec![tx, json!("latest")])
        {
            if let Some(hex_str) = result.as_str() {
                return hex_str.to_string();
            }
        }
    }

    format!(
        "success: {} called on {} at {}",
        function_name, contract_address, chain_config.name
    )
}

/// Mint a new asset or token
///
/// # Arguments
/// * `name` - The name of the asset
/// * `metadata` - Additional metadata for the asset
///
/// # Returns
/// * `i64` - The generated ID for the minted asset
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// use std::collections::HashMap;
/// let mut metadata = HashMap::new();
/// metadata.insert("description".to_string(), "A unique NFT".to_string());
/// metadata.insert("image".to_string(), "ipfs://...".to_string());
/// let asset_id = chain::mint("MyNFT".to_string(), metadata);
/// ```
pub fn mint(name: String, metadata: HashMap<String, String>) -> i64 {
    // Log the mint operation for audit purposes
    crate::stdlib::log::audit(
        "mint",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("name".to_string(), Value::String(name.clone()));
            data.insert(
                "metadata".to_string(),
                Value::String(format!("{:?}", metadata)),
            );
            data
        },
        Some("chain"),
    );

    // Unique ID: hash(name + metadata + timestamp) for uniqueness; fit to i64
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let mut combined = name.as_bytes().to_vec();
    combined.extend(format!("{:?}", metadata).as_bytes());
    combined.extend(timestamp.to_be_bytes());
    let hash = md5::compute(&combined);
    let hash_val: i64 = (hash[0] as i64)
        .wrapping_shl(24)
        .wrapping_add((hash[1] as i64).wrapping_shl(16))
        .wrapping_add((hash[2] as i64).wrapping_shl(8))
        .wrapping_add(hash[3] as i64);
    let asset_id = timestamp
        .abs()
        .wrapping_mul(10000)
        .wrapping_add(hash_val & 0x7FFF_FFFF);

    // Log success
    crate::stdlib::log::info(
        "mint_success",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("asset_id".to_string(), Value::Int(asset_id));
            data
        },
        None,
    );

    asset_id
}

/// Update an existing asset or token
///
/// # Arguments
/// * `asset_id` - The ID of the asset to update
/// * `updates` - The updates to apply to the asset
///
/// # Returns
/// * `bool` - True if update was successful, false otherwise
///
/// # Example
/// ```rust
/// use dist_agent_lang::stdlib::chain;
/// use std::collections::HashMap;
/// let mut updates = HashMap::new();
/// updates.insert("description".to_string(), "Updated description".to_string());
/// let success = chain::update(12345, updates);
/// ```
pub fn update(asset_id: i64, updates: HashMap<String, String>) -> bool {
    crate::stdlib::log::audit(
        "update",
        {
            let mut data = std::collections::HashMap::new();
            data.insert("asset_id".to_string(), Value::Int(asset_id));
            data.insert(
                "updates".to_string(),
                Value::String(format!("{:?}", updates)),
            );
            data
        },
        Some("chain"),
    );

    #[cfg(feature = "http-interface")]
    if let Some(raw_hex) = updates
        .get("raw_transaction")
        .or_else(|| updates.get("signed_tx"))
    {
        let chain_id = updates
            .get("chain_id")
            .and_then(|s| s.trim().parse::<i64>().ok())
            .or_else(|| {
                env::var("CHAIN_ASSET_CHAIN_ID")
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
            });
        if let Some(chain_id) = chain_id {
            if let Some(config) = get_chain_config(chain_id) {
                let tx_hex = if raw_hex.starts_with("0x") {
                    raw_hex.clone()
                } else {
                    format!("0x{}", raw_hex)
                };
                if let Ok(result) = rpc::rpc_request(
                    &config.rpc_url,
                    "eth_sendRawTransaction",
                    vec![serde_json::json!(tx_hex)],
                ) {
                    if let Some(s) = result.as_str() {
                        if s.len() > 2 {
                            return true;
                        }
                    }
                }
            }
        }
    }

    let success = asset_id > 0;

    if success {
        // Log success
        crate::stdlib::log::info(
            "update_success",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("asset_id".to_string(), Value::Int(asset_id));
                data
            },
            None,
        );
    } else {
        // Log failure
        crate::stdlib::log::info(
            "update_failed",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("asset_id".to_string(), Value::Int(asset_id));
                data.insert(
                    "reason".to_string(),
                    Value::String("Invalid asset ID".to_string()),
                );
                data
            },
            None,
        );
    }

    success
}

/// Get asset information. When CHAIN_ASSET_CHAIN_ID and CHAIN_ASSET_CONTRACT are set and
/// http-interface is enabled, calls tokenURI(asset_id) on the contract and adds token_uri to metadata.
pub fn get(asset_id: i64) -> HashMap<String, String> {
    let mut asset_info = HashMap::new();
    asset_info.insert("id".to_string(), asset_id.to_string());
    asset_info.insert("name".to_string(), format!("Asset_{}", asset_id));
    asset_info.insert("created_at".to_string(), (asset_id / 10000).to_string());
    asset_info.insert("status".to_string(), "active".to_string());

    #[cfg(feature = "http-interface")]
    if let (Ok(chain_id_str), Ok(contract)) = (
        env::var("CHAIN_ASSET_CHAIN_ID"),
        env::var("CHAIN_ASSET_CONTRACT"),
    ) {
        if let Ok(chain_id) = chain_id_str.trim().parse::<i64>() {
            if let Some(config) = get_chain_config(chain_id) {
                let token_id = if asset_id < 0 { 0u64 } else { asset_id as u64 };
                let data = format!("0xc87b56dd{:064x}", token_id); // tokenURI(uint256)
                let to = if contract.starts_with("0x") {
                    contract.clone()
                } else {
                    format!("0x{}", contract)
                };
                let tx = serde_json::json!({ "to": to, "data": data });
                if let Ok(result) = rpc::rpc_request(
                    &config.rpc_url,
                    "eth_call",
                    vec![tx, serde_json::json!("latest")],
                ) {
                    if let Some(hex_str) = result.as_str() {
                        asset_info.insert("token_uri_result".to_string(), hex_str.to_string());
                    }
                }
            }
        }
    }

    let mut metadata = HashMap::new();
    metadata.insert("description".to_string(), "A blockchain asset".to_string());
    metadata.insert("version".to_string(), "1".to_string());
    asset_info.insert("metadata".to_string(), format!("{:?}", metadata));
    asset_info
}

/// Check if an asset exists. When CHAIN_ASSET_CHAIN_ID and CHAIN_ASSET_CONTRACT are set and
/// http-interface is enabled, calls ownerOf(asset_id); if non-zero address, returns true.
pub fn exists(asset_id: i64) -> bool {
    #[cfg(feature = "http-interface")]
    if let (Ok(chain_id_str), Ok(contract)) = (
        env::var("CHAIN_ASSET_CHAIN_ID"),
        env::var("CHAIN_ASSET_CONTRACT"),
    ) {
        if let Ok(chain_id) = chain_id_str.trim().parse::<i64>() {
            if let Some(config) = get_chain_config(chain_id) {
                let token_id = if asset_id < 0 { 0u64 } else { asset_id as u64 };
                let data = format!("0x6352211e{:064x}", token_id); // ownerOf(uint256)
                let to = if contract.starts_with("0x") {
                    contract.clone()
                } else {
                    format!("0x{}", contract)
                };
                let tx = serde_json::json!({ "to": to, "data": data });
                if let Ok(result) = rpc::rpc_request(
                    &config.rpc_url,
                    "eth_call",
                    vec![tx, serde_json::json!("latest")],
                ) {
                    if let Some(hex_str) = result.as_str() {
                        let s = hex_str
                            .strip_prefix("0x")
                            .unwrap_or(hex_str)
                            .trim_start_matches('0');
                        if !s.is_empty() && s != "0" {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
    }
    asset_id > 0
}
