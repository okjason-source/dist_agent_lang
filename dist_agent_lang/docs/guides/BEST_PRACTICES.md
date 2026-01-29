# 📘 Best Practices Guide (v1.0.1)

> **📢 Beta Release v1.0.1:** Follow these best practices and test thoroughly. Actively updated with improvements. **Beta testing contributions appreciated!** 🙏

Comprehensive guide to writing secure, efficient, and maintainable **dist_agent_lang** code.

---

## 📋 Table of Contents

1. [Security Best Practices](#security-best-practices)
2. [Performance Optimization](#performance-optimization)
3. [Code Organization](#code-organization)
4. [Error Handling](#error-handling)
5. [Testing Strategies](#testing-strategies)
6. [Gas Optimization](#gas-optimization)
7. [Oracle Integration](#oracle-integration)
8. [Multi-Chain Development](#multi-chain-development)
9. [Common Patterns](#common-patterns)
10. [Anti-Patterns (What to Avoid)](#anti-patterns-what-to-avoid)

---

## 🔒 Security Best Practices

### 1. Always Enable Security Features

**✅ DO:**
```dal
@contract
@reentrancy_guard        // Automatic reentrancy protection
@safe_math               // Automatic overflow/underflow protection
@access_control          // Role-based access control
contract Secure {
    // Your code is protected
}
```

**❌ DON'T:**
```dal
@contract
contract Vulnerable {
    // No security attributes - vulnerable to attacks!
}
```

### 2. Input Validation

**✅ DO:**
```dal
@public
function transfer(address to, uint256 amount) {
    // Validate inputs
    require(to != address(0), "Invalid address");
    require(amount > 0, "Amount must be positive");
    require(amount <= balances[msg.sender], "Insufficient balance");
    
    // Perform transfer
    balances[msg.sender] -= amount;
    balances[to] += amount;
}
```

**❌ DON'T:**
```dal
@public
function transfer(address to, uint256 amount) {
    // No validation - dangerous!
    balances[msg.sender] -= amount;
    balances[to] += amount;
}
```

### 3. State Changes After External Calls

**✅ DO (Checks-Effects-Interactions Pattern):**
```dal
@public
function withdraw(uint256 amount) {
    // 1. Checks
    require(balances[msg.sender] >= amount, "Insufficient balance");
    
    // 2. Effects (update state BEFORE external call)
    balances[msg.sender] -= amount;
    
    // 3. Interactions (external calls last)
    msg.sender.transfer(amount);
}
```

**❌ DON'T:**
```dal
@public
function withdraw(uint256 amount) {
    require(balances[msg.sender] >= amount);
    
    // External call BEFORE state update - reentrancy risk!
    msg.sender.transfer(amount);
    balances[msg.sender] -= amount;
}
```

### 4. Use Events for Important State Changes

**✅ DO:**
```dal
event Transfer(address indexed from, address indexed to, uint256 amount);
event Approval(address indexed owner, address indexed spender, uint256 amount);

@public
function transfer(address to, uint256 amount) {
    // ... transfer logic ...
    
    emit Transfer(msg.sender, to, amount);  // Always emit events
}
```

### 5. Secure Oracle Usage

**✅ DO:**
```dal
@public
function updatePrice() {
    // Multi-source validation with consensus
    let price = oracle::fetch_with_consensus(
        ["chainlink", "uniswap", "band"],
        oracle::create_query("BTC/USD"),
        0.66  // 66% consensus threshold
    );
    
    // Validate freshness
    require(price.timestamp > block.timestamp - 300, "Price too old");
    
    btcPrice = price.data;
}
```

**❌ DON'T:**
```dal
@public
function updatePrice() {
    // Single source, no validation - risky!
    let price = oracle::fetch("chainlink", "BTC/USD");
    btcPrice = price.data;
}
```

---

## ⚡ Performance Optimization

### 1. Minimize Storage Operations

**✅ DO:**
```dal
@public
function batchTransfer(address[] memory recipients, uint256[] memory amounts) {
    uint256 totalAmount = 0;
    
    // Calculate total in memory
    for (uint i = 0; i < amounts.length; i++) {
        totalAmount += amounts[i];
    }
    
    // Single storage read
    require(balances[msg.sender] >= totalAmount);
    
    // Single storage write
    balances[msg.sender] -= totalAmount;
    
    // Update recipients
    for (uint i = 0; i < recipients.length; i++) {
        balances[recipients[i]] += amounts[i];
    }
}
```

**❌ DON'T:**
```dal
@public
function batchTransfer(address[] memory recipients, uint256[] memory amounts) {
    for (uint i = 0; i < recipients.length; i++) {
        // Multiple storage reads/writes - expensive!
        require(balances[msg.sender] >= amounts[i]);
        balances[msg.sender] -= amounts[i];
        balances[recipients[i]] += amounts[i];
    }
}
```

### 2. Use Appropriate Data Types

**✅ DO:**
```dal
// Pack variables to save storage slots
contract Optimized {
    uint128 public price;      // 16 bytes
    uint64 public timestamp;   // 8 bytes
    uint32 public count;       // 4 bytes
    uint32 public flags;       // 4 bytes
    // Total: 32 bytes = 1 storage slot
}
```

**❌ DON'T:**
```dal
contract Wasteful {
    uint256 public price;      // 32 bytes = 1 slot
    uint256 public timestamp;  // 32 bytes = 1 slot
    uint256 public count;      // 32 bytes = 1 slot
    uint256 public flags;      // 32 bytes = 1 slot
    // Total: 128 bytes = 4 storage slots (4x cost!)
}
```

### 3. Cache Expensive Computations

**✅ DO:**
```dal
@public
@view
function getComplexValue() -> uint256 {
    // Cache result
    if (cachedValue != 0 && block.timestamp - cacheTime < 3600) {
        return cachedValue;
    }
    
    // Expensive computation
    uint256 result = expensiveCalculation();
    
    // Update cache
    cachedValue = result;
    cacheTime = block.timestamp;
    
    return result;
}
```

### 4. Use Async for Non-Critical Operations

**✅ DO:**
```dal
@public
@async
async function processData() {
    // Non-blocking async operation
    let result = await externalService::fetchData();
    
    // Process in background
    await processInBackground(result);
}
```

---

## 📁 Code Organization

### 1. Project Structure

**✅ Recommended Structure:**
```
my-project/
├── contracts/
│   ├── core/              # Core business logic
│   │   ├── Token.dal
│   │   └── Marketplace.dal
│   ├── interfaces/        # Contract interfaces
│   │   ├── IERC20.dal
│   │   └── IMarketplace.dal
│   ├── libraries/         # Reusable libraries
│   │   ├── Math.dal
│   │   └── SafeTransfer.dal
│   └── utils/             # Utility contracts
│       └── Ownable.dal
├── tests/
│   ├── unit/
│   ├── integration/
│   └── fixtures/
├── scripts/
│   ├── deploy.sh
│   └── verify.sh
├── docs/
│   └── architecture.md
├── .env.example
├── deployment-order.yaml
└── README.md
```

### 2. Contract Organization

**✅ DO:**
```dal
@contract
@version("1.0.0")
@author("Your Name")
contract WellOrganized {
    // 1. State variables (grouped by visibility)
    // Public state
    uint256 public totalSupply;
    mapping(address => uint256) public balances;
    
    // Private state
    uint256 private _adminFee;
    address private _owner;
    
    // 2. Events
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event FeeUpdated(uint256 oldFee, uint256 newFee);
    
    // 3. Modifiers
    @modifier
    modifier onlyOwner() {
        require(msg.sender == _owner, "Not owner");
        _;
    }
    
    // 4. Constructor
    constructor(uint256 initialSupply) {
        _owner = msg.sender;
        totalSupply = initialSupply;
        balances[msg.sender] = initialSupply;
    }
    
    // 5. External functions
    @public
    function transfer(address to, uint256 amount) {
        // ...
    }
    
    // 6. Public functions
    @public
    @view
    function balanceOf(address account) -> uint256 {
        return balances[account];
    }
    
    // 7. Internal functions
    @private
    function _transfer(address from, address to, uint256 amount) {
        // ...
    }
    
    // 8. Private functions
    @private
    function _calculateFee(uint256 amount) -> uint256 {
        // ...
    }
}
```

### 3. Use Interfaces

**✅ DO:**
```dal
// interfaces/IERC20.dal
@interface
interface IERC20 {
    function transfer(address to, uint256 amount) -> bool;
    function balanceOf(address account) -> uint256;
}

// contracts/MyContract.dal
@contract
contract MyContract {
    IERC20 private token;
    
    constructor(address tokenAddress) {
        token = IERC20(tokenAddress);
    }
    
    @public
    function interact() {
        token.transfer(msg.sender, 100);
    }
}
```

---

## 🚨 Error Handling

### 1. Use Descriptive Error Messages

**✅ DO:**
```dal
@public
function withdraw(uint256 amount) {
    require(amount > 0, "Withdrawal amount must be greater than zero");
    require(balances[msg.sender] >= amount, "Insufficient balance");
    require(!paused, "Contract is paused");
    
    // ... withdrawal logic ...
}
```

**❌ DON'T:**
```dal
@public
function withdraw(uint256 amount) {
    require(amount > 0);  // No message!
    require(balances[msg.sender] >= amount, "Error");  // Too vague!
    
    // ... withdrawal logic ...
}
```

### 2. Use Custom Error Types

**✅ DO:**
```dal
// Define custom errors
error InsufficientBalance(uint256 available, uint256 requested);
error Unauthorized(address caller);
error InvalidAmount(uint256 amount);

@contract
contract ModernErrors {
    @public
    function transfer(address to, uint256 amount) {
        if (balances[msg.sender] < amount) {
            revert InsufficientBalance(balances[msg.sender], amount);
        }
        
        if (amount == 0) {
            revert InvalidAmount(amount);
        }
        
        // ... transfer logic ...
    }
}
```

### 3. Handle External Call Failures

**✅ DO:**
```dal
@public
function callExternal(address target, bytes memory data) {
    (bool success, bytes memory result) = target.call(data);
    
    if (!success) {
        // Handle failure appropriately
        if (result.length > 0) {
            // Revert with the error message from the external call
            assembly {
                revert(add(result, 32), mload(result))
            }
        } else {
            revert("External call failed");
        }
    }
    
    // Process successful result
    processResult(result);
}
```

---

## 🧪 Testing Strategies

### 1. Comprehensive Test Coverage

**✅ DO:**
```dal
// tests/token_tests.dal
@test_suite("Token Tests")
suite TokenTests {
    @test("Should transfer tokens correctly")
    async function testTransfer() {
        let token = await deploy("Token", [1000000]);
        let recipient = getTestAddress(1);
        
        await token.transfer(recipient, 1000);
        
        assert_eq(await token.balanceOf(recipient), 1000);
        assert_eq(await token.balanceOf(deployer), 999000);
    }
    
    @test("Should fail on insufficient balance")
    async function testInsufficientBalance() {
        let token = await deploy("Token", [1000]);
        
        await assert_reverts(
            token.transfer(recipient, 2000),
            "Insufficient balance"
        );
    }
    
    @test("Should fail on zero address")
    async function testZeroAddress() {
        let token = await deploy("Token", [1000]);
        
        await assert_reverts(
            token.transfer(address(0), 100),
            "Invalid address"
        );
    }
}
```

### 2. Property-Based Testing

**✅ DO:**
```dal
@property_test("Token balance invariant")
function prop_totalSupplyInvariant(
    address[] memory accounts,
    uint256[] memory amounts
) {
    // Property: Total supply should never change
    uint256 totalBefore = token.totalSupply();
    
    // Perform random transfers
    for (uint i = 0; i < accounts.length; i++) {
        if (token.balanceOf(accounts[i]) >= amounts[i]) {
            token.transfer(accounts[i], amounts[i]);
        }
    }
    
    uint256 totalAfter = token.totalSupply();
    assert_eq(totalBefore, totalAfter);
}
```

### 3. Integration Testing

**✅ DO:**
```dal
@integration_test("Full marketplace flow")
async function testMarketplaceFlow() {
    // Deploy entire system
    let token = await deploy("Token", [1000000]);
    let nft = await deploy("NFT");
    let marketplace = await deploy("Marketplace", [token.address, nft.address]);
    
    // Setup
    await token.approve(marketplace.address, 10000);
    await nft.mint(seller, 1);
    await nft.approve(marketplace.address, 1);
    
    // Test full flow
    await marketplace.listItem(1, 1000);
    await marketplace.buyItem(1);
    
    // Verify final state
    assert_eq(await nft.ownerOf(1), buyer);
    assert_eq(await token.balanceOf(seller), 1000);
}
```

---

## ⛽ Gas Optimization

### 1. Batch Operations

**✅ DO:**
```dal
@public
function batchMint(address[] memory recipients, uint256[] memory amounts) {
    uint256 length = recipients.length;
    require(length == amounts.length, "Array length mismatch");
    
    for (uint256 i = 0; i < length; i++) {
        _mint(recipients[i], amounts[i]);
    }
}
```

### 2. Use `unchecked` for Safe Operations

**✅ DO:**
```dal
@public
function optimizedLoop() {
    uint256 length = items.length;
    
    for (uint256 i = 0; i < length;) {
        processItem(items[i]);
        
        unchecked {
            ++i;  // i can never overflow in this context
        }
    }
}
```

### 3. Short-Circuit Evaluation

**✅ DO:**
```dal
// Put cheaper checks first
require(amount > 0 && balances[msg.sender] >= amount);
```

---

## 🔮 Oracle Integration

### 1. Multi-Source Validation

**✅ DO:**
```dal
@public
function updatePriceWithConsensus() {
    let price = oracle::fetch_with_consensus(
        ["chainlink", "uniswap_v3", "band_protocol"],
        oracle::create_query("ETH/USD")
            .require_signature(true)
            .with_confirmations(2),
        0.66  // 66% agreement required
    );
    
    require(price.verified, "Oracle signature verification failed");
    require(price.confidence_score >= 0.66, "Insufficient consensus");
    
    ethPrice = price.data;
    lastUpdate = block.timestamp;
}
```

### 2. Price Freshness Checks

**✅ DO:**
```dal
uint256 constant MAX_PRICE_AGE = 300;  // 5 minutes

@public
@view
function getPrice() -> uint256 {
    require(
        block.timestamp - lastUpdate <= MAX_PRICE_AGE,
        "Price data is stale"
    );
    
    return ethPrice;
}
```

---

## 🌍 Multi-Chain Development

### 1. Chain-Agnostic Code

**✅ DO:**
```dal
@contract
@blockchain("ethereum")
@blockchain("polygon")
@blockchain("arbitrum")
contract MultiChain {
    // Use chain-agnostic code
    @public
    function transfer(address to, uint256 amount) {
        // This works on all chains
        balances[msg.sender] -= amount;
        balances[to] += amount;
    }
}
```

### 2. Chain-Specific Logic

**✅ DO:**
```dal
@public
function getGasPrice() -> uint256 {
    if (chain::id() == 1) {
        // Ethereum mainnet
        return tx.gasprice;
    } else if (chain::id() == 137) {
        // Polygon
        return 30 gwei;
    } else {
        return 1 gwei;
    }
}
```

---

## 🎯 Common Patterns

### 1. Pull Over Push for Payments

**✅ DO:**
```dal
mapping(address => uint256) public pendingWithdrawals;

@public
function withdraw() {
    uint256 amount = pendingWithdrawals[msg.sender];
    require(amount > 0, "No funds to withdraw");
    
    pendingWithdrawals[msg.sender] = 0;  // Update first
    msg.sender.transfer(amount);          // Transfer last
}
```

### 2. Emergency Stop Pattern

**✅ DO:**
```dal
bool public paused = false;
address public owner;

@modifier
modifier whenNotPaused() {
    require(!paused, "Contract is paused");
    _;
}

@modifier
modifier onlyOwner() {
    require(msg.sender == owner, "Not owner");
    _;
}

@public
@onlyOwner
function pause() {
    paused = true;
}

@public
@onlyOwner
function unpause() {
    paused = false;
}

@public
@whenNotPaused
function criticalFunction() {
    // Only works when not paused
}
```

---

## ❌ Anti-Patterns (What to Avoid)

### 1. DON'T Use `tx.origin` for Authorization

**❌ DON'T:**
```dal
// Vulnerable to phishing attacks!
require(tx.origin == owner);
```

**✅ DO:**
```dal
require(msg.sender == owner);
```

### 2. DON'T Ignore Return Values

**❌ DON'T:**
```dal
token.transfer(recipient, amount);  // Ignores return value!
```

**✅ DO:**
```dal
bool success = token.transfer(recipient, amount);
require(success, "Transfer failed");
```

### 3. DON'T Use Block Values for Randomness

**❌ DON'T:**
```dal
// Miners can manipulate this!
uint256 random = uint256(blockhash(block.number - 1));
```

**✅ DO:**
```dal
// Use oracle-based randomness
uint256 random = oracle::fetch("chainlink_vrf", "random");
```

---

## 📚 Additional Resources

- [Security Guide](SECURITY_GUIDE.md) - Deep dive into security
- [Performance Guide](PERFORMANCE_GUIDE.md) - Advanced optimization
- [API Reference](API_REFERENCE.md) - Complete stdlib documentation
- [Tutorials](tutorials/) - Learn by building

---

**Next:** [API Reference →](API_REFERENCE.md)

