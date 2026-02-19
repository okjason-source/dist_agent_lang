# Manual MEV Protection Implementation Guide

## Overview

This guide shows how to implement MEV (Maximal Extractable Value) protection **manually in DAL code** without relying on parser attributes. These patterns allow you to harden your services against front-running, back-running, and sandwich attacks.

---

## Strategy 1: Commit-Reveal Pattern

### How It Works

1. **Commit Phase**: User submits a hash of their transaction (details hidden)
2. **Reveal Phase**: User reveals the actual transaction data
3. **Execution**: Transaction executes only after reveal

### Implementation

```dal
@secure
service DeFiService {
    // Storage for commitments
    commitments: map<string, map<string, any>>,
    revealed_transactions: map<string, bool>,
    
    // Commit phase: Submit hash of transaction
    fn commit_swap(
        commitment_hash: string,
        nonce: int,
        deadline: int  // Block number deadline
    ) -> Result<string, string> {
        // Verify caller is authenticated (@secure handles this)
        
        // Store commitment
        let caller = auth::current_caller();
        let commit_id = crypto::hash(format!("{}:{}:{}", caller, commitment_hash, nonce), "SHA256");
        
        self.commitments[commit_id] = {
            "hash": commitment_hash,
            "caller": caller,
            "nonce": nonce,
            "deadline": deadline,
            "timestamp": chain::block_number()
        };
        
        log::audit("commit_swap", {
            "commit_id": commit_id,
            "caller": caller
        }, Some("defi"));
        
        return Ok(commit_id);
    }
    
    // Reveal phase: Reveal actual transaction details
    fn reveal_swap(
        commit_id: string,
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int,
        nonce: int
    ) -> Result<bool, string> {
        // Verify commitment exists
        let commitment = self.commitments.get(commit_id);
        if commitment.is_none() {
            throw "Commitment not found";
        }
        
        let commit = commitment.unwrap();
        
        // Verify deadline hasn't passed
        if chain::block_number() > commit.get("deadline").unwrap_or(0) {
            throw "Commitment expired";
        }
        
        // Verify nonce matches
        if nonce != commit.get("nonce").unwrap_or(0) {
            throw "Invalid nonce";
        }
        
        // Reconstruct and verify commitment hash
        let reveal_data = format!("{}:{}:{}:{}:{}:{}", 
            commit.get("caller").unwrap_or(""),
            token_in,
            token_out,
            amount_in,
            min_amount_out,
            nonce
        );
        
        let expected_hash = crypto::hash(reveal_data, "SHA256");
        if expected_hash != commit.get("hash").unwrap_or("") {
            throw "Commitment hash mismatch";
        }
        
        // Mark as revealed
        self.revealed_transactions[commit_id] = true;
        
        // Execute swap
        return self.execute_swap(token_in, token_out, amount_in, min_amount_out);
    }
    
    // Internal: Execute the actual swap
    fn execute_swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int
    ) -> Result<bool, string> {
        // Your swap logic here
        // At this point, transaction details are public but execution is immediate
        // Front-runners can't see the details during commit phase
        
        return Ok(true);
    }
}
```

### Usage

```dal
// Step 1: User commits (details hidden)
let nonce = 12345;
let token_in = "ETH";
let token_out = "USDC";
let amount_in = 1000;
let min_amount_out = 2000;

let reveal_data = format!("{}:{}:{}:{}:{}", 
    auth::current_caller(),
    token_in, token_out, amount_in, min_amount_out, nonce
);
let commitment_hash = crypto::hash(reveal_data);

let commit_id = defi_service.commit_swap(
    commitment_hash,
    nonce,
    chain::block_number() + 10  // 10 blocks to reveal
);

// Step 2: User reveals (after some blocks)
let result = defi_service.reveal_swap(
    commit_id,
    token_in,
    token_out,
    amount_in,
    min_amount_out,
    nonce
);
```

### Protection Level: ⭐⭐⭐⭐⭐
- ✅ Strong protection against front-running
- ✅ Hides transaction intent during commit phase
- ⚠️ Requires two-phase process

---

## Strategy 2: Time-Delayed Execution

### How It Works

Delay transaction execution by N blocks to reduce front-running window.

### Implementation

```dal
@secure
service DeFiService {
    pending_transactions: map<string, map<string, any>>,
    
    fn submit_delayed_swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        delay_blocks: int
    ) -> Result<string, string> {
        let caller = auth::current_caller();
        let tx_id = crypto::hash(format!("{}:{}:{}", caller, chain::block_number(), amount_in), "SHA256");
        
        // Store transaction with execution block
        self.pending_transactions[tx_id] = {
            "caller": caller,
            "token_in": token_in,
            "token_out": token_out,
            "amount_in": amount_in,
            "execute_at_block": chain::block_number() + delay_blocks,
            "submitted_at": chain::block_number()
        };
        
        log::audit("submit_delayed_swap", {
            "tx_id": tx_id,
            "execute_at": chain::block_number() + delay_blocks
        }, Some("defi"));
        
        return Ok(tx_id);
    }
    
    // Called by anyone after delay period
    fn execute_delayed_swap(tx_id: string) -> Result<bool, string> {
        let tx = self.pending_transactions.get(tx_id);
        if tx.is_none() {
            throw "Transaction not found";
        }
        
        let pending = tx.unwrap();
        
        // Verify execution time has passed
        if chain::block_number() < pending.get("execute_at_block").unwrap_or(0) {
            throw "Execution time not reached";
        }
        
        // Verify not too old (prevent stale transactions)
        let max_age = 100;  // blocks
        if chain::block_number() > pending.get("execute_at_block").unwrap_or(0) + max_age {
            throw "Transaction expired";
        }
        
        // Execute swap
        let result = self.execute_swap(
            pending.get("token_in").unwrap_or(""),
            pending.get("token_out").unwrap_or(""),
            pending.get("amount_in").unwrap_or(0),
            0  // No min amount check for delayed execution
        );
        
        // Clean up
        self.pending_transactions.remove(tx_id);
        
        return result;
    }
}
```

### Protection Level: ⭐⭐⭐
- ✅ Reduces front-running window
- ⚠️ Still vulnerable if execution time is predictable
- ⚠️ Requires separate execution call

---

## Strategy 3: Fair Batch Ordering

### How It Works

Collect transactions in a time window, then execute in random/fair order.

### Implementation

```dal
@secure
service DeFiService {
    batch_pool: map<int, list<map<string, any>>>,
    batch_size: int = 10,
    
    fn submit_to_batch(
        token_in: string,
        token_out: string,
        amount_in: int
    ) -> Result<string, string> {
        let caller = auth::current_caller();
        let current_batch = (chain::block_number() / self.batch_size) * self.batch_size;
        
        // Get or create batch
        let batch = self.batch_pool.get(current_batch).unwrap_or([]);
        
        // Add transaction to batch
        let tx = {
            "caller": caller,
            "token_in": token_in,
            "token_out": token_out,
            "amount_in": amount_in,
            "submitted_at": chain::block_number(),
            "random_seed": crypto::generate_random(16)  // For fair ordering
        };
        
        batch.push(tx);
        self.batch_pool[current_batch] = batch;
        
        let tx_id = crypto::hash(format!("{}:{}:{}", caller, current_batch, amount_in), "SHA256");
        
        // Process batch if full
        if batch.len() >= self.batch_size {
            self.process_batch(current_batch);
        }
        
        return Ok(tx_id);
    }
    
    fn process_batch(batch_id: int) -> Result<bool, string> {
        let batch = self.batch_pool.get(batch_id);
        if batch.is_none() {
            throw "Batch not found";
        }
        
        let transactions = batch.unwrap();
        
        // Sort by random seed for fair ordering
        // (In real implementation, use VRF or block hash for randomness)
        let block_hash = chain::block_hash();
        let random_factor = crypto::hash(block_hash, "SHA256");
        
        // Shuffle transactions based on block hash (fair randomness)
        let shuffled = self.shuffle_by_hash(transactions, random_factor);
        
        // Execute in shuffled order
        for tx in shuffled {
            self.execute_swap(
                tx.get("token_in").unwrap_or(""),
                tx.get("token_out").unwrap_or(""),
                tx.get("amount_in").unwrap_or(0),
                0
            );
        }
        
        // Clean up batch
        self.batch_pool.remove(batch_id);
        
        return Ok(true);
    }
    
    // Helper: Shuffle using hash for deterministic randomness
    fn shuffle_by_hash(
        transactions: list<map<string, any>>,
        seed: string
    ) -> list<map<string, any>> {
        // Simple shuffle based on hash seed
        // In production, use proper VRF or cryptographic shuffle
        let result = [];
        let used_indices = [];
        
        for i in 0..transactions.len() {
            let hash = crypto::hash(format!("{}:{}:{}", seed, i, transactions.len()), "SHA256");
            // Convert hash hex string to integer (use first 16 chars as hex number)
            // Simple implementation: parse first 8 hex chars as u32
            let hash_int = self.hex_to_int(hash);  // Helper function below
            let index = (hash_int % transactions.len());
            
            // Find next unused index
            let mut final_index = index;
            let mut attempts = 0;
            while used_indices.contains(final_index) && attempts < transactions.len() {
                final_index = (final_index + 1) % transactions.len();
                attempts = attempts + 1;
            }
            
            used_indices.push(final_index);
            result.push(transactions[final_index]);
        }
        
        return result;
    }
    
    // Helper: Convert hex string to integer
    fn hex_to_int(hex_str: string) -> int {
        // Take first 8 characters of hex string and parse as hex
        let hex_part = hex_str.substring(0, 8);
        // Simple hex parsing (in production, use proper hex parsing)
        let mut result = 0;
        for i in 0..hex_part.len() {
            let char = hex_part[i];
            let digit = if char >= '0' && char <= '9' {
                char - '0'
            } else if char >= 'a' && char <= 'f' {
                char - 'a' + 10
            } else if char >= 'A' && char <= 'F' {
                char - 'A' + 10
            } else {
                0
            };
            result = result * 16 + digit;
        }
        return result;
    }
}
```

### Protection Level: ⭐⭐⭐⭐
- ✅ Fair ordering prevents manipulation
- ✅ Can use VRF for true randomness
- ⚠️ Requires batching (delayed execution)

---

## Strategy 4: Price Slippage Protection

### How It Works

Use maximum slippage limits and price oracles to prevent MEV exploitation.

### Implementation

```dal
@secure
service DeFiService {
    oracle: OracleService,
    max_slippage_bps: int = 50,  // 0.5% max slippage
    
    fn protected_swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        max_slippage: int  // Basis points (100 = 1%)
    ) -> Result<map<string, any>, string> {
        // Get current price from oracle
        let current_price = self.oracle.get_price(token_in, token_out);
        
        // Calculate expected output
        let expected_out = (amount_in * current_price) / 1000000;  // Assuming 6 decimals
        
        // Execute swap
        let actual_out = self.execute_swap(token_in, token_out, amount_in, 0);
        
        // Verify slippage
        let slippage_bps = ((expected_out - actual_out) * 10000) / expected_out;
        
        if slippage_bps > max_slippage {
            throw format!("Slippage too high: {} bps (max: {} bps)", slippage_bps, max_slippage);
        }
        
        // Also check for sudden price changes (potential MEV attack)
        let new_price = self.oracle.get_price(token_in, token_out);
        let price_change_bps = ((new_price - current_price) * 10000) / current_price;
        
        if price_change_bps > self.max_slippage_bps {
            throw "Suspicious price movement detected - potential MEV attack";
        }
        
        return Ok({
            "amount_in": amount_in,
            "amount_out": actual_out,
            "slippage_bps": slippage_bps,
            "price": current_price
        });
    }
}
```

### Protection Level: ⭐⭐⭐
- ✅ Prevents excessive slippage
- ✅ Detects suspicious price movements
- ⚠️ Requires reliable oracle

---

## Strategy 5: Rate Limiting & Cooldowns

### How It Works

Limit transaction frequency to prevent rapid-fire MEV attacks.

### Implementation

```dal
@secure
service DeFiService {
    last_transaction: map<string, int>,  // caller -> block number
    cooldown_blocks: int = 5,
    
    fn rate_limited_swap(
        token_in: string,
        token_out: string,
        amount_in: int
    ) -> Result<bool, string> {
        let caller = auth::current_caller();
        let current_block = chain::block_number();
        
        // Check cooldown
        let last_tx_block = self.last_transaction.get(caller).unwrap_or(0);
        let blocks_since_last = current_block - last_tx_block;
        
        if blocks_since_last < self.cooldown_blocks {
            throw format!("Cooldown active: {} blocks remaining", 
                self.cooldown_blocks - blocks_since_last);
        }
        
        // Update last transaction
        self.last_transaction[caller] = current_block;
        
        // Execute swap
        return self.execute_swap(token_in, token_out, amount_in, 0);
    }
}
```

### Protection Level: ⭐⭐
- ✅ Prevents rapid-fire attacks
- ⚠️ May inconvenience legitimate users
- ⚠️ Doesn't prevent single-attack MEV

---

## Strategy 6: Combined Multi-Layer Protection

### Best Practice: Combine Multiple Strategies

```dal
@secure
service SecureDeFiService {
    commitments: map<string, map<string, any>>,
    batch_pool: map<int, list<map<string, any>>>,
    oracle: OracleService,
    
    // Multi-layer protected swap
    fn secure_swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int,
        use_commit_reveal: bool,
        use_batching: bool
    ) -> Result<string, string> {
        let caller = auth::current_caller();
        
        if use_commit_reveal {
            // Strategy 1: Commit-reveal
            return self.commit_swap(token_in, token_out, amount_in, min_amount_out);
        } else if use_batching {
            // Strategy 3: Fair batching
            return self.submit_to_batch(token_in, token_out, amount_in);
        } else {
            // Strategy 4: Slippage protection only
            return self.protected_swap(token_in, token_out, amount_in, 50);
        }
    }
    
    // Internal commit
    fn commit_swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int
    ) -> Result<string, string> {
        let nonce = crypto::generate_random(16);
        let reveal_data = format!("{}:{}:{}:{}:{}", 
            auth::current_caller(), token_in, token_out, amount_in, min_amount_out, nonce
        );
        let commitment_hash = crypto::hash(reveal_data, "SHA256");
        
        // Store commitment
        let commit_id = crypto::hash(format!("{}:{}", caller, commitment_hash), "SHA256");
        self.commitments[commit_id] = {
            "hash": commitment_hash,
            "nonce": nonce,
            "deadline": chain::block_number() + 10
        };
        
        return Ok(commit_id);
    }
}
```

---

## Strategy 7: Using Cryptographic Functions

### Available Crypto Functions

DAL provides cryptographic functions you can use:

```dal
// Hash function
        let hash = crypto::hash("data", "SHA256");
let hash_sha512 = crypto::hash("data", "SHA512");

// Random number generation
let random_bytes = crypto::generate_random(32);  // 32 bytes
let random_id = crypto::generate_random_bytes(16);

// Sign/verify (if available)
let signature = crypto::sign(data, private_key);
let valid = crypto::verify(data, signature, public_key);
```

### Example: Commit-Reveal with Crypto

```dal
fn create_commitment(
    token_in: string,
    token_out: string,
    amount: int
) -> map<string, any> {
    // Generate random nonce
    let nonce = crypto::generate_random(16);  // 16 bytes random
    
    // Create commitment data
    let data = format!("{}:{}:{}:{}:{}", 
        auth::current_caller(),
        token_in,
        token_out,
        amount,
        nonce
    );
    
    // Hash commitment
    let commitment_hash = crypto::hash(data, "SHA256");
    
    return {
        "commitment_hash": commitment_hash,
        "nonce": nonce,
        "data": data  // Keep secret until reveal
    };
}
```

---

## Best Practices

### 1. **Choose Strategy Based on Use Case**

- **High-value transactions**: Use commit-reveal
- **DEX swaps**: Use fair batching + slippage protection
- **Admin operations**: Use time delays
- **General operations**: Use slippage protection + rate limiting

### 2. **Combine Multiple Strategies**

```dal
// Best: Multi-layer protection
@secure
service BestPracticeService {
    fn ultra_secure_swap(...) {
        // 1. Commit-reveal (hide intent)
        // 2. Fair batching (fair ordering)
        // 3. Slippage protection (prevent exploitation)
        // 4. Rate limiting (prevent spam)
    }
}
```

### 3. **Use Oracles for Price Validation**

Always verify prices against trusted oracles before executing swaps.

### 4. **Implement Time Windows**

Set deadlines for commitments and reveals to prevent stale transactions.

### 5. **Log Everything**

Use `log::audit()` to track all MEV protection events for analysis.

---

## Comparison Table

| Strategy | Protection Level | Complexity | Delay | Use Case |
|----------|-----------------|------------|-------|----------|
| Commit-Reveal | ⭐⭐⭐⭐⭐ | High | Medium | High-value swaps |
| Time Delay | ⭐⭐⭐ | Low | High | Admin operations |
| Fair Batching | ⭐⭐⭐⭐ | Medium | Medium | DEX swaps |
| Slippage Protection | ⭐⭐⭐ | Low | None | All swaps |
| Rate Limiting | ⭐⭐ | Low | Low | Spam prevention |
| Combined | ⭐⭐⭐⭐⭐ | High | Medium | Critical operations |

---

## Example: Complete Protected DEX Service

```dal
@secure
@trust("decentralized")
@chain("ethereum")
service ProtectedDEX {
    commitments: map<string, map<string, any>>,
    batch_pool: map<int, list<map<string, any>>>,
    oracle: OracleService,
    
    // Main entry point with multiple protection layers
    fn swap(
        token_in: string,
        token_out: string,
        amount_in: int,
        min_amount_out: int,
        protection_mode: string  // "commit_reveal", "batch", "direct"
    ) -> Result<map<string, any>, string> {
        // Always check slippage
        let current_price = self.oracle.get_price(token_in, token_out);
        let expected_out = (amount_in * current_price) / 1000000;
        
        if expected_out < min_amount_out {
            throw "Slippage too high";
        }
        
        // Route to protection mechanism
        if protection_mode == "commit_reveal" {
            return self.commit_swap(token_in, token_out, amount_in, min_amount_out);
        } else if protection_mode == "batch" {
            return self.submit_to_batch(token_in, token_out, amount_in);
        } else {
            // Direct swap with slippage protection only
            return self.direct_swap(token_in, token_out, amount_in, min_amount_out);
        }
    }
    
    // Implementation methods (as shown above)
    // ...
}
```

---

## Conclusion

Before parser-level `@mev_protection` attributes, developers can already implement robust MEV protection using:

1. **Commit-reveal patterns** (strongest protection)
2. **Fair batching** (good for DEX)
3. **Slippage protection** (essential for all swaps)
4. **Time delays** (for admin operations)
5. **Rate limiting** (spam prevention)

**Recommendation**: Combine multiple strategies for maximum protection, especially for high-value DeFi operations.

---

**Last Updated**: February 2026
**Status**: Implementation Guide
