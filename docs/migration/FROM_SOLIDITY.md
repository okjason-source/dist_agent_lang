# 🔄 Migration Guide: Solidity → dist_agent_lang (v1.0.2)

> **📢 Beta Release v1.0.2:** Actively developed with consistent updates. Test migrated contracts thoroughly. **Beta testing contributions appreciated!** 🙏

Complete guide to migrating your Solidity contracts to **dist_agent_lang**.

---

## 📋 Table of Contents

1. [Why Migrate?](#why-migrate)
2. [Key Differences](#key-differences)
3. [Syntax Comparison](#syntax-comparison)
4. [Step-by-Step Migration](#step-by-step-migration)
5. [Common Patterns](#common-patterns)
6. [Tools & Automation](#tools--automation)

---

## 🎯 Why Migrate?

### Advantages of dist_agent_lang

| Feature | Solidity | DAL |
|---------|----------|-----|
| **Multi-Chain** | EVM only | Ethereum, Polygon, Solana, Arbitrum, + more |
| **Security** | Manual (OpenZeppelin) | Built-in (@reentrancy_guard, @safe_math) |
| **Oracle Access** | External contracts | Native (oracle::) |
| **Async/Await** | ❌ | ✅ |
| **Type Safety** | Medium | Strong |
| **Testing** | Requires Hardhat/Truffle | Built-in testing framework |
| **Deployment** | Complex | Single command multi-chain |

---

## 🔑 Key Differences

### 1. Attributes vs Pragmas

**Solidity:**
```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
```

**DAL:**
```dal
@contract
@version("1.0.0")
@blockchain("ethereum")
```

### 2. Built-in Security

**Solidity:**
```solidity
// Must use external libraries
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract MyContract is ReentrancyGuard {
    function withdraw() external nonReentrant {
        // ...
    }
}
```

**DAL:**
```dal
@contract
@reentrancy_guard        // Automatic!
@safe_math               // Automatic overflow protection!
contract MyContract {
    @public
    function withdraw() {
        // Automatically protected
    }
}
```

### 3. Function Visibility

**Solidity:**
```solidity
function myPublic() public { }
function myExternal() external { }
function myInternal() internal { }
function myPrivate() private { }
```

**DAL:**
```dal
@public
function myPublic() { }

@public      // external = public in DAL
function myExternal() { }

@private
function myPrivate() { }
```

### 4. View/Pure Functions

**Solidity:**
```solidity
function getValue() public view returns (uint256) {
    return value;
}

function calculate(uint256 x) public pure returns (uint256) {
    return x * 2;
}
```

**DAL:**
```dal
@public
@view
function getValue() -> uint256 {
    return value;
}

@public
@view
function calculate(uint256 x) -> uint256 {
    return x * 2;
}
```

### 5. Events

**Solidity:**
```solidity
event Transfer(address indexed from, address indexed to, uint256 value);

emit Transfer(msg.sender, recipient, amount);
```

**DAL:**
```dal
event Transfer(address indexed from, address indexed to, uint256 value);

emit Transfer(msg.sender, recipient, amount);  // Same syntax!
```

---

## 📊 Syntax Comparison

### Variable Declarations

| Solidity | DAL | Notes |
|----------|-----|-------|
| `uint256 public value;` | `uint256 public value;` | Identical |
| `address public owner;` | `address public owner;` | Identical |
| `mapping(address => uint256) balances;` | `mapping(address => uint256) balances;` | Identical |
| `string memory text;` | `string memory text;` | Identical |

### Control Flow

| Solidity | DAL | Notes |
|----------|-----|-------|
| `if (condition) { }` | `if (condition) { }` | Identical |
| `for (uint i = 0; i < length; i++) { }` | `for (uint i = 0; i < length; i++) { }` | Identical |
| `while (condition) { }` | `while (condition) { }` | Identical |
| `require(condition, "error");` | `require(condition, "error");` | Identical |
| `revert("error");` | `revert("error");` | Identical |

### Special Variables

| Solidity | DAL | Notes |
|----------|-----|-------|
| `msg.sender` | `msg.sender` | Identical |
| `msg.value` | `msg.value` | Identical |
| `block.timestamp` | `block.timestamp` or `chain::timestamp()` | Both work |
| `block.number` | `block.number` or `chain::block_number()` | Both work |
| `address(this)` | `address(this)` | Identical |

---

## 🔧 Step-by-Step Migration

### Example: ERC-20 Token

#### Original Solidity Contract

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract MyToken is ERC20, ReentrancyGuard, Ownable {
    uint256 public maxSupply = 1000000 * 10**18;
    
    constructor() ERC20("MyToken", "MTK") {
        _mint(msg.sender, 100000 * 10**18);
    }
    
    function mint(address to, uint256 amount) external onlyOwner {
        require(totalSupply() + amount <= maxSupply, "Exceeds max supply");
        _mint(to, amount);
    }
    
    function burn(uint256 amount) external {
        _burn(msg.sender, amount);
    }
}
```

#### Migrated DAL Contract

```dal
@contract
@version("1.0.0")
@blockchain("ethereum")
@blockchain("polygon")      // Now multi-chain!
@reentrancy_guard            // Built-in!
@safe_math                   // Built-in overflow protection!
contract MyToken {
    // State
    string public name = "MyToken";
    string public symbol = "MTK";
    uint8 public decimals = 18;
    uint256 public totalSupply;
    uint256 public maxSupply = 1000000 * 10**18;
    address public owner;
    
    mapping(address => uint256) public balances;
    mapping(address => mapping(address => uint256)) public allowances;
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);
    
    // Modifiers
    @modifier
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    // Constructor
    constructor() {
        owner = msg.sender;
        totalSupply = 100000 * 10**18;
        balances[owner] = totalSupply;
        emit Transfer(address(0), owner, totalSupply);
    }
    
    // ERC-20 Functions
    @public
    @view
    function balanceOf(address account) -> uint256 {
        return balances[account];
    }
    
    @public
    function transfer(address to, uint256 amount) -> bool {
        require(to != address(0), "Invalid address");
        require(balances[msg.sender] >= amount, "Insufficient balance");
        
        balances[msg.sender] -= amount;
        balances[to] += amount;
        
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    @public
    function approve(address spender, uint256 amount) -> bool {
        allowances[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }
    
    @public
    function transferFrom(address from, address to, uint256 amount) -> bool {
        require(balances[from] >= amount, "Insufficient balance");
        require(allowances[from][msg.sender] >= amount, "Insufficient allowance");
        
        balances[from] -= amount;
        balances[to] += amount;
        allowances[from][msg.sender] -= amount;
        
        emit Transfer(from, to, amount);
        return true;
    }
    
    // Admin Functions
    @public
    @onlyOwner
    function mint(address to, uint256 amount) {
        require(totalSupply + amount <= maxSupply, "Exceeds max supply");
        
        totalSupply += amount;
        balances[to] += amount;
        
        emit Transfer(address(0), to, amount);
    }
    
    @public
    function burn(uint256 amount) {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        
        totalSupply -= amount;
        balances[msg.sender] -= amount;
        
        emit Transfer(msg.sender, address(0), amount);
    }
}
```

### Key Changes Made:

1. ✅ **Removed imports** - Security features are built-in
2. ✅ **Added attributes** - `@contract`, `@reentrancy_guard`, `@safe_math`
3. ✅ **Multi-chain support** - Added `@blockchain` attributes
4. ✅ **Implemented ERC-20** - No inheritance needed
5. ✅ **Same syntax** - Most Solidity code works as-is!

---

## 🎨 Common Patterns

### Pattern 1: Ownable

**Solidity (OpenZeppelin):**
```solidity
import "@openzeppelin/contracts/access/Ownable.sol";

contract MyContract is Ownable {
    function adminFunction() external onlyOwner {
        // ...
    }
}
```

**DAL:**
```dal
@contract
@access_control
contract MyContract {
    address public owner;
    
    constructor() {
        owner = msg.sender;
    }
    
    @modifier
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    @public
    @onlyOwner
    function adminFunction() {
        // ...
    }
}
```

### Pattern 2: Pausable

**Solidity:**
```solidity
import "@openzeppelin/contracts/security/Pausable.sol";

contract MyContract is Pausable {
    function criticalFunction() external whenNotPaused {
        // ...
    }
}
```

**DAL:**
```dal
@contract
contract MyContract {
    bool public paused = false;
    
    @modifier
    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }
    
    @public
    @whenNotPaused
    function criticalFunction() {
        // ...
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
}
```

### Pattern 3: SafeERC20

**Solidity:**
```solidity
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

using SafeERC20 for IERC20;

function transferTokens(IERC20 token, address to, uint256 amount) external {
    token.safeTransfer(to, amount);
}
```

**DAL:**
```dal
@contract
@safe_math  // Built-in safe operations!
contract MyContract {
    @public
    function transferTokens(address token, address to, uint256 amount) {
        // Call token contract
        (bool success,) = token.call(
            abi.encodeWithSignature("transfer(address,uint256)", to, amount)
        );
        require(success, "Transfer failed");
    }
}
```

---

## 🛠️ Tools & Automation

### Automated Conversion Tool ✅ **Available Now**

```bash
# Convert Solidity to DAL
dist_agent_lang convert MyContract.sol --output MyContract.dal
# or
dist_agent_lang convert MyContract.sol -o MyContract.dal

# Analyze compatibility
dist_agent_lang analyze MyContract.sol

# Test migrated contract
dist_agent_lang test MyContract.dal
```

**Note**: The conversion tool is in active development. It handles common Solidity patterns but may require manual review for complex contracts. Always test converted contracts thoroughly!

### Manual Checklist

- [ ] Remove `pragma solidity` statements
- [ ] Add `@contract` attribute
- [ ] Add security attributes (`@reentrancy_guard`, `@safe_math`)
- [ ] Remove OpenZeppelin imports
- [ ] Implement security features directly
- [ ] Add `@blockchain` attributes for multi-chain
- [ ] Update function visibility with `@public`/@private`
- [ ] Add `@view` to read-only functions
- [ ] Test thoroughly!
- [ ] Deploy to testnet first

---

## 📈 Migration Timeline

| Contract Size | Estimated Time | Difficulty |
|---------------|----------------|------------|
| Simple (< 100 lines) | 1-2 hours | Easy |
| Medium (100-500 lines) | 4-8 hours | Medium |
| Complex (500+ lines) | 1-3 days | Hard |

---

## 🎓 Learning Resources

1. **[Quick Start Guide](../QUICK_START.md)** - Get familiar with DAL
2. **[Best Practices](../BEST_PRACTICES.md)** - Learn DAL patterns
3. **[API Reference](../API_REFERENCE.md)** - Explore new features
4. **[Tutorials](../tutorials/)** - Build real projects

---

## 💡 Pro Tips

1. **Start with tests** - Migrate tests first to verify behavior
2. **Use built-in security** - Remove dependencies on external libraries
3. **Leverage multi-chain** - Deploy to multiple networks with one contract
4. **Add oracle integration** - Use native oracle support for price feeds
5. **Test extensively** - Test on testnets before mainnet

---

## 🆘 Common Issues

### Issue 1: Missing OpenZeppelin Imports

**Problem:** Contract relies heavily on OpenZeppelin
**Solution:** DAL has built-in equivalents - use attributes and native features

### Issue 2: Complex Inheritance

**Problem:** Multiple inheritance levels
**Solution:** Flatten the contract and use composition

### Issue 3: Assembly Code

**Problem:** Inline assembly (Yul)
**Solution:** Most assembly can be replaced with DAL's built-in functions

---

## 📞 Get Help

- **Discord**: [Join Community](#)
- **GitHub Issues**: [Report Problems](https://github.com/your-org/dist_agent_lang/issues)
- **Documentation**: [Full Docs](../Documentation.md)

---

**Next:** [Migration from Rust →](FROM_RUST.md)

