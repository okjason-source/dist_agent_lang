use crate::runtime::values::Value;
use crate::stdlib::abi;
use std::collections::HashMap;
use std::env;

/// Chain namespace for comprehensive blockchain operations
/// Provides multi-chain support with deployment, interaction, and monitoring

fn env_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Strict production policy for chain interactions.
///
/// Enabled when either:
/// - DAL_CHAIN_STRICT is truthy (1/true/yes/on), or
/// - DAL_CHAIN_STRICT_FROM_TRUST_MODE is truthy and DAL_COMPILE_TRUST_MODE == "decentralized"
fn strict_chain_policy_enabled() -> bool {
    if let Ok(v) = env::var("DAL_CHAIN_STRICT") {
        return env_truthy(&v);
    }
    if let Ok(v) = env::var("DAL_CHAIN_STRICT_FROM_TRUST_MODE") {
        if env_truthy(&v) {
            if let Ok(mode) = env::var("DAL_COMPILE_TRUST_MODE") {
                if mode.trim().eq_ignore_ascii_case("decentralized") {
                    return true;
                }
            }
        }
    }
    false
}

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

        // Arbitrum One
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

        // Avalanche C-Chain
        m.insert(43114, ChainConfig {
            chain_id: 43114,
            name: "Avalanche C-Chain".to_string(),
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            explorer: "https://snowtrace.io".to_string(),
            gas_limit: 21000,
            gas_price: 25.0,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypedDeployRequest {
    raw_transaction: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypedCallRequest {
    to: String,
    data: String,
    /// Optional Solidity signature hint for return decoding (e.g. `balanceOf(address)`).
    function_signature: Option<String>,
}

/// Stable, machine-parseable failure taxonomy for typed chain deploy/call surfaces.
///
/// These values are exposed on [`ChainDeployResult::error_code`] / [`ChainCallResult::error_code`]
/// as stable uppercase tokens (e.g. `RPC_FAILURE`). They are intended for agents and tests;
/// human-readable detail remains in `message` / evidence fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum TypedChainErrorCode {
    /// Policy denied the operation (strict mode or trust configuration).
    StrictPolicy,
    /// Required input fields were absent (e.g. calldata or signed deploy payload).
    MissingRequiredField,
    /// Chain id is not registered / supported.
    ChainUnsupported,
    /// JSON-RPC or transport failed after a typed request was formed.
    RpcFailure,
    /// Transaction receipt reports failure (`reverted`) while an address may still be present.
    TxReverted,
    /// RPC returned bytes but ABI decoding produced a warning (`decode_error` is set).
    AbiDecodeFailed,
}

impl TypedChainErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            TypedChainErrorCode::StrictPolicy => "STRICT_POLICY",
            TypedChainErrorCode::MissingRequiredField => "MISSING_REQUIRED_FIELD",
            TypedChainErrorCode::ChainUnsupported => "CHAIN_UNSUPPORTED",
            TypedChainErrorCode::RpcFailure => "RPC_FAILURE",
            TypedChainErrorCode::TxReverted => "TX_REVERTED",
            TypedChainErrorCode::AbiDecodeFailed => "ABI_DECODE_FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypedDeployResponse {
    contract_address: Option<String>,
    tx_hash: Option<String>,
    receipt_status: Option<String>,
    revert_data: Option<String>,
    error_code: Option<TypedChainErrorCode>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypedCallResponse {
    result_hex: Option<String>,
    decoded: Option<String>,
    decode_error: Option<String>,
    tx_hash: Option<String>,
    receipt_status: Option<String>,
    revert_data: Option<String>,
    error_code: Option<TypedChainErrorCode>,
    message: String,
}

impl TypedDeployResponse {
    fn success(contract_address: String) -> Self {
        Self {
            contract_address: Some(contract_address),
            tx_hash: None,
            receipt_status: None,
            revert_data: None,
            error_code: None,
            message: "ok".to_string(),
        }
    }

    fn success_with_evidence(
        contract_address: String,
        tx_hash: Option<String>,
        receipt_status: Option<String>,
        revert_data: Option<String>,
    ) -> Self {
        let error_code = if receipt_status.as_deref() == Some("reverted") {
            Some(TypedChainErrorCode::TxReverted)
        } else {
            None
        };
        Self {
            contract_address: Some(contract_address),
            tx_hash,
            receipt_status,
            revert_data,
            error_code,
            message: "ok".to_string(),
        }
    }

    fn error(error_code: TypedChainErrorCode, message: String) -> Self {
        Self {
            contract_address: None,
            tx_hash: None,
            receipt_status: None,
            revert_data: None,
            error_code: Some(error_code),
            message,
        }
    }

    fn into_legacy_string(self) -> String {
        if let Some(address) = self.contract_address {
            return address;
        }
        self.message
    }
}

impl TypedCallResponse {
    fn success_with_evidence(
        result_hex: String,
        decoded: Option<String>,
        decode_error: Option<String>,
        tx_hash: Option<String>,
        receipt_status: Option<String>,
        revert_data: Option<String>,
    ) -> Self {
        let error_code = if decode_error.is_some() {
            Some(TypedChainErrorCode::AbiDecodeFailed)
        } else {
            None
        };
        Self {
            result_hex: Some(result_hex),
            decoded,
            decode_error,
            tx_hash,
            receipt_status,
            revert_data,
            error_code,
            message: "ok".to_string(),
        }
    }

    fn error(error_code: TypedChainErrorCode, message: String) -> Self {
        Self {
            result_hex: None,
            decoded: None,
            decode_error: None,
            tx_hash: None,
            receipt_status: None,
            revert_data: None,
            error_code: Some(error_code),
            message,
        }
    }

    fn passthrough_message(message: String) -> Self {
        Self {
            result_hex: None,
            decoded: None,
            decode_error: None,
            tx_hash: None,
            receipt_status: None,
            revert_data: None,
            error_code: None,
            message,
        }
    }

    fn into_legacy_string(self) -> String {
        if let Some(result_hex) = self.result_hex {
            return result_hex;
        }
        self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainDeployResult {
    pub contract_address: Option<String>,
    pub tx_hash: Option<String>,
    pub receipt_status: Option<String>,
    pub revert_data: Option<String>,
    /// Machine-parseable taxonomy token when the operation did not fully succeed
    /// (e.g. `CHAIN_UNSUPPORTED`, `MISSING_REQUIRED_FIELD`, `RPC_FAILURE`, `TX_REVERTED`).
    pub error_code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainCallResult {
    pub result_hex: Option<String>,
    pub decoded: Option<String>,
    pub decode_error: Option<String>,
    pub tx_hash: Option<String>,
    pub receipt_status: Option<String>,
    pub revert_data: Option<String>,
    /// Machine-parseable taxonomy token when applicable
    /// (e.g. `CHAIN_UNSUPPORTED`, `MISSING_REQUIRED_FIELD`, `RPC_FAILURE`, `ABI_DECODE_FAILED`).
    pub error_code: Option<String>,
    pub message: String,
}

impl ChainDeployResult {
    fn from_internal(internal: TypedDeployResponse) -> Self {
        Self {
            contract_address: internal.contract_address,
            tx_hash: internal.tx_hash,
            receipt_status: internal.receipt_status,
            revert_data: internal.revert_data,
            error_code: internal.error_code.map(|e| e.as_str().to_string()),
            message: internal.message,
        }
    }

    /// Legacy string projection for `chain::deploy(...)` callers.
    pub fn into_legacy_string(self) -> String {
        self.contract_address.unwrap_or(self.message)
    }

    /// Convert to a runtime `Value::Map` for `chain::deploy_typed(...)`.
    ///
    /// The provenance contract: every map produced by this method contains exactly these keys
    /// (present as `Value::Null` when no data is available):
    /// `contract_address`, `tx_hash`, `receipt_status`, `revert_data`, `error_code`, `message`.
    pub fn to_value_map(self) -> HashMap<String, Value> {
        let mut out = HashMap::new();
        out.insert(
            "contract_address".to_string(),
            self.contract_address
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        out.insert(
            "tx_hash".to_string(),
            self.tx_hash.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "receipt_status".to_string(),
            self.receipt_status
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        out.insert(
            "revert_data".to_string(),
            self.revert_data.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "error_code".to_string(),
            self.error_code.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert("message".to_string(), Value::String(self.message));
        out
    }

    /// True when there is evidence worth auditing (at least one provenance field set).
    pub fn has_evidence(&self) -> bool {
        self.tx_hash.is_some()
            || self.receipt_status.is_some()
            || self.revert_data.is_some()
            || self.error_code.is_some()
    }

    /// Build an audit-log evidence map for `crate::stdlib::log::audit(...)`.
    ///
    /// Includes `chain_id`, `contract_name`, and `result` as context, plus any provenance fields
    /// that are present.
    pub fn to_audit_evidence(&self, chain_id: i64, contract_name: &str) -> HashMap<String, Value> {
        let mut evidence = HashMap::new();
        evidence.insert("chain_id".to_string(), Value::Int(chain_id));
        evidence.insert(
            "contract_name".to_string(),
            Value::String(contract_name.to_string()),
        );
        let result_str = self
            .contract_address
            .clone()
            .unwrap_or_else(|| self.message.clone());
        evidence.insert("result".to_string(), Value::String(result_str));
        if let Some(code) = &self.error_code {
            evidence.insert("error_code".to_string(), Value::String(code.clone()));
        }
        if let Some(tx_hash) = &self.tx_hash {
            evidence.insert("tx_hash".to_string(), Value::String(tx_hash.clone()));
        }
        if let Some(status) = &self.receipt_status {
            evidence.insert("receipt_status".to_string(), Value::String(status.clone()));
        }
        if let Some(revert_data) = &self.revert_data {
            evidence.insert(
                "revert_data".to_string(),
                Value::String(revert_data.clone()),
            );
        }
        evidence
    }
}

impl ChainCallResult {
    fn from_internal(internal: TypedCallResponse) -> Self {
        Self {
            result_hex: internal.result_hex,
            decoded: internal.decoded,
            decode_error: internal.decode_error,
            tx_hash: internal.tx_hash,
            receipt_status: internal.receipt_status,
            revert_data: internal.revert_data,
            error_code: internal.error_code.map(|e| e.as_str().to_string()),
            message: internal.message,
        }
    }

    /// Legacy string projection for `chain::call(...)` callers.
    pub fn into_legacy_string(self) -> String {
        self.result_hex.unwrap_or(self.message)
    }

    /// Convert to a runtime `Value::Map` for `chain::call_typed(...)`.
    ///
    /// The provenance contract: every map produced by this method contains exactly these keys
    /// (present as `Value::Null` when no data is available):
    /// `result_hex`, `decoded`, `decode_error`, `tx_hash`, `receipt_status`, `revert_data`,
    /// `error_code`, `message`.
    pub fn to_value_map(self) -> HashMap<String, Value> {
        let mut out = HashMap::new();
        out.insert(
            "result_hex".to_string(),
            self.result_hex.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "decoded".to_string(),
            self.decoded.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "decode_error".to_string(),
            self.decode_error.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "tx_hash".to_string(),
            self.tx_hash.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "receipt_status".to_string(),
            self.receipt_status
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        out.insert(
            "revert_data".to_string(),
            self.revert_data.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert(
            "error_code".to_string(),
            self.error_code.map(Value::String).unwrap_or(Value::Null),
        );
        out.insert("message".to_string(), Value::String(self.message));
        out
    }

    /// True when there is evidence worth auditing (at least one provenance field set).
    pub fn has_evidence(&self) -> bool {
        self.tx_hash.is_some()
            || self.receipt_status.is_some()
            || self.revert_data.is_some()
            || self.error_code.is_some()
            || self.decode_error.is_some()
    }

    /// Build an audit-log evidence map for `crate::stdlib::log::audit(...)`.
    ///
    /// Includes `chain_id`, `contract_address`, `function_name`, and `result` as context,
    /// plus any provenance fields that are present.
    pub fn to_audit_evidence(
        &self,
        chain_id: i64,
        contract_address: &str,
        function_name: &str,
    ) -> HashMap<String, Value> {
        let mut evidence = HashMap::new();
        evidence.insert("chain_id".to_string(), Value::Int(chain_id));
        evidence.insert(
            "contract_address".to_string(),
            Value::String(contract_address.to_string()),
        );
        evidence.insert(
            "function_name".to_string(),
            Value::String(function_name.to_string()),
        );
        let result_str = self
            .result_hex
            .clone()
            .unwrap_or_else(|| self.message.clone());
        evidence.insert("result".to_string(), Value::String(result_str));
        if let Some(code) = &self.error_code {
            evidence.insert("error_code".to_string(), Value::String(code.clone()));
        }
        if let Some(tx_hash) = &self.tx_hash {
            evidence.insert("tx_hash".to_string(), Value::String(tx_hash.clone()));
        }
        if let Some(status) = &self.receipt_status {
            evidence.insert("receipt_status".to_string(), Value::String(status.clone()));
        }
        if let Some(revert_data) = &self.revert_data {
            evidence.insert(
                "revert_data".to_string(),
                Value::String(revert_data.clone()),
            );
        }
        if let Some(decoded) = &self.decoded {
            evidence.insert("decoded".to_string(), Value::String(decoded.clone()));
        }
        if let Some(decode_err) = &self.decode_error {
            evidence.insert(
                "decode_error".to_string(),
                Value::String(decode_err.clone()),
            );
        }
        evidence
    }
}

fn typed_deploy_request_from_args(
    args: &HashMap<String, String>,
) -> Result<TypedDeployRequest, String> {
    let raw_tx = args
        .get("raw_transaction")
        .or_else(|| args.get("signed_tx"))
        .ok_or_else(|| "missing required deploy field: raw_transaction or signed_tx".to_string())?;
    let normalized = if raw_tx.starts_with("0x") {
        raw_tx.clone()
    } else {
        format!("0x{}", raw_tx)
    };
    Ok(TypedDeployRequest {
        raw_transaction: normalized,
    })
}

fn typed_call_request_from_args(
    contract_address: &str,
    args: &HashMap<String, String>,
) -> Result<TypedCallRequest, String> {
    let call_data = args
        .get("data")
        .or_else(|| args.get("calldata"))
        .ok_or_else(|| "missing required call field: data or calldata".to_string())?;
    let to = if contract_address.starts_with("0x") {
        contract_address.to_string()
    } else {
        format!("0x{}", contract_address)
    };
    let data = if call_data.starts_with("0x") {
        call_data.clone()
    } else {
        format!("0x{}", call_data)
    };
    let function_signature = args.get("function_signature").cloned();
    Ok(TypedCallRequest {
        to,
        data,
        function_signature,
    })
}

/// Typed deploy arguments for [`deploy_typed`] / [`deploy_typed_with_args`].
///
/// Makes the required signed-transaction payload explicit at the type level while
/// preserving a catch-all `extra` map for forward-compatible kwargs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainDeployArgs {
    /// Hex-encoded signed transaction payload (normalized to `0x` prefix).
    pub raw_transaction: String,
    /// Additional kwargs forwarded to the deploy pipeline.
    pub extra: HashMap<String, String>,
}

impl ChainDeployArgs {
    /// Parse from a flat string map, validating required fields and normalizing hex.
    ///
    /// Accepts `raw_transaction` or `signed_tx` as the payload key.
    pub fn from_map(mut map: HashMap<String, String>) -> Result<Self, String> {
        let raw_tx = map
            .remove("raw_transaction")
            .or_else(|| map.remove("signed_tx"))
            .ok_or_else(|| {
                "missing required deploy field: raw_transaction or signed_tx".to_string()
            })?;
        let normalized = if raw_tx.starts_with("0x") {
            raw_tx
        } else {
            format!("0x{}", raw_tx)
        };
        Ok(Self {
            raw_transaction: normalized,
            extra: map,
        })
    }

    fn to_flat_map(self) -> HashMap<String, String> {
        let mut m = self.extra;
        m.insert("raw_transaction".to_string(), self.raw_transaction);
        m
    }
}

/// Typed call arguments for [`call_typed`] / [`call_typed_with_args`].
///
/// Makes the required calldata explicit at the type level, surfaces the optional
/// `function_signature` decode hint, and preserves a catch-all `extra` map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainCallArgs {
    /// Hex-encoded calldata (normalized to `0x` prefix).
    pub data: String,
    /// Optional Solidity signature hint for ABI return decoding (e.g. `balanceOf(address)`).
    pub function_signature: Option<String>,
    /// Additional kwargs forwarded to the call pipeline.
    pub extra: HashMap<String, String>,
}

impl ChainCallArgs {
    /// Parse from a flat string map, validating required fields and normalizing hex.
    ///
    /// Accepts `data` or `calldata` as the calldata key.
    pub fn from_map(mut map: HashMap<String, String>) -> Result<Self, String> {
        let call_data = map
            .remove("data")
            .or_else(|| map.remove("calldata"))
            .ok_or_else(|| "missing required call field: data or calldata".to_string())?;
        let data = if call_data.starts_with("0x") {
            call_data
        } else {
            format!("0x{}", call_data)
        };
        let function_signature = map.remove("function_signature");
        Ok(Self {
            data,
            function_signature,
            extra: map,
        })
    }

    fn to_flat_map(self) -> HashMap<String, String> {
        let mut m = self.extra;
        m.insert("data".to_string(), self.data);
        if let Some(sig) = self.function_signature {
            m.insert("function_signature".to_string(), sig);
        }
        m
    }
}

/// Convert a runtime object map (e.g. DAL map literal for `chain::deploy` / `chain::call` kwargs)
/// into the flat string map consumed by [`deploy_typed`], [`call_typed`], and legacy wrappers.
///
/// Coercion rules are implemented by [`Value::to_chain_arg_string`]; this function is the
/// **single** bridge from runtime maps into the chain stdlib so key/value handling stays aligned
/// with typed call/deploy internals.
pub fn chain_arg_map_from_runtime_values(map: &HashMap<String, Value>) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.clone(), v.to_chain_arg_string()))
        .collect()
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
fn deploy_typed_internal(
    chain_id: i64,
    contract_name: String,
    constructor_args: HashMap<String, String>,
) -> TypedDeployResponse {
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
            return TypedDeployResponse::error(
                TypedChainErrorCode::ChainUnsupported,
                String::new(),
            );
        }
    };

    let typed_deploy_request = typed_deploy_request_from_args(&constructor_args).ok();

    #[cfg(feature = "http-interface")]
    if let Some(request) = typed_deploy_request.as_ref() {
        match deploy_via_raw_transaction(&chain_config.rpc_url, &request.raw_transaction) {
            Ok((addr, tx_hash, receipt_status, revert_data)) => {
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
                return TypedDeployResponse::success_with_evidence(
                    addr,
                    Some(tx_hash),
                    receipt_status,
                    revert_data,
                );
            }
            Err(rpc_err) if strict_chain_policy_enabled() => {
                crate::stdlib::log::error(
                    "deploy",
                    {
                        let mut data = std::collections::HashMap::new();
                        data.insert("chain_id".to_string(), Value::Int(chain_id));
                        data.insert("contract_name".to_string(), Value::String(contract_name));
                        data.insert("error".to_string(), Value::String(rpc_err.clone()));
                        data
                    },
                    Some("chain"),
                );
                return TypedDeployResponse::error(
                    TypedChainErrorCode::RpcFailure,
                    format!("error: RPC failure during deploy: {}", rpc_err),
                );
            }
            Err(_) => {}
        }
    }

    if strict_chain_policy_enabled() {
        crate::stdlib::log::error(
            "deploy",
            {
                let mut data = std::collections::HashMap::new();
                data.insert("chain_id".to_string(), Value::Int(chain_id));
                data.insert("contract_name".to_string(), Value::String(contract_name));
                data.insert(
                    "error".to_string(),
                    Value::String(
                        "strict chain policy: deploy requires raw_transaction or signed_tx"
                            .to_string(),
                    ),
                );
                data
            },
            Some("chain"),
        );
        return TypedDeployResponse::error(
            TypedChainErrorCode::MissingRequiredField,
            "error: strict chain policy requires raw_transaction or signed_tx for deploy"
                .to_string(),
        );
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

    TypedDeployResponse::success(address)
}

pub fn deploy(
    chain_id: i64,
    contract_name: String,
    constructor_args: HashMap<String, String>,
) -> String {
    deploy_typed_internal(chain_id, contract_name, constructor_args).into_legacy_string()
}

pub fn deploy_typed(
    chain_id: i64,
    contract_name: String,
    constructor_args: HashMap<String, String>,
) -> ChainDeployResult {
    ChainDeployResult::from_internal(deploy_typed_internal(
        chain_id,
        contract_name,
        constructor_args,
    ))
}

/// Typed-args deploy: accepts a [`ChainDeployArgs`] instead of a raw `HashMap`.
///
/// The struct makes the required `raw_transaction` field explicit at the type level,
/// removing the ambiguity of which map keys are mandatory.
pub fn deploy_typed_with_args(
    chain_id: i64,
    contract_name: String,
    args: ChainDeployArgs,
) -> ChainDeployResult {
    deploy_typed(chain_id, contract_name, args.to_flat_map())
}

#[cfg(feature = "http-interface")]
fn deploy_via_raw_transaction(
    rpc_url: &str,
    raw_tx_hex: &str,
) -> Result<(String, String, Option<String>, Option<String>), String> {
    use serde_json::json;
    let tx_hex = raw_tx_hex.strip_prefix("0x").unwrap_or(raw_tx_hex);
    let result = rpc::rpc_request(
        rpc_url,
        "eth_sendRawTransaction",
        vec![json!(format!("0x{}", tx_hex))],
    )?;
    let tx_hash = result
        .as_str()
        .ok_or("expected tx hash string")?
        .to_string();
    let receipt = wait_for_receipt(rpc_url, &tx_hash, 30)?;
    let addr = receipt
        .get("contractAddress")
        .and_then(|v| v.as_str())
        .ok_or("no contractAddress in receipt")?
        .to_string();
    let receipt_status = receipt
        .get("status")
        .and_then(|v| v.as_str())
        .map(|status_hex| {
            let normalized = status_hex.trim().to_ascii_lowercase();
            if normalized == "0x1" || normalized == "1" {
                "success".to_string()
            } else if normalized == "0x0" || normalized == "0" {
                "reverted".to_string()
            } else {
                "unknown".to_string()
            }
        });
    let revert_data = receipt
        .get("revertData")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok((addr, tx_hash, receipt_status, revert_data))
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
        None => {
            if strict_chain_policy_enabled() {
                return -1;
            }
            return 0;
        }
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

    if strict_chain_policy_enabled() {
        return -1;
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
    if chain_config.is_testnet {
        base_gas / 2
    } else {
        base_gas
    }
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
    #[cfg(feature = "http-interface")]
    if let Some(config) = get_chain_config(chain_id) {
        use serde_json::json;
        if let Ok(result) = rpc::rpc_request(
            &config.rpc_url,
            "eth_getBlockByNumber",
            vec![json!("latest"), json!(false)],
        ) {
            if let Some(hex_ts) = result.get("timestamp").and_then(|v| v.as_str()) {
                let ts = rpc::hex_to_i64(hex_ts);
                if ts > 0 {
                    return ts;
                }
            }
        }
    }

    if strict_chain_policy_enabled() {
        return -1;
    }

    // Fallback: system time when RPC unavailable or not configured
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Get latest block hash for a chain (example-only API).
///
/// # Arguments
/// * `chain_id` - The target chain ID
///
/// # Returns
/// * `String` - Block hash (0x-prefixed hex), or a placeholder when RPC unavailable
pub fn get_block_hash(chain_id: i64) -> String {
    #[cfg(feature = "http-interface")]
    if let Some(config) = get_chain_config(chain_id) {
        use serde_json::json;
        if let Ok(result) = rpc::rpc_request(
            &config.rpc_url,
            "eth_getBlockByNumber",
            vec![json!("latest"), json!(false)],
        ) {
            if let Some(hash) = result.get("hash").and_then(|v| v.as_str()) {
                return hash.to_string();
            }
        }
    }
    if strict_chain_policy_enabled() {
        return "error: strict chain policy requires RPC-backed block hash".to_string();
    }
    // Placeholder when RPC unavailable
    format!(
        "0x{:064x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
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
            if strict_chain_policy_enabled() {
                return "error: strict chain policy requires supported chain and RPC receipt"
                    .to_string();
            }
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

    if strict_chain_policy_enabled() {
        return "error: strict chain policy requires RPC transaction receipt".to_string();
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

    if strict_chain_policy_enabled() {
        return -1;
    }

    // Fallback: mock balance when RPC unavailable or chain not supported
    if chain_config.is_some() {
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

/// Known ERC20 contract addresses by symbol (mainnet chain_id 1). Used when token_symbol is not 0x.
fn erc20_contract_for_symbol(chain_id: i64, symbol: &str) -> Option<String> {
    if chain_id != 1 {
        return None;
    }
    let s = symbol.to_uppercase();
    let addr = match s.as_str() {
        "USDC" => "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "USDT" => "0xdac17f958d2ee523a2206206994597c13d831ec7",
        "WETH" => "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
        "DAI" => "0x6b175474e89094c44da98b954eedeac495271d0f",
        "WBTC" => "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599",
        _ => return None,
    };
    Some(format!("0x{}", addr))
}

fn build_erc20_balance_of_calldata(address: &str) -> String {
    let addr_hex = address.strip_prefix("0x").unwrap_or(address).to_lowercase();
    let padded = format!("{:0>64}", addr_hex);
    format!("0x{}{}", abi::SELECTOR_ERC20_BALANCE_OF, padded)
}

#[cfg(test)]
fn encode_address_word(address: &str) -> String {
    let addr_hex = address.strip_prefix("0x").unwrap_or(address).to_lowercase();
    format!("{:0>64}", addr_hex)
}

#[cfg(test)]
fn encode_uint256_word(value: u128) -> String {
    format!("{:064x}", value)
}

#[cfg(test)]
fn encode_bool_word(value: bool) -> String {
    if value {
        encode_uint256_word(1)
    } else {
        encode_uint256_word(0)
    }
}

#[cfg(test)]
fn encode_bytes32_word(value_hex: &str) -> String {
    let stripped = value_hex
        .strip_prefix("0x")
        .unwrap_or(value_hex)
        .to_lowercase();
    assert_eq!(
        stripped.len(),
        64,
        "bytes32 values must be exactly 32 bytes (64 hex chars)"
    );
    assert!(
        stripped.chars().all(|c| c.is_ascii_hexdigit()),
        "bytes32 value contains non-hex characters"
    );
    stripped
}

#[cfg(test)]
fn decode_uint256_word(word_hex: &str) -> Result<u128, String> {
    crate::stdlib::abi_codec::decode_uint256_word(word_hex)
}

#[cfg(test)]
fn decode_bool_word(word_hex: &str) -> Result<bool, String> {
    crate::stdlib::abi_codec::decode_bool_word(word_hex)
}

#[cfg(test)]
fn decode_address_word(word_hex: &str) -> Result<String, String> {
    crate::stdlib::abi_codec::decode_address_word(word_hex)
}

#[cfg(test)]
fn decode_abi_string_data(payload_hex: &str) -> Result<String, String> {
    crate::stdlib::abi_codec::decode_abi_string_data(payload_hex)
}

#[cfg(test)]
fn decode_abi_bytes_data(payload_hex: &str) -> Result<Vec<u8>, String> {
    crate::stdlib::abi_codec::decode_abi_bytes_data(payload_hex)
}

#[cfg(test)]
fn decode_abi_tuple_string_bytes_payload(payload_hex: &str) -> Result<(String, Vec<u8>), String> {
    crate::stdlib::abi_codec::decode_abi_tuple_string_bytes_payload(payload_hex)
}

#[cfg(test)]
fn decode_custom_error_payload_words(
    payload_hex: &str,
    expected_selector_hex: &str,
) -> Result<Vec<String>, String> {
    crate::stdlib::abi_codec::decode_custom_error_payload_words(payload_hex, expected_selector_hex)
}

#[cfg(test)]
fn decode_static_tuple_address_uint_bool_payload(
    payload_hex: &str,
) -> Result<(String, u128, bool), String> {
    crate::stdlib::abi_codec::decode_static_tuple_address_uint_bool_payload(payload_hex)
}

#[cfg(test)]
fn decode_revert_error_string_payload(payload_hex: &str) -> Result<String, String> {
    crate::stdlib::abi_codec::decode_revert_error_string_payload(payload_hex)
}

#[cfg(test)]
fn decode_revert_panic_code_payload(payload_hex: &str) -> Result<u128, String> {
    crate::stdlib::abi_codec::decode_revert_panic_code_payload(payload_hex)
}

#[cfg(test)]
fn build_revert_error_payload(message: &str) -> String {
    let message_hex = hex::encode(message.as_bytes());
    let padded_hex_len = if message_hex.len() % 64 == 0 {
        message_hex.len()
    } else {
        message_hex.len() + (64 - (message_hex.len() % 64))
    };
    let mut padded_message_hex = message_hex;
    if padded_message_hex.len() < padded_hex_len {
        padded_message_hex.push_str(&"0".repeat(padded_hex_len - padded_message_hex.len()));
    }
    format!(
        "0x{}{}{}{}",
        abi::SELECTOR_ERROR_STRING,
        encode_uint256_word(32),
        encode_uint256_word(message.len() as u128),
        padded_message_hex
    )
}

#[cfg(test)]
fn build_panic_revert_payload(code: u128) -> String {
    format!(
        "0x{}{}",
        abi::SELECTOR_PANIC_UINT256,
        encode_uint256_word(code)
    )
}

#[cfg(test)]
fn build_custom_error_payload(selector_hex: &str, encoded_words: &[String]) -> String {
    let selector = selector_hex.trim().trim_start_matches("0x").to_lowercase();
    let mut out = format!("0x{}", selector);
    for word in encoded_words {
        let normalized = word.trim().trim_start_matches("0x");
        out.push_str(normalized);
    }
    out
}

#[cfg(test)]
fn build_erc20_transfer_calldata(to: &str, amount: u128) -> String {
    format!(
        "0x{}{}{}",
        abi::SELECTOR_ERC20_TRANSFER,
        encode_address_word(to),
        encode_uint256_word(amount)
    )
}

#[cfg(test)]
fn build_erc20_approve_calldata(spender: &str, amount: u128) -> String {
    format!(
        "0x{}{}{}",
        abi::SELECTOR_ERC20_APPROVE,
        encode_address_word(spender),
        encode_uint256_word(amount)
    )
}

#[cfg(test)]
fn build_erc20_transfer_from_calldata(from: &str, to: &str, amount: u128) -> String {
    format!(
        "0x{}{}{}{}",
        abi::SELECTOR_ERC20_TRANSFER_FROM,
        encode_address_word(from),
        encode_address_word(to),
        encode_uint256_word(amount)
    )
}

#[cfg(test)]
fn build_erc20_allowance_calldata(owner: &str, spender: &str) -> String {
    format!(
        "0x{}{}{}",
        abi::SELECTOR_ERC20_ALLOWANCE,
        encode_address_word(owner),
        encode_address_word(spender)
    )
}

#[cfg(test)]
fn build_erc20_total_supply_calldata() -> String {
    format!("0x{}", abi::SELECTOR_ERC20_TOTAL_SUPPLY)
}

#[cfg(test)]
fn build_erc20_decimals_calldata() -> String {
    format!("0x{}", abi::SELECTOR_ERC20_DECIMALS)
}

fn build_erc721_token_uri_calldata(token_id: u64) -> String {
    format!("0x{}{:064x}", abi::SELECTOR_ERC721_TOKEN_URI, token_id)
}

fn build_erc721_owner_of_calldata(token_id: u64) -> String {
    format!("0x{}{:064x}", abi::SELECTOR_ERC721_OWNER_OF, token_id)
}

/// Get ERC20 token balance (example-only API).
///
/// # Arguments
/// * `chain_id` - The target chain ID
/// * `token_symbol_or_contract` - Token symbol (e.g. "USDC") or contract address (0x...)
/// * `address` - Account address to query
///
/// # Returns
/// * `i64` - Token balance (raw units, e.g. 6 decimals for USDC)
pub fn get_token_balance(chain_id: i64, token_symbol_or_contract: String, address: String) -> i64 {
    let contract = if token_symbol_or_contract.starts_with("0x") {
        token_symbol_or_contract.clone()
    } else if let Some(addr) = erc20_contract_for_symbol(chain_id, &token_symbol_or_contract) {
        addr
    } else {
        return 0;
    };

    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => return 0,
    };

    #[cfg(feature = "http-interface")]
    {
        use serde_json::json;
        let data = build_erc20_balance_of_calldata(&address);
        let to = if contract.starts_with("0x") {
            contract
        } else {
            format!("0x{}", contract)
        };
        let tx = json!({ "to": to, "data": data });
        if let Ok(result) =
            rpc::rpc_request(&chain_config.rpc_url, "eth_call", vec![tx, json!("latest")])
        {
            if let Some(hex_str) = result.as_str() {
                return rpc::hex_to_i64(hex_str);
            }
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
fn call_typed_internal(
    chain_id: i64,
    contract_address: String,
    function_name: String,
    args: HashMap<String, String>,
) -> TypedCallResponse {
    let chain_config = match get_chain_config(chain_id) {
        Some(c) => c,
        None => {
            return TypedCallResponse::error(
                TypedChainErrorCode::ChainUnsupported,
                "error: chain not supported".to_string(),
            )
        }
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
    if let Ok(request) = typed_call_request_from_args(&contract_address, &args) {
        use serde_json::json;
        let tx = json!({ "to": request.to, "data": request.data });
        match rpc::rpc_request(&chain_config.rpc_url, "eth_call", vec![tx, json!("latest")]) {
            Ok(result) => {
                if let Some(hex_str) = result.as_str() {
                    let selector_hint = request
                        .data
                        .get(0..10)
                        .map(|s| s.to_string())
                        .filter(|s| s.starts_with("0x"));
                    let (decoded, decode_error) =
                        crate::stdlib::add_sol::decode_registered_result_as_string(
                            chain_id,
                            &contract_address,
                            &function_name,
                            selector_hint.as_deref(),
                            request.function_signature.as_deref(),
                            &Some(hex_str.to_string()),
                        );
                    // `eth_call` has no transaction receipt; evidence uses `None` here.
                    return TypedCallResponse::success_with_evidence(
                        hex_str.to_string(),
                        decoded,
                        decode_error,
                        None,
                        None,
                        None,
                    );
                } else if strict_chain_policy_enabled() {
                    return TypedCallResponse::error(
                        TypedChainErrorCode::RpcFailure,
                        format!(
                            "error: unexpected eth_call RPC result (expected hex string): {:?}",
                            result
                        ),
                    );
                }
            }
            Err(rpc_err) if strict_chain_policy_enabled() => {
                return TypedCallResponse::error(
                    TypedChainErrorCode::RpcFailure,
                    format!("error: RPC failure during eth_call: {}", rpc_err),
                );
            }
            Err(_) => {}
        }
    }

    if strict_chain_policy_enabled() {
        crate::stdlib::log::error(
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
                data.insert(
                    "error".to_string(),
                    Value::String(
                        "strict chain policy: call requires data or calldata".to_string(),
                    ),
                );
                data
            },
            Some("chain"),
        );
        return TypedCallResponse::error(
            TypedChainErrorCode::MissingRequiredField,
            "error: strict chain policy requires data or calldata for call".to_string(),
        );
    }

    TypedCallResponse::passthrough_message(format!(
        "success: {} called on {} at {}",
        function_name, contract_address, chain_config.name
    ))
}

pub fn call(
    chain_id: i64,
    contract_address: String,
    function_name: String,
    args: HashMap<String, String>,
) -> String {
    call_typed_internal(chain_id, contract_address, function_name, args).into_legacy_string()
}

pub fn call_typed(
    chain_id: i64,
    contract_address: String,
    function_name: String,
    args: HashMap<String, String>,
) -> ChainCallResult {
    ChainCallResult::from_internal(call_typed_internal(
        chain_id,
        contract_address,
        function_name,
        args,
    ))
}

/// Typed-args call: accepts a [`ChainCallArgs`] instead of a raw `HashMap`.
///
/// The struct makes the required `data` field and optional `function_signature` hint
/// explicit at the type level, removing map-key ambiguity.
pub fn call_typed_with_args(
    chain_id: i64,
    contract_address: String,
    function_name: String,
    args: ChainCallArgs,
) -> ChainCallResult {
    call_typed(
        chain_id,
        contract_address,
        function_name,
        args.to_flat_map(),
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
                let data = build_erc721_token_uri_calldata(token_id);
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
                let data = build_erc721_owner_of_calldata(token_id);
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

#[cfg(test)]
mod abi_golden_vectors {
    use super::{
        build_custom_error_payload, build_erc20_allowance_calldata, build_erc20_approve_calldata,
        build_erc20_balance_of_calldata, build_erc20_decimals_calldata,
        build_erc20_total_supply_calldata, build_erc20_transfer_calldata,
        build_erc20_transfer_from_calldata, build_erc721_owner_of_calldata,
        build_erc721_token_uri_calldata, build_panic_revert_payload, build_revert_error_payload,
        call_typed, call_typed_with_args, chain_arg_map_from_runtime_values, decode_abi_bytes_data,
        decode_abi_string_data, decode_abi_tuple_string_bytes_payload, decode_address_word,
        decode_bool_word, decode_custom_error_payload_words, decode_revert_error_string_payload,
        decode_revert_panic_code_payload, decode_static_tuple_address_uint_bool_payload,
        decode_uint256_word, deploy_typed, deploy_typed_with_args, encode_address_word,
        encode_bool_word, encode_bytes32_word, encode_uint256_word, typed_call_request_from_args,
        typed_deploy_request_from_args, ChainCallArgs, ChainDeployArgs, TypedCallResponse,
        TypedChainErrorCode, TypedDeployResponse,
    };
    use crate::runtime::values::Value;
    use crate::stdlib::abi::{canonical_selector_catalog, selector_from_signature};

    #[test]
    fn chain_arg_map_from_runtime_values_preserves_chain_kwarg_contract() {
        let mut m = std::collections::HashMap::new();
        m.insert("calldata".into(), Value::String("70a08231".into()));
        m.insert("chain_id".into(), Value::Int(1));
        m.insert("flag".into(), Value::Bool(false));
        m.insert("empty".into(), Value::Null);
        let out = chain_arg_map_from_runtime_values(&m);
        assert_eq!(out.get("calldata").map(String::as_str), Some("70a08231"));
        assert_eq!(out.get("chain_id").map(String::as_str), Some("1"));
        assert_eq!(out.get("flag").map(String::as_str), Some("false"));
        assert_eq!(out.get("empty").map(String::as_str), Some("null"));
    }

    // -- Typed kwargs struct tests --

    #[test]
    fn chain_deploy_args_from_map_requires_signed_payload_and_normalizes_hex() {
        let mut m = std::collections::HashMap::new();
        m.insert("signed_tx".to_string(), "deadbeef".to_string());
        m.insert("gas_limit".to_string(), "21000".to_string());
        let args = ChainDeployArgs::from_map(m).expect("should parse");
        assert_eq!(args.raw_transaction, "0xdeadbeef");
        assert_eq!(
            args.extra.get("gas_limit").map(String::as_str),
            Some("21000")
        );
        assert!(!args.extra.contains_key("signed_tx"));
        assert!(!args.extra.contains_key("raw_transaction"));
    }

    #[test]
    fn chain_deploy_args_from_map_rejects_missing_payload() {
        let m = std::collections::HashMap::new();
        assert!(ChainDeployArgs::from_map(m).is_err());
    }

    #[test]
    fn chain_deploy_args_roundtrips_through_flat_map() {
        let args = ChainDeployArgs {
            raw_transaction: "0xabc".to_string(),
            extra: std::collections::HashMap::new(),
        };
        let flat = args.to_flat_map();
        assert_eq!(
            flat.get("raw_transaction").map(String::as_str),
            Some("0xabc")
        );
    }

    #[test]
    fn chain_call_args_from_map_requires_calldata_and_normalizes_hex() {
        let mut m = std::collections::HashMap::new();
        m.insert("calldata".to_string(), "70a08231".to_string());
        m.insert(
            "function_signature".to_string(),
            "balanceOf(address)".to_string(),
        );
        m.insert("value".to_string(), "0".to_string());
        let args = ChainCallArgs::from_map(m).expect("should parse");
        assert_eq!(args.data, "0x70a08231");
        assert_eq!(
            args.function_signature.as_deref(),
            Some("balanceOf(address)")
        );
        assert_eq!(args.extra.get("value").map(String::as_str), Some("0"));
        assert!(!args.extra.contains_key("calldata"));
        assert!(!args.extra.contains_key("function_signature"));
    }

    #[test]
    fn chain_call_args_from_map_rejects_missing_calldata() {
        let m = std::collections::HashMap::new();
        assert!(ChainCallArgs::from_map(m).is_err());
    }

    #[test]
    fn chain_call_args_roundtrips_through_flat_map() {
        let args = ChainCallArgs {
            data: "0x70a08231".to_string(),
            function_signature: Some("balanceOf(address)".to_string()),
            extra: std::collections::HashMap::new(),
        };
        let flat = args.to_flat_map();
        assert_eq!(flat.get("data").map(String::as_str), Some("0x70a08231"));
        assert_eq!(
            flat.get("function_signature").map(String::as_str),
            Some("balanceOf(address)")
        );
    }

    #[test]
    fn deploy_typed_with_args_produces_same_result_as_deploy_typed() {
        let mut flat = std::collections::HashMap::new();
        flat.insert("raw_transaction".to_string(), "0xdead".to_string());
        let result_flat = deploy_typed(999_999, "Demo".to_string(), flat);

        let args = ChainDeployArgs {
            raw_transaction: "0xdead".to_string(),
            extra: std::collections::HashMap::new(),
        };
        let result_typed = deploy_typed_with_args(999_999, "Demo".to_string(), args);

        assert_eq!(result_flat.error_code, result_typed.error_code);
        assert_eq!(result_flat.message, result_typed.message);
    }

    #[test]
    fn call_typed_with_args_produces_same_result_as_call_typed() {
        let mut flat = std::collections::HashMap::new();
        flat.insert("data".to_string(), "0x70a08231".to_string());
        let result_flat = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            flat,
        );

        let args = ChainCallArgs {
            data: "0x70a08231".to_string(),
            function_signature: None,
            extra: std::collections::HashMap::new(),
        };
        let result_typed = call_typed_with_args(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            args,
        );

        assert_eq!(result_flat.error_code, result_typed.error_code);
        assert_eq!(result_flat.message, result_typed.message);
    }

    // -- End typed kwargs struct tests --

    #[test]
    fn erc20_balance_of_selector_and_padding_match_golden_vector() {
        let calldata =
            build_erc20_balance_of_calldata("0x000000000000000000000000000000000000dead");
        assert_eq!(
            calldata,
            "0x70a08231000000000000000000000000000000000000000000000000000000000000dead"
        );
        assert_eq!(calldata.len(), 74);
    }

    #[test]
    fn erc20_balance_of_calldata_normalizes_case_and_prefix() {
        let lower = build_erc20_balance_of_calldata("0x000000000000000000000000000000000000dead");
        let upper = build_erc20_balance_of_calldata("000000000000000000000000000000000000DEAD");
        assert_eq!(lower, upper);
    }

    #[test]
    fn typed_call_request_requires_calldata_and_normalizes_hex() {
        let mut args = std::collections::HashMap::new();
        args.insert("calldata".to_string(), "70a08231".to_string());
        let request =
            typed_call_request_from_args("000000000000000000000000000000000000beef", &args)
                .expect("expected typed call request");
        assert_eq!(
            request.to,
            "0x000000000000000000000000000000000000beef".to_string()
        );
        assert_eq!(request.data, "0x70a08231".to_string());
        assert_eq!(request.function_signature, None);
    }

    #[test]
    fn typed_call_request_preserves_optional_function_signature() {
        let mut args = std::collections::HashMap::new();
        args.insert("data".to_string(), "0x70a08231".to_string());
        args.insert(
            "function_signature".to_string(),
            "balanceOf(address)".to_string(),
        );
        let request =
            typed_call_request_from_args("0x000000000000000000000000000000000000beef", &args)
                .expect("expected typed call request");
        assert_eq!(
            request.function_signature.as_deref(),
            Some("balanceOf(address)")
        );
    }

    #[test]
    fn typed_deploy_request_requires_signed_payload_and_normalizes_hex() {
        let mut args = std::collections::HashMap::new();
        args.insert("signed_tx".to_string(), "deadbeef".to_string());
        let request = typed_deploy_request_from_args(&args).expect("expected typed deploy request");
        assert_eq!(request.raw_transaction, "0xdeadbeef".to_string());
    }

    #[test]
    fn typed_response_envelopes_preserve_legacy_output_contract() {
        let deploy_ok = TypedDeployResponse::success("0xabc".to_string()).into_legacy_string();
        assert_eq!(deploy_ok, "0xabc".to_string());

        let deploy_err = TypedDeployResponse::error(
            TypedChainErrorCode::MissingRequiredField,
            "error: strict chain policy requires raw_transaction or signed_tx for deploy"
                .to_string(),
        )
        .into_legacy_string();
        assert!(deploy_err.contains("strict chain policy"));

        let call_ok = TypedCallResponse::success_with_evidence(
            "0x01".to_string(),
            None,
            None,
            None,
            None,
            None,
        )
        .into_legacy_string();
        assert_eq!(call_ok, "0x01".to_string());
    }

    #[test]
    fn typed_response_envelopes_capture_evidence_fields() {
        let deploy = TypedDeployResponse::success_with_evidence(
            "0xabc".to_string(),
            Some("0xtx".to_string()),
            Some("success".to_string()),
            Some("0x08c379a0".to_string()),
        );
        assert_eq!(deploy.tx_hash, Some("0xtx".to_string()));
        assert_eq!(deploy.receipt_status, Some("success".to_string()));
        assert_eq!(deploy.revert_data, Some("0x08c379a0".to_string()));
        assert_eq!(deploy.error_code, None);

        let deploy_reverted = TypedDeployResponse::success_with_evidence(
            "0xabc".to_string(),
            Some("0xtx".to_string()),
            Some("reverted".to_string()),
            Some("0x08c379a0".to_string()),
        );
        assert_eq!(
            deploy_reverted.error_code,
            Some(TypedChainErrorCode::TxReverted)
        );

        let call = TypedCallResponse::success_with_evidence(
            "0x01".to_string(),
            Some("true".to_string()),
            Some("decode warning".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(call.tx_hash, None);
        assert_eq!(call.decoded, Some("true".to_string()));
        assert_eq!(call.decode_error, Some("decode warning".to_string()));
        assert_eq!(call.receipt_status, None);
        assert_eq!(call.revert_data, None);
        assert_eq!(call.error_code, Some(TypedChainErrorCode::AbiDecodeFailed));
    }

    #[test]
    #[serial_test::serial]
    fn typed_deploy_strict_missing_payload_uses_missing_required_field_code() {
        std::env::set_var("DAL_CHAIN_STRICT", "1");
        let r = deploy_typed(1, "Demo".to_string(), std::collections::HashMap::new());
        std::env::remove_var("DAL_CHAIN_STRICT");
        assert_eq!(r.error_code, Some("MISSING_REQUIRED_FIELD".to_string()));
        assert!(r.message.contains("strict chain policy"));
    }

    #[test]
    #[serial_test::serial]
    fn typed_call_strict_missing_calldata_uses_missing_required_field_code() {
        std::env::set_var("DAL_CHAIN_STRICT", "1");
        let r = call_typed(
            1,
            "0x000000000000000000000000000000000000dead".to_string(),
            "transfer".to_string(),
            std::collections::HashMap::new(),
        );
        std::env::remove_var("DAL_CHAIN_STRICT");
        assert_eq!(r.error_code, Some("MISSING_REQUIRED_FIELD".to_string()));
        assert!(r.message.contains("strict chain policy"));
    }

    #[test]
    fn typed_chain_error_code_tokens_are_unique() {
        use std::collections::HashSet;
        let codes = [
            TypedChainErrorCode::StrictPolicy,
            TypedChainErrorCode::MissingRequiredField,
            TypedChainErrorCode::ChainUnsupported,
            TypedChainErrorCode::RpcFailure,
            TypedChainErrorCode::TxReverted,
            TypedChainErrorCode::AbiDecodeFailed,
        ];
        let mut seen = HashSet::new();
        for c in codes {
            assert!(
                seen.insert(c.as_str()),
                "duplicate error token: {}",
                c.as_str()
            );
        }
    }

    #[test]
    fn public_typed_api_surfaces_chain_unsupported_with_typed_error_code() {
        let deploy_result = deploy_typed(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(
            deploy_result.error_code,
            Some("CHAIN_UNSUPPORTED".to_string())
        );

        let call_result = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(
            call_result.error_code,
            Some("CHAIN_UNSUPPORTED".to_string())
        );
    }

    #[test]
    fn public_typed_api_chain_unsupported_preserves_evidence_contract() {
        let deploy_result = deploy_typed(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(deploy_result.contract_address, None);
        assert_eq!(deploy_result.tx_hash, None);
        assert_eq!(deploy_result.receipt_status, None);
        assert_eq!(deploy_result.revert_data, None);
        assert_eq!(
            deploy_result.error_code,
            Some("CHAIN_UNSUPPORTED".to_string())
        );

        let call_result = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(call_result.result_hex, None);
        assert_eq!(call_result.decoded, None);
        assert_eq!(call_result.decode_error, None);
        assert_eq!(call_result.tx_hash, None);
        assert_eq!(call_result.receipt_status, None);
        assert_eq!(call_result.revert_data, None);
        assert_eq!(
            call_result.error_code,
            Some("CHAIN_UNSUPPORTED".to_string())
        );
    }

    #[test]
    fn legacy_and_typed_wrappers_remain_behaviorally_aligned() {
        let deploy_legacy = super::deploy(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        let deploy_typed_result = deploy_typed(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(
            deploy_legacy,
            deploy_typed_result.clone().into_legacy_string()
        );
        assert_eq!(
            deploy_legacy,
            deploy_typed_result
                .contract_address
                .unwrap_or(deploy_typed_result.message)
        );

        let call_legacy = super::call(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        let call_typed_result = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        assert_eq!(call_legacy, call_typed_result.clone().into_legacy_string());
        assert_eq!(
            call_legacy,
            call_typed_result
                .result_hex
                .unwrap_or(call_typed_result.message)
        );
    }

    #[test]
    fn deploy_result_to_value_map_provenance_contract_has_exact_keys() {
        let result = deploy_typed(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        let map = result.to_value_map();
        let expected_keys: std::collections::HashSet<&str> = [
            "contract_address",
            "tx_hash",
            "receipt_status",
            "revert_data",
            "error_code",
            "message",
        ]
        .iter()
        .copied()
        .collect();
        let actual_keys: std::collections::HashSet<&str> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(
            actual_keys, expected_keys,
            "deploy provenance key contract violated"
        );
    }

    #[test]
    fn call_result_to_value_map_provenance_contract_has_exact_keys() {
        let result = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        let map = result.to_value_map();
        let expected_keys: std::collections::HashSet<&str> = [
            "result_hex",
            "decoded",
            "decode_error",
            "tx_hash",
            "receipt_status",
            "revert_data",
            "error_code",
            "message",
        ]
        .iter()
        .copied()
        .collect();
        let actual_keys: std::collections::HashSet<&str> = map.keys().map(|k| k.as_str()).collect();
        assert_eq!(
            actual_keys, expected_keys,
            "call provenance key contract violated"
        );
    }

    #[test]
    fn deploy_result_audit_evidence_includes_context_and_provenance() {
        let result = deploy_typed(
            999_999,
            "Demo".to_string(),
            std::collections::HashMap::new(),
        );
        assert!(result.has_evidence());
        let evidence = result.to_audit_evidence(999_999, "Demo");
        assert_eq!(evidence.get("chain_id"), Some(&Value::Int(999_999)));
        assert_eq!(
            evidence.get("contract_name"),
            Some(&Value::String("Demo".to_string()))
        );
        assert!(evidence.contains_key("error_code"));
        assert!(evidence.contains_key("result"));
    }

    #[test]
    fn call_result_audit_evidence_includes_context_and_provenance() {
        let result = call_typed(
            999_999,
            "0x000000000000000000000000000000000000dead".to_string(),
            "balanceOf".to_string(),
            std::collections::HashMap::new(),
        );
        assert!(result.has_evidence());
        let evidence = result.to_audit_evidence(
            999_999,
            "0x000000000000000000000000000000000000dead",
            "balanceOf",
        );
        assert_eq!(evidence.get("chain_id"), Some(&Value::Int(999_999)));
        assert_eq!(
            evidence.get("contract_address"),
            Some(&Value::String(
                "0x000000000000000000000000000000000000dead".to_string()
            ))
        );
        assert_eq!(
            evidence.get("function_name"),
            Some(&Value::String("balanceOf".to_string()))
        );
        assert!(evidence.contains_key("error_code"));
        assert!(evidence.contains_key("result"));
    }

    #[test]
    fn erc721_token_uri_selector_and_padding_match_golden_vector() {
        let calldata = build_erc721_token_uri_calldata(1);
        assert_eq!(
            calldata,
            "0xc87b56dd0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(calldata.len(), 74);
    }

    #[test]
    fn erc721_owner_of_selector_and_padding_match_golden_vector() {
        let calldata = build_erc721_owner_of_calldata(1);
        assert_eq!(
            calldata,
            "0x6352211e0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(calldata.len(), 74);
    }

    #[test]
    fn selector_reference_parity_matrix_for_selected_signatures() {
        // Reference matrix values locked to widely used protocol/interface selectors.
        let matrix = [
            ("balanceOf(address)", "70a08231"),
            ("transfer(address,uint256)", "a9059cbb"),
            ("approve(address,uint256)", "095ea7b3"),
            ("transferFrom(address,address,uint256)", "23b872dd"),
            ("allowance(address,address)", "dd62ed3e"),
            ("supportsInterface(bytes4)", "01ffc9a7"),
            ("name()", "06fdde03"),
            ("symbol()", "95d89b41"),
            ("getReserves()", "0902f1ac"),
            ("swap(uint256,uint256,address,bytes)", "022c0d9f"),
        ];
        for (signature, expected_selector) in matrix {
            assert_eq!(
                selector_from_signature(signature),
                expected_selector,
                "selector mismatch for {}",
                signature
            );
        }
    }

    #[test]
    fn supports_interface_bytes4_calldata_layout_matches_reference_vector() {
        // ERC165 supportsInterface(bytes4) with interface id 0x80ac58cd (ERC721).
        let interface_id = "80ac58cd";
        let calldata = format!(
            "0x{}{}{}",
            selector_from_signature("supportsInterface(bytes4)"),
            "0".repeat(56),
            interface_id
        );
        assert_eq!(
            calldata,
            "0x01ffc9a70000000000000000000000000000000000000000000000000000000080ac58cd"
        );
        assert_eq!(calldata.len(), 74);
    }

    #[test]
    fn abi_word_encoding_matches_golden_vectors() {
        assert_eq!(
            encode_address_word("0x000000000000000000000000000000000000dead"),
            "000000000000000000000000000000000000000000000000000000000000dead"
        );
        assert_eq!(
            encode_uint256_word(1),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            encode_uint256_word(u128::MAX),
            "00000000000000000000000000000000ffffffffffffffffffffffffffffffff"
        );
        assert_eq!(
            encode_bytes32_word(
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            ),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }

    #[test]
    fn static_tuple_like_argument_pack_matches_golden_layout() {
        // Baseline for (address,uint256,bool,bytes32) static argument packing.
        let selector = "0x11223344";
        let payload = build_custom_error_payload(
            selector,
            &[
                encode_address_word("0x000000000000000000000000000000000000dead"),
                encode_uint256_word(42),
                encode_uint256_word(1),
                encode_bytes32_word(
                    "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                ),
            ],
        );
        assert_eq!(
            payload,
            "0x11223344000000000000000000000000000000000000000000000000000000000000dead000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000011234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        assert_eq!(payload.len(), 266);
    }

    #[test]
    fn static_tuple_like_argument_pack_boundary_permutations_match_layout() {
        // Boundary permutation:
        // (max address, max uint128-backed uint256 word, bool=false, 0xff..ff bytes32)
        let selector = "0x99aabbcc";
        let payload = build_custom_error_payload(
            selector,
            &[
                encode_address_word("0xffffffffffffffffffffffffffffffffffffffff"),
                encode_uint256_word(u128::MAX),
                encode_bool_word(false),
                encode_bytes32_word(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                ),
            ],
        );
        let expected = format!(
            "0x99aabbcc{}{}{}{}",
            encode_address_word("0xffffffffffffffffffffffffffffffffffffffff"),
            encode_uint256_word(u128::MAX),
            encode_bool_word(false),
            encode_bytes32_word(
                "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            )
        );
        assert_eq!(payload, expected);
        assert_eq!(payload.len(), 266);
    }

    #[test]
    fn erc20_transfer_selector_and_arguments_match_golden_vector() {
        let calldata =
            build_erc20_transfer_calldata("0x000000000000000000000000000000000000dead", 1);
        assert_eq!(
            calldata,
            "0xa9059cbb000000000000000000000000000000000000000000000000000000000000dead0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(calldata.len(), 138);
    }

    #[test]
    fn erc20_transfer_and_approve_bool_return_decode_vectors() {
        // Typical ERC20 success return word
        let success_word = "0000000000000000000000000000000000000000000000000000000000000001";
        // Typical ERC20 failure return word
        let failure_word = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(decode_bool_word(success_word).unwrap());
        assert!(!decode_bool_word(failure_word).unwrap());
    }

    #[test]
    fn erc20_approve_selector_and_arguments_match_golden_vector() {
        let calldata =
            build_erc20_approve_calldata("0x000000000000000000000000000000000000dead", 1);
        assert_eq!(
            calldata,
            "0x095ea7b3000000000000000000000000000000000000000000000000000000000000dead0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(calldata.len(), 138);
    }

    #[test]
    fn erc20_transfer_from_selector_and_arguments_match_golden_vector() {
        let calldata = build_erc20_transfer_from_calldata(
            "0x000000000000000000000000000000000000beef",
            "0x000000000000000000000000000000000000dead",
            1,
        );
        assert_eq!(
            calldata,
            "0x23b872dd000000000000000000000000000000000000000000000000000000000000beef000000000000000000000000000000000000000000000000000000000000dead0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(calldata.len(), 202);
    }

    #[test]
    fn erc20_allowance_selector_and_arguments_match_golden_vector() {
        let calldata = build_erc20_allowance_calldata(
            "0x000000000000000000000000000000000000beef",
            "0x000000000000000000000000000000000000dead",
        );
        assert_eq!(
            calldata,
            "0xdd62ed3e000000000000000000000000000000000000000000000000000000000000beef000000000000000000000000000000000000000000000000000000000000dead"
        );
        assert_eq!(calldata.len(), 138);
    }

    #[test]
    fn erc20_total_supply_and_decimals_selectors_match_golden_vectors() {
        let total_supply = build_erc20_total_supply_calldata();
        let decimals = build_erc20_decimals_calldata();
        assert_eq!(total_supply, "0x18160ddd");
        assert_eq!(decimals, "0x313ce567");
        assert_eq!(total_supply.len(), 10);
        assert_eq!(decimals.len(), 10);
    }

    #[test]
    fn erc20_total_supply_and_decimals_return_word_decode_vectors() {
        // Typical EVM return words for constant calls:
        // totalSupply = 1_000_000, decimals = 18
        let total_supply_word = "00000000000000000000000000000000000000000000000000000000000f4240";
        let decimals_word = "0000000000000000000000000000000000000000000000000000000000000012";
        assert_eq!(decode_uint256_word(total_supply_word).unwrap(), 1_000_000);
        assert_eq!(decode_uint256_word(decimals_word).unwrap(), 18);
    }

    #[test]
    fn erc20_constant_call_return_decode_boundary_vectors() {
        // Constant-call decode boundaries for uint-return functions like balanceOf/allowance.
        let zero_word = "0000000000000000000000000000000000000000000000000000000000000000";
        let one_ether_word = "0000000000000000000000000000000000000000000000000de0b6b3a7640000";
        let max_u128_word = "00000000000000000000000000000000ffffffffffffffffffffffffffffffff";

        assert_eq!(decode_uint256_word(zero_word).unwrap(), 0);
        assert_eq!(
            decode_uint256_word(one_ether_word).unwrap(),
            1_000_000_000_000_000_000
        );
        assert_eq!(decode_uint256_word(max_u128_word).unwrap(), u128::MAX);
    }

    #[test]
    fn decode_uint_bool_and_address_words_match_golden_vectors() {
        assert_eq!(
            decode_uint256_word("000000000000000000000000000000000000000000000000000000000000002a")
                .unwrap(),
            42
        );
        assert!(decode_bool_word(
            "0000000000000000000000000000000000000000000000000000000000000001"
        )
        .unwrap());
        assert!(!decode_bool_word(
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap());
        assert_eq!(
            decode_address_word("000000000000000000000000000000000000000000000000000000000000dead")
                .unwrap(),
            "0x000000000000000000000000000000000000dead"
        );
    }

    #[test]
    fn decode_bool_rejects_non_boolean_word_values() {
        let err =
            decode_bool_word("0000000000000000000000000000000000000000000000000000000000000002")
                .unwrap_err();
        assert!(err.contains("invalid ABI bool value"));
    }

    #[test]
    fn revert_error_payload_matches_golden_vector_for_short_ascii_message() {
        // Error(string) with "nope"
        let payload = build_revert_error_payload("nope");
        assert_eq!(
            payload,
            "0x08c379a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000046e6f706500000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn revert_error_payload_handles_empty_exact32_and_over32_messages() {
        let empty = build_revert_error_payload("");
        assert_eq!(
            empty,
            "0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000"
        );

        let exact_32 = build_revert_error_payload("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(
            exact_32,
            "0x08c379a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000206161616161616161616161616161616161616161616161616161616161616161"
        );

        let over_32 = build_revert_error_payload("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        assert_eq!(
            over_32,
            "0x08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002161616161616161616161616161616161616161616161616161616161616161616100000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn dynamic_string_return_decode_vectors() {
        // ABI return payload for string "hello":
        // offset=0x20, len=5, data=68656c6c6f padded to 32-byte boundary.
        let payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000005\
68656c6c6f000000000000000000000000000000000000000000000000000000";
        assert_eq!(decode_abi_string_data(payload).unwrap(), "hello");

        let empty_payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(decode_abi_string_data(empty_payload).unwrap(), "");
    }

    #[test]
    fn dynamic_bytes_return_decode_vectors() {
        // ABI return payload for bytes 0xdeadbeef:
        // offset=0x20, len=4, data=deadbeef padded to 32-byte boundary.
        let payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000004\
deadbeef00000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            decode_abi_bytes_data(payload).unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );

        let empty_payload = "0x0000000000000000000000000000000000000000000000000000000000000020\
0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            decode_abi_bytes_data(empty_payload).unwrap(),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn static_tuple_decode_parity_vectors_for_address_uint_bool() {
        let payload = format!(
            "{}{}{}",
            encode_address_word("0x000000000000000000000000000000000000beef"),
            encode_uint256_word(42),
            encode_bool_word(true)
        );
        let decoded = decode_static_tuple_address_uint_bool_payload(&payload).unwrap();
        assert_eq!(
            decoded.0,
            "0x000000000000000000000000000000000000beef".to_string()
        );
        assert_eq!(decoded.1, 42);
        assert!(decoded.2);
    }

    #[test]
    fn nested_dynamic_tuple_decode_parity_vectors() {
        // tuple(string,bytes): ("hello", 0xdeadbeef)
        // head:
        //  - string offset = 0x40
        //  - bytes offset  = 0x80
        let payload = "0x0000000000000000000000000000000000000000000000000000000000000040\
0000000000000000000000000000000000000000000000000000000000000080\
0000000000000000000000000000000000000000000000000000000000000005\
68656c6c6f000000000000000000000000000000000000000000000000000000\
0000000000000000000000000000000000000000000000000000000000000004\
deadbeef00000000000000000000000000000000000000000000000000000000";
        let decoded = decode_abi_tuple_string_bytes_payload(payload).unwrap();
        assert_eq!(decoded.0, "hello".to_string());
        assert_eq!(decoded.1, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn selector_mismatch_rejection_vectors_for_custom_error_payloads() {
        let payload = build_custom_error_payload("0xdeadbeef", &[encode_uint256_word(1)]);
        let err = decode_custom_error_payload_words(&payload, "0xfeedface").unwrap_err();
        assert!(err.contains("unexpected custom error selector"));

        let words = decode_custom_error_payload_words(&payload, "0xdeadbeef").unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(decode_uint256_word(&words[0]).unwrap(), 1);
    }

    #[test]
    fn revert_data_decode_parity_vectors_for_error_string_and_panic() {
        let err_payload = build_revert_error_payload("insufficient balance");
        let panic_payload = build_panic_revert_payload(0x11);
        assert_eq!(
            decode_revert_error_string_payload(&err_payload).unwrap(),
            "insufficient balance"
        );
        assert_eq!(
            decode_revert_panic_code_payload(&panic_payload).unwrap(),
            0x11
        );
    }

    #[test]
    fn dynamic_and_revert_decode_reject_malformed_payload_vectors() {
        let short_dynamic = "0x00";
        let err = decode_abi_string_data(short_dynamic).unwrap_err();
        assert!(err.contains("too short"));

        let bad_offset_dynamic =
            "0x0000000000000000000000000000000000000000000000000000000000000001\
0000000000000000000000000000000000000000000000000000000000000000";
        let err = decode_abi_bytes_data(bad_offset_dynamic).unwrap_err();
        assert!(err.contains("invalid ABI bytes offset"));

        let wrong_selector_error =
            "0xdeadbeef0000000000000000000000000000000000000000000000000000000000000020";
        let err = decode_revert_error_string_payload(wrong_selector_error).unwrap_err();
        assert!(err.contains("unexpected revert selector"));

        let short_panic = "0x4e487b71";
        let err = decode_revert_panic_code_payload(short_panic).unwrap_err();
        assert!(err.contains("panic payload too short"));

        let bad_tuple_len = "0x00";
        let err = decode_static_tuple_address_uint_bool_payload(bad_tuple_len).unwrap_err();
        assert!(err.contains("expected static tuple payload"));

        let bad_custom_error = "0xdeadbeef00";
        let err = decode_custom_error_payload_words(bad_custom_error, "0xdeadbeef").unwrap_err();
        assert!(err.contains("not 32-byte aligned"));

        let bad_nested_dynamic =
            "0x0000000000000000000000000000000000000000000000000000000000000001\
0000000000000000000000000000000000000000000000000000000000000080";
        let err = decode_abi_tuple_string_bytes_payload(bad_nested_dynamic).unwrap_err();
        assert!(err.contains("invalid tuple offsets"));
    }

    #[test]
    fn custom_error_payload_baseline_matches_selector_plus_encoded_words() {
        // Baseline custom-error payload layout:
        // <4-byte selector><encoded arg word 1>
        let payload = build_custom_error_payload(
            "0xdeadbeef",
            &[encode_address_word(
                "0x000000000000000000000000000000000000dead",
            )],
        );
        assert_eq!(
            payload,
            "0xdeadbeef000000000000000000000000000000000000000000000000000000000000dead"
        );
    }

    #[test]
    fn custom_error_payload_multi_arg_baseline_matches_expected_layout() {
        // Example baseline: CustomError(address,uint256)
        // selector chosen as deterministic placeholder for layout locking
        let payload = build_custom_error_payload(
            "0x12345678",
            &[
                encode_address_word("0x000000000000000000000000000000000000dead"),
                encode_uint256_word(42),
            ],
        );
        assert_eq!(
            payload,
            "0x12345678000000000000000000000000000000000000000000000000000000000000dead000000000000000000000000000000000000000000000000000000000000002a"
        );
        assert_eq!(payload.len(), 138);
    }

    #[test]
    fn custom_error_payload_strips_selector_prefix_and_keeps_word_order() {
        let payload = build_custom_error_payload(
            "abcdef01",
            &[
                encode_uint256_word(1),
                encode_uint256_word(2),
                encode_uint256_word(3),
            ],
        );
        assert_eq!(
            payload,
            "0xabcdef01000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
        );
    }

    #[test]
    fn protocol_style_custom_error_family_layout_vectors() {
        // Protocol-style family using canonical keccak-derived selectors.
        // - ERC20InsufficientBalance(address,uint256,uint256)
        // - ERC20InvalidSender(address)
        // - ERC20InvalidReceiver(address)
        let insufficient_balance_selector =
            selector_from_signature("ERC20InsufficientBalance(address,uint256,uint256)");
        let invalid_sender_selector = selector_from_signature("ERC20InvalidSender(address)");
        let invalid_receiver_selector = selector_from_signature("ERC20InvalidReceiver(address)");

        // Locked known selectors for ERC-6093-style errors.
        assert_eq!(insufficient_balance_selector, "e450d38c");
        assert_eq!(invalid_sender_selector, "96c6fd1e");
        assert_eq!(invalid_receiver_selector, "ec442f05");

        let insufficient_balance = build_custom_error_payload(
            &insufficient_balance_selector,
            &[
                encode_address_word("0x000000000000000000000000000000000000beef"),
                encode_uint256_word(10),
                encode_uint256_word(100),
            ],
        );
        assert_eq!(
            insufficient_balance,
            "0xe450d38c000000000000000000000000000000000000000000000000000000000000beef000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000064"
        );
        assert_eq!(insufficient_balance.len(), 202);

        let invalid_sender = build_custom_error_payload(
            &invalid_sender_selector,
            &[encode_address_word(
                "0x0000000000000000000000000000000000000000",
            )],
        );
        assert_eq!(
            invalid_sender,
            "0x96c6fd1e0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(invalid_sender.len(), 74);

        let invalid_receiver = build_custom_error_payload(
            &invalid_receiver_selector,
            &[encode_address_word(
                "0x000000000000000000000000000000000000dead",
            )],
        );
        assert_eq!(
            invalid_receiver,
            "0xec442f05000000000000000000000000000000000000000000000000000000000000dead"
        );
        assert_eq!(invalid_receiver.len(), 74);
    }

    #[test]
    fn protocol_custom_error_selector_catalog_and_argument_decode_parity_vectors() {
        let catalog = canonical_selector_catalog();
        let selector = catalog
            .get("ERC20InsufficientBalance(address,uint256,uint256)")
            .expect("catalog should include ERC20InsufficientBalance selector");
        assert_eq!(
            selector_from_signature("ERC20InsufficientBalance(address,uint256,uint256)"),
            *selector
        );

        let payload = build_custom_error_payload(
            selector,
            &[
                encode_address_word("0x000000000000000000000000000000000000beef"),
                encode_uint256_word(10),
                encode_uint256_word(100),
            ],
        );
        assert_eq!(&payload[2..10], *selector);

        let encoded_words = &payload[10..];
        let owner_word = &encoded_words[0..64];
        let balance_word = &encoded_words[64..128];
        let needed_word = &encoded_words[128..192];

        assert_eq!(
            decode_address_word(owner_word).unwrap(),
            "0x000000000000000000000000000000000000beef"
        );
        assert_eq!(decode_uint256_word(balance_word).unwrap(), 10);
        assert_eq!(decode_uint256_word(needed_word).unwrap(), 100);
    }

    #[test]
    fn canonical_selector_catalog_matches_golden_values() {
        let catalog = canonical_selector_catalog();
        assert_eq!(catalog.get("Error(string)"), Some(&"08c379a0"));
        assert_eq!(catalog.get("Panic(uint256)"), Some(&"4e487b71"));
        assert_eq!(catalog.get("ERC20.balanceOf(address)"), Some(&"70a08231"));
        assert_eq!(
            catalog.get("ERC20.transfer(address,uint256)"),
            Some(&"a9059cbb")
        );
        assert_eq!(
            catalog.get("ERC20.approve(address,uint256)"),
            Some(&"095ea7b3")
        );
        assert_eq!(
            catalog.get("ERC20.transferFrom(address,address,uint256)"),
            Some(&"23b872dd")
        );
        assert_eq!(
            catalog.get("ERC20.allowance(address,address)"),
            Some(&"dd62ed3e")
        );
        assert_eq!(catalog.get("ERC20.totalSupply()"), Some(&"18160ddd"));
        assert_eq!(catalog.get("ERC20.decimals()"), Some(&"313ce567"));
        assert_eq!(catalog.get("ERC721.ownerOf(uint256)"), Some(&"6352211e"));
        assert_eq!(catalog.get("ERC721.tokenURI(uint256)"), Some(&"c87b56dd"));
    }

    #[test]
    fn panic_revert_payload_matches_golden_vector_for_arithmetic_overflow_code() {
        // Solidity Panic(0x11): arithmetic overflow/underflow
        let payload = build_panic_revert_payload(0x11);
        assert_eq!(
            payload,
            "0x4e487b710000000000000000000000000000000000000000000000000000000000000011"
        );
    }

    #[test]
    fn canonical_selector_catalog_has_unique_8_hex_selectors() {
        let catalog = canonical_selector_catalog();
        let mut unique = std::collections::HashSet::new();
        for selector in catalog.values() {
            assert_eq!(selector.len(), 8);
            assert!(selector.chars().all(|c| c.is_ascii_hexdigit()));
            assert!(
                unique.insert(*selector),
                "duplicate selector in catalog: {}",
                selector
            );
        }
        assert_eq!(unique.len(), catalog.len());
    }
}
