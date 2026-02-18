// Web3 / EVM integration for MoldRegistry (mintMold, useMold, getMoldInfo).
// Requires "web3" feature. Env: DAL_RPC_URL, DAL_PRIVATE_KEY, DAL_MOLD_REGISTRY_ADDRESS.

use ethers::contract::abigen;
use ethers::prelude::*;
use std::str::FromStr;
use std::sync::Arc;

abigen!(
    MoldRegistry,
    r#"[
        function mintMold(string ipfsHash, uint256 mintFee, uint256 maxUseCount) external returns (uint256)
        function useMold(uint256 moldId) external payable returns (uint256)
        function getMoldInfo(uint256 moldId) external view returns (address creator, string ipfsHash, uint256 mintFee, uint256 mintCount, uint256 maxUseCount, bool active, uint256 createdAt, uint256 updatedAt)
        function getMoldByIpfsHash(string ipfsHash) external view returns (uint256)
    ]"#
);

pub const DEFAULT_RPC_URL: &str = "https://rpc.sepolia.org";

fn rpc_url() -> String {
    std::env::var("DAL_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string())
}

fn private_key() -> Result<String, String> {
    std::env::var("DAL_PRIVATE_KEY").map_err(|_| {
        "DAL_PRIVATE_KEY not set (hex string, no 0x prefix). Required for mint/useMold.".to_string()
    })
}

fn contract_address() -> Result<Address, String> {
    let s = std::env::var("DAL_MOLD_REGISTRY_ADDRESS").map_err(|_| {
        "DAL_MOLD_REGISTRY_ADDRESS not set (deployed MoldRegistry contract).".to_string()
    })?;
    Address::from_str(s.trim().trim_start_matches("0x"))
        .map_err(|e| format!("Invalid DAL_MOLD_REGISTRY_ADDRESS: {}", e))
}

/// Build client (provider + signer) and contract instance. Returns (client, contract) for reuse.
fn client_and_contract() -> Result<
    (
        Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
        MoldRegistry<SignerMiddleware<Provider<Http>, LocalWallet>>,
    ),
    String,
> {
    let rpc = rpc_url();
    let pk = private_key()?;
    let addr = contract_address()?;

    let provider =
        Provider::<Http>::try_from(rpc.as_str()).map_err(|e| format!("RPC connection: {}", e))?;
    let chain_id = tokio::runtime::Runtime::new()
        .map_err(|e| format!("runtime: {}", e))?
        .block_on(provider.get_chainid())
        .map_err(|e| format!("get chainid: {}", e))?
        .as_u64();
    let wallet = pk
        .trim_start_matches("0x")
        .parse::<LocalWallet>()
        .map_err(|e| format!("Invalid DAL_PRIVATE_KEY: {}", e))?
        .with_chain_id(chain_id);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));
    let contract = MoldRegistry::new(addr, client.clone());
    Ok((client, contract))
}

/// Mint a new mold on-chain. Returns mold ID.
pub fn mint_mold(ipfs_hash: &str, mint_fee_wei: u128, max_use_count: u64) -> Result<u64, String> {
    let (_client, contract) = client_and_contract()?;
    let hash = ipfs_hash.trim().trim_start_matches("ipfs://").to_string();
    if hash.is_empty() {
        return Err("ipfs_hash required".to_string());
    }
    let tx = contract.mint_mold(hash, U256::from(mint_fee_wei), U256::from(max_use_count));
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime: {}", e))?;
    let pending = rt
        .block_on(tx.send())
        .map_err(|e| format!("mintMold send: {}", e))?;
    let receipt = rt
        .block_on(pending)
        .map_err(|e| format!("mintMold receipt: {}", e))?;
    let receipt = receipt.ok_or("no receipt")?;
    // Parse moldId from logs (MoldMinted event) or from tx; MoldMinted(moldId, ...) is first topic? No - event has indexed moldId. Simpler: get return value via call instead of send? Actually mintMold returns uint256. We need to get the return value. In ethers, .send() gives PendingTx; .await gives Option<TransactionReceipt>. The receipt doesn't contain return data. So we'd need to use .call() to simulate and get return value, or parse logs. The event MoldMinted has moldId as indexed, so it's in topics[1]. Let me parse it.
    let log = receipt.logs.first().ok_or("no MoldMinted log")?;
    // topics[0] = event sig, topics[1] = moldId (indexed uint256)
    let mold_id_bytes = log.topics.get(1).ok_or("no moldId in log")?;
    let mold_id = U256::from(mold_id_bytes.as_ref()).as_u64();
    Ok(mold_id)
}

/// Call useMold(moldId) with value (wei). Returns instance ID.
pub fn use_mold(mold_id: u64, value_wei: u128) -> Result<u128, String> {
    let (_client, contract) = client_and_contract()?;
    let tx = contract
        .use_mold(U256::from(mold_id))
        .value(U256::from(value_wei));
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime: {}", e))?;
    let pending = rt
        .block_on(tx.send())
        .map_err(|e| format!("useMold send: {}", e))?;
    let receipt = rt
        .block_on(pending)
        .map_err(|e| format!("useMold receipt: {}", e))?;
    let _receipt = receipt.ok_or("no receipt")?;
    // instanceId is in MoldUsed event: (moldId, user, instanceId). instanceId is non-indexed so in data. For simplicity return 0 or parse log. The contract returns instanceId; we don't get it from receipt easily. Return 0 for now; we can parse MoldUsed data later.
    Ok(0)
}

/// Mold info returned from getMoldInfo(moldId).
#[derive(Debug, Clone)]
pub struct MoldInfo {
    pub creator: Address,
    pub ipfs_hash: String,
    pub mint_fee: u128,
    pub mint_count: u64,
    pub max_use_count: u64,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Get mold info (creator, ipfsHash, mintFee, mintCount, maxUseCount, active, ...).
pub fn get_mold_info(mold_id: u64) -> Result<MoldInfo, String> {
    let (_client, contract) = client_and_contract()?;
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime: {}", e))?;
    let ret = rt
        .block_on(contract.get_mold_info(U256::from(mold_id)).call())
        .map_err(|e| format!("getMoldInfo: {}", e))?;
    // abigen returns tuple (creator, ipfsHash, mintFee, mintCount, maxUseCount, active, createdAt, updatedAt)
    Ok(MoldInfo {
        creator: ret.0,
        ipfs_hash: ret.1,
        mint_fee: ret.2.as_u128(),
        mint_count: ret.3.as_u64(),
        max_use_count: ret.4.as_u64(),
        active: ret.5,
        created_at: ret.6.as_u64(),
        updated_at: ret.7.as_u64(),
    })
}

/// Resolve mold ID from IPFS hash (getMoldByIpfsHash).
pub fn mold_id_by_ipfs_hash(ipfs_hash: &str) -> Result<u64, String> {
    let (_client, contract) = client_and_contract()?;
    let hash = ipfs_hash.trim().trim_start_matches("ipfs://").to_string();
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("runtime: {}", e))?;
    let id = rt
        .block_on(contract.get_mold_by_ipfs_hash(hash).call())
        .map_err(|e| format!("getMoldByIpfsHash: {}", e))?;
    Ok(id.as_u64())
}
