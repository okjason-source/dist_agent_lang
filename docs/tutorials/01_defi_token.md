# Tutorial 1: Building a DeFi Token with Oracle Integration (v1.0.1)

> **📢 Beta Release v1.0.1:** Test thoroughly in development environments. **Beta testing feedback appreciated!** 🙏

Learn to build a production-ready DeFi token with price feeds from oracles.

**Time**: 30 minutes  
**Difficulty**: Beginner  
**Prerequisites**: Basic programming knowledge

---

## 🎯 What You'll Build

A DeFi token that:
- Implements ERC-20 standard
- Uses oracle price feeds for dynamic pricing
- Includes burn/mint mechanics
- Has built-in security features
- Supports multi-chain deployment

---

## 📦 Setup

```bash
mkdir defi-token && cd defi-token
touch DefiToken.dal
```

---

## Step 1: Basic Token Structure

```dal
// DefiToken.dal
@contract
@blockchain("ethereum")
@blockchain("polygon")
@version("1.0.0")
@reentrancy_guard
@safe_math
contract DefiToken {
    // State variables
    string public name = "DeFi Token";
    string public symbol = "DFT";
    uint8 public decimals = 18;
    uint256 public totalSupply;
    
    // Balances
    mapping(address => uint256) public balances;
    mapping(address => mapping(address => uint256)) public allowances;
    
    // Owner
    address public owner;
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 amount);
    event Approval(address indexed owner, address indexed spender, uint256 amount);
    event Mint(address indexed to, uint256 amount);
    event Burn(address indexed from, uint256 amount);
    
    // Constructor
    constructor(uint256 initialSupply) {
        owner = msg.sender;
        totalSupply = initialSupply * (10 ** uint256(decimals));
        balances[owner] = totalSupply;
        
        emit Transfer(address(0), owner, totalSupply);
    }
}
```

---

## Step 2: Add Core ERC-20 Functions

```dal
    @public
    @view
    function balanceOf(address account) -> uint256 {
        return balances[account];
    }
    
    @public
    function transfer(address to, uint256 amount) -> bool {
        require(to != address(0), "Cannot transfer to zero address");
        require(balances[msg.sender] >= amount, "Insufficient balance");
        
        balances[msg.sender] -= amount;
        balances[to] += amount;
        
        emit Transfer(msg.sender, to, amount);
        return true;
    }
    
    @public
    function approve(address spender, uint256 amount) -> bool {
        require(spender != address(0), "Cannot approve zero address");
        
        allowances[msg.sender][spender] = amount;
        
        emit Approval(msg.sender, spender, amount);
        return true;
    }
    
    @public
    function transferFrom(address from, address to, uint256 amount) -> bool {
        require(from != address(0), "Cannot transfer from zero address");
        require(to != address(0), "Cannot transfer to zero address");
        require(balances[from] >= amount, "Insufficient balance");
        require(allowances[from][msg.sender] >= amount, "Insufficient allowance");
        
        balances[from] -= amount;
        balances[to] += amount;
        allowances[from][msg.sender] -= amount;
        
        emit Transfer(from, to, amount);
        return true;
    }
```

---

## Step 3: Add Oracle-Based Pricing

```dal
    // Oracle state
    uint256 public currentPrice;  // Price in USD (scaled by 1e18)
    uint256 public lastPriceUpdate;
    uint256 constant PRICE_MAX_AGE = 300;  // 5 minutes
    
    @public
    function updatePrice() {
        // Fetch price from multiple oracles for security
        let priceResponse = oracle::fetch_with_consensus(
            ["chainlink", "uniswap_v3", "band"],
            oracle::create_query("DFT/USD")
                .require_signature(true)
                .with_confirmations(2),
            0.66  // 66% consensus required
        );
        
        // Validate response
        require(priceResponse.verified, "Oracle signature verification failed");
        require(priceResponse.confidence_score >= 0.66, "Insufficient oracle consensus");
        
        // Update price
        currentPrice = priceResponse.data;
        lastPriceUpdate = block.timestamp;
        
        log::info("Price updated: " + string::from_int(currentPrice));
    }
    
    @public
    @view
    function getPrice() -> uint256 {
        require(
            block.timestamp - lastPriceUpdate <= PRICE_MAX_AGE,
            "Price data is stale"
        );
        return currentPrice;
    }
    
    @public
    @view
    function calculateValue(uint256 tokenAmount) -> uint256 {
        // Calculate USD value of token amount
        return (tokenAmount * getPrice()) / (10 ** uint256(decimals));
    }
```

---

## Step 4: Add Burn/Mint Mechanics

```dal
    @modifier
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner can call this function");
        _;
    }
    
    @public
    @onlyOwner
    function mint(address to, uint256 amount) {
        require(to != address(0), "Cannot mint to zero address");
        
        totalSupply += amount;
        balances[to] += amount;
        
        emit Mint(to, amount);
        emit Transfer(address(0), to, amount);
        
        log::info("Minted " + string::from_int(amount) + " tokens to " + to);
    }
    
    @public
    function burn(uint256 amount) {
        require(balances[msg.sender] >= amount, "Insufficient balance to burn");
        
        totalSupply -= amount;
        balances[msg.sender] -= amount;
        
        emit Burn(msg.sender, amount);
        emit Transfer(msg.sender, address(0), amount);
        
        log::info("Burned " + string::from_int(amount) + " tokens");
    }
```

---

## Step 5: Add Pausable Feature

```dal
    bool public paused = false;
    
    event Paused(address account);
    event Unpaused(address account);
    
    @modifier
    modifier whenNotPaused() {
        require(!paused, "Contract is paused");
        _;
    }
    
    @public
    @onlyOwner
    function pause() {
        paused = true;
        emit Paused(msg.sender);
    }
    
    @public
    @onlyOwner
    function unpause() {
        paused = false;
        emit Unpaused(msg.sender);
    }
    
    // Update transfer to check paused state
    @public
    @whenNotPaused
    function transfer(address to, uint256 amount) -> bool {
        // ... existing transfer code ...
    }
```

---

## Step 6: Compile & Test

```bash
# Compile
dal compile DefiToken.dal

# Run tests
dal test DefiToken.dal

# Start local testnet
dal testnet start

# Deploy
dal deploy DefiToken.dal --network local --constructor 1000000
```

---

## Step 7: Write Tests

Create `tests/defi_token_test.dal`:

```dal
@test_suite("DeFi Token Tests")
suite DefiTokenTests {
    @test("Should deploy with correct initial supply")
    async function testInitialSupply() {
        let token = await deploy("DefiToken", [1000000]);
        let supply = await token.totalSupply();
        
        assert_eq(supply, 1000000 * 10**18);
    }
    
    @test("Should transfer tokens correctly")
    async function testTransfer() {
        let token = await deploy("DefiToken", [1000000]);
        let recipient = getTestAddress(1);
        
        await token.transfer(recipient, 1000);
        
        let balance = await token.balanceOf(recipient);
        assert_eq(balance, 1000);
    }
    
    @test("Should update price from oracles")
    async function testPriceUpdate() {
        let token = await deploy("DefiToken", [1000000]);
        
        await token.updatePrice();
        let price = await token.getPrice();
        
        assert_gt(price, 0);
    }
    
    @test("Should mint new tokens")
    async function testMint() {
        let token = await deploy("DefiToken", [1000000]);
        let recipient = getTestAddress(1);
        
        let initialSupply = await token.totalSupply();
        await token.mint(recipient, 5000);
        let newSupply = await token.totalSupply();
        
        assert_eq(newSupply, initialSupply + 5000);
    }
    
    @test("Should burn tokens")
    async function testBurn() {
        let token = await deploy("DefiToken", [1000000]);
        
        let initialSupply = await token.totalSupply();
        await token.burn(1000);
        let newSupply = await token.totalSupply();
        
        assert_eq(newSupply, initialSupply - 1000);
    }
}
```

---

## Step 8: Deploy to Production

```bash
# Deploy to multiple chains
dal deploy DefiToken.dal \
  --networks ethereum,polygon,arbitrum \
  --constructor 1000000 \
  --verify

# Output:
# ✅ Deployed to Ethereum: 0x1234...
# ✅ Deployed to Polygon: 0x5678...
# ✅ Deployed to Arbitrum: 0x9abc...
```

---

## 🎉 Congratulations!

You've built a production-ready DeFi token with:
- ✅ ERC-20 standard compliance
- ✅ Oracle price feeds with multi-source validation
- ✅ Burn/mint mechanics
- ✅ Pausable functionality
- ✅ Multi-chain support
- ✅ Built-in security (reentrancy guard, safe math)

---

## 📚 Next Steps

1. **Tutorial 2**: [NFT Marketplace](02_nft_marketplace.md)
2. **Tutorial 3**: [Cross-Chain Bridge](03_cross_chain_bridge.md)
3. **Deploy to mainnet**: See [Deployment Guide](../DEPLOYMENT_GUIDE.md)

---

## 💡 Pro Tips

1. **Always test on testnet first**
2. **Use multi-source oracle validation** for security
3. **Implement emergency pause** for critical contracts
4. **Monitor price data freshness**
5. **Set up alerts** for unusual activity

---

**Full code**: [examples/defi_token/](../../examples/defi_token/)

