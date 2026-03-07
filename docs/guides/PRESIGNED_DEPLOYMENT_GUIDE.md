# Pre-signed contract deployment with DAL

This guide explains how to **build** a pre-signed deployment transaction (outside DAL) and **submit** it from DAL with `chain::deploy`. Real on-chain deployment only occurs when you pass a signed tx; otherwise DAL returns a mock address for demos.

---

## Does DAL compile to EVM?

**Yes, in a limited way.** DAL has a **blockchain compile target** that:

1. **Transpiles** a DAL service (with `@compile_target("blockchain")`) to **Solidity** — contract name, state variables, function signatures, and events. Function **bodies** are emitted as stubs: `revert("DAL transpiled; implement in Solidity")`.
2. Runs **solc** on that Solidity to produce **EVM bytecode** (`.bin`) and **ABI** (`.abi`).

So the pipeline is: **DAL service → Solidity (skeleton) → solc → EVM bytecode + ABI.** You get deployable bytecode, but the on-chain logic is only the revert stub until you either hand-edit the generated `.sol` and recompile, or use the output as a template. The runtime does **not** compile or sign; that’s done at **build** time (`dal build <file.dal> --target blockchain`) and/or in Foundry/Hardhat when building the actual deploy transaction.

---

## Hybrid use case: DAL shape → bytecode → pre-signed deploy → orchestration

A practical hybrid flow is:

1. **Define the contract shape in DAL** — one service with `@compile_target("blockchain")`, state fields, and method signatures (and optionally events).
2. **Build to EVM:**  
   `dal build contract.dal --target blockchain --output out/`  
   This produces `out/ServiceName.sol`, `out/ServiceName.bin`, `out/ServiceName.abi`.
3. **Option A — use as template:** Edit `ServiceName.sol` to implement the function bodies, then run solc (or Foundry/Hardhat) to get final bytecode. Use Foundry/Hardhat to build the deployment tx and sign it.
4. **Option B — use stub bytecode for flow testing:** Build the deploy tx from the generated `.bin` (e.g. with `cast` or a small script), sign it, and pass the signed tx to DAL to test deploy + orchestration without real logic on-chain.
5. **Orchestration in DAL:** A separate DAL script (or the same codebase, different entry) receives the **signed deployment tx** (e.g. from env, CI, or a deploy service), calls `chain::deploy(chain_id, "ServiceName", { "raw_transaction": hex })`, and uses the returned address for:
   - `chain::call(chain_id, address, "methodName", args)` for subsequent calls
   - Logging, multi-chain registration, or agent workflows

So: **one language (DAL) for both the contract interface/skeleton and the deploy/call orchestration**; Solidity (generated or hand-filled) + solc give you EVM bytecode; external tooling builds and signs the deploy tx; DAL submits and orchestrates.

---

## Practical uses for the stub (oracles, NFT modules)

The generated stub has the **right interface** (state vars, function selectors, events) but every method **reverts**. That makes it useful in a few concrete ways.

### 1. Placeholder / reserved address (oracles and modules)

Deploy the stub so your system has a **fixed address** for “the oracle” or “the NFT module” before the real implementation exists:

- **Oracles:** Your app or DAL script points to `oracle_address`. If that address is the stub, any call (e.g. `getPrice(id)`) **reverts** instead of returning bad or stale data. So “oracle not yet live” or “module disabled” is explicit and safe. When the real oracle is ready, you switch config to the new address (or upgrade via proxy).
- **NFT / module:** Same idea. The stub has `mint`, `transfer`, etc. with the correct selectors but they revert. You can deploy “NFT module at 0x…” and have registries or DAL hold that address; no one can mint or transfer until you deploy the real implementation. Useful for (a) deployment order — registry points to stub, then you deploy the real contract and update the registry — or (b) feature flags — point to stub = disabled, point to real = enabled.

So the stub is a **safe, interface-correct placeholder**: correct ABI and selectors, no behavior until you replace or upgrade.

### 2. Interface validation (off-chain)

The stub does **not** validate oracle data or NFT metadata on-chain. You use the **generated ABI** (and optionally bytecode) **off-chain** for validation:

- **Oracle responses:** DAL (or another tool) loads the `.abi` from the same DAL contract definition and checks that an oracle response or an on-chain return value matches the expected shape (e.g. `getPrice(bytes32) returns (uint256)`). One source of truth in DAL for the oracle interface; the ABI drives validation in your orchestrator or tests.
- **Contract compatibility:** Check that a contract at some address exposes the expected selectors (e.g. for a “NFT module” interface). The stub’s ABI defines the canonical interface; you don’t need the stub deployed to validate that another contract is compatible.

So: **stub on-chain = placeholder; ABI from same build = validation and compatibility checks.**

### 3. When it’s practical

| Use | Practical? | Why |
|-----|------------|-----|
| Deploy stub as “oracle not live” / “module disabled” | Yes | Safe placeholder with correct interface; revert instead of wrong data or accidental mint. |
| Deploy stub first, then replace with real contract | Yes | Fixed address and interface from day one; swap implementation or config later. |
| Use generated ABI to validate oracle responses or contract interfaces | Yes | Single DAL definition → ABI → validation in DAL or tooling; no stub deployment required for validation. |
| Stub “validates” oracle data on-chain | No | Stub has no logic; validation happens off-chain using the ABI or in a separate contract you implement. |
| Stub as full oracle or NFT implementation | No | Stub always reverts; for real behavior, implement in Solidity and deploy that bytecode. |

**Summary:** Use the stub for **placeholders and reserved addresses** (oracles, NFT modules); use the **ABI** from the same DAL build for **validating oracles or module interfaces** off-chain. That combination is practical; relying on the stub to “validate” or implement behavior on-chain is not.

---

## Why pre-signed?

At **runtime**, DAL does not compile Solidity or build deployment transactions. ABI encoding and signing stay in your build pipeline or tooling (Foundry, Hardhat, Node). DAL’s role at runtime is to:

- **Submit** a signed deployment tx via `eth_sendRawTransaction`
- **Wait** for the receipt and return the contract address
- **Orchestrate** multi-chain, CI/CD, or agent-driven flows that consume that address

So the flow is: **build + sign tx (using DAL-generated bytecode or hand-written Solidity) → pass raw hex to DAL → `chain::deploy` sends it.**

---

## How to build a pre-signed deployment tx

### Option A: Foundry (Cast + script)

**1. Compile and get bytecode + constructor args**

```bash
# Build
forge build

# Encode constructor calldata (example: ERC20 name, symbol, totalSupply)
cast calldata "constructor(string,string,uint256)" "KEYS Token" "KEYS" 120000000000000000000000000
# → 0x... (use this as data for contract creation)
```

**2. Create and sign the deployment transaction**

Use a script (e.g. `script/Deploy.s.sol`) that builds the deployment tx and signs it with a key from env:

```solidity
// script/Deploy.s.sol (Foundry)
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/KEYS_Token.sol";

contract DeployScript is Script {
    function run() external returns (address) {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        KEYS_Token token = new KEYS_Token("KEYS Token", "KEYS", 120_000_000 * 1e18);
        address addr = address(token);

        vm.stopBroadcast();
        return addr;
    }
}
```

Then export the **signed raw transaction** (hex) instead of broadcasting:

- Use Foundry’s `--broadcast` with a custom serializer, or
- A small Node/script that uses `ethers` / `viem` to build the same deployment, sign it, and print `serializedTransaction` (hex).

**3. Minimal Node script to output raw deploy tx (ethers v6)**

```javascript
// scripts/build-deploy-tx.js (Node + ethers v6)
const { ethers } = require("ethers");
const fs = require("fs");

async function main() {
  const factory = await ethers.getContractFactory("KEYS_Token");
  const deployTx = await factory.getDeployTransaction(
    "KEYS Token",
    "KEYS",
    ethers.parseEther("120000000")
  );
  const wallet = new ethers.Wallet(
    process.env.PRIVATE_KEY,
    new ethers.JsonRpcProvider(process.env.RPC_URL)
  );
  const signed = await wallet.signTransaction({
    ...deployTx,
    chainId: parseInt(process.env.CHAIN_ID || "1"),
    gasLimit: deployTx.gasLimit || 5_000_000,
  });
  // Output hex for DAL (no 0x prefix is ok; DAL accepts with or without)
  console.log(signed);
}

main().catch(console.error);
```

Run: `PRIVATE_KEY=... RPC_URL=... CHAIN_ID=1 node scripts/build-deploy-tx.js` and use the printed hex in DAL.

### Option B: Hardhat

In a Hardhat script, build the deploy tx, sign it with the deployer wallet, and print the serialized tx:

```javascript
// scripts/buildDeployTx.js
const hre = require("hardhat");

async function main() {
  const [deployer] = await ethers.getSigners();
  const Factory = await ethers.getContractFactory("KEYS_Token");
  const tx = await Factory.getDeployTransaction(
    "KEYS Token",
    "KEYS",
    ethers.parseEther("120000000")
  );
  const signed = await deployer.signTransaction({
    ...tx,
    chainId: (await ethers.provider.getNetwork()).chainId,
    gasLimit: tx.gasLimit || 5000000,
  });
  console.log(signed);
}

main();
```

Run: `npx hardhat run scripts/buildDeployTx.js --network mainnet` (or use a custom RPC). Use the printed hex in DAL as `raw_transaction` or `signed_tx`.

---

## Using the raw tx in DAL

Once you have the hex string (with or without `0x`), pass it in the constructor-args map under **`raw_transaction`** or **`signed_tx`**:

```dal
@trust("hybrid")
@chain("ethereum")
service Deployer {
    fn deploy_with_presigned(raw_tx_hex: string) -> string {
        let address = chain::deploy(
            1,
            "KEYS_Token",
            {
                "raw_transaction": raw_tx_hex
            }
        );
        return address;
    }
}
```

- If the runtime has `http-interface` and the chain is configured, `chain::deploy` will call `eth_sendRawTransaction`, wait for the receipt, and return the real `contractAddress`.
- If `raw_transaction` / `signed_tx` is missing, DAL returns a **mock** address (no real deploy).

---

## Use cases

| Use case | How pre-signed helps |
|----------|----------------------|
| **DAL → EVM hybrid** | Define contract shape in DAL; `dal build --target blockchain` gives Solidity + bytecode. Build and sign the deploy tx from that bytecode (or from hand-edited Solidity). DAL script receives the signed tx and submits via `chain::deploy`, then orchestrates `chain::call` and logging with the deployed address. One codebase for interface + orchestration. |
| **CI/CD** | Build and sign the deploy tx in CI (e.g. GitHub Actions with a stored key or HSM). DAL script receives the hex (e.g. from env or a secure artifact) and submits it. No private key in DAL. |
| **Multi-chain from one DAL script** | Build one signed tx per chain in a small script (or per-chain CI step). DAL calls `chain::deploy(chain_id, name, { "raw_transaction": tx_hex })` for each chain and records addresses. |
| **Governance / approval** | Off-chain process (governance UI, multisig) produces the signed deploy tx. Once approved, DAL only submits it; no signing inside DAL. |
| **Agent-driven deploy** | Agent asks an external “deploy service” (API or script) for a signed tx; the service compiles, builds, and signs. Agent passes the hex to `chain::deploy` and uses the returned address in later steps. |
| **Testing / staging** | Same DAL script works in tests with a mock (no `raw_transaction`) and in staging/production with a real signed tx from your pipeline. |

---

## Security notes

- **Never** put a private key in DAL source or in constructor args. Signing must happen in Foundry/Hardhat/Node or in a secure service.
- Pre-signed tx should be passed in via **environment**, **secure config**, or **API** (e.g. from a deploy service the agent calls).
- Prefer short-lived tokens or one-time deploy keys in CI; rotate and restrict RPC and key access.

---

## Summary

1. **Build** the deployment transaction (bytecode + constructor args) and **sign** it in Foundry, Hardhat, or Node.
2. **Output** the signed serialized tx as hex.
3. **Pass** that hex to DAL as `chain::deploy(chain_id, contract_name, { "raw_transaction": hex })` (or `signed_tx`).
4. DAL submits it and returns the deployed contract address when `http-interface` and chain config are available.

See also: [CHAIN_NAMESPACE_GAPS_AND_FIXES.md](../design/CHAIN_NAMESPACE_GAPS_AND_FIXES.md) (deploy behavior), [SOLIDITY_INTEGRATION_GUIDE.md](../SOLIDITY_INTEGRATION_GUIDE.md) (orchestrating Solidity from DAL).
