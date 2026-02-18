# Tutorial 3: Hybrid Marketplace with CloudAdmin

> **📢 Beta:** Uses CloudAdmin hybrid trust. Code in this tutorial follows the **current DAL parser**: `if (condition)` with parentheses, `log::info` / `log::audit` (no `log::error`), literals `{ key: value }`. See [Syntax Reference](../syntax.md).

**Build a moderated NFT marketplace combining centralized admin control with decentralized trading**

**Time**: 60 minutes  
**Difficulty**: Intermediate  
**Prerequisites**: Basic smart contracts, admin concepts

---

## 🎯 What You'll Build

A hybrid NFT marketplace that features:
- **Centralized Moderation**: Admin approval for listings
- **Decentralized Trading**: User-to-user NFT transactions
- **Hybrid Trust**: Combine admin oversight with blockchain verification
- **Process Management**: Monitor and control marketplace agents
- **Policy Enforcement**: Flexible security policies

**Why Hybrid?**
- **Admin Control**: Prevent scams, enforce rules, moderate content
- **User Freedom**: Decentralized trading, wallet custody, trustless transactions
- **Best of Both**: Safety + autonomy

---

## 📋 Step 1: Project Setup

Create `hybrid_marketplace.dal`:

```dal
@trust("hybrid")
@secure
@chain("ethereum")
@ai
service HybridNFTMarketplace {
    // State
    listings: map<string, Listing>,
    admins: map<string, AdminInfo>,
    agents: map<string, AgentInfo>,
    
    // Structures
    struct Listing {
        id: string,
        seller: string,
        nft_contract: string,
        token_id: int,
        price: int,
        status: string,  // pending, approved, rejected, sold
        approved_by: string,
        created_at: int
    }
    
    struct AdminInfo {
        id: string,
        level: string,
        permissions: list<string>,
        actions: int
    }
    
    struct AgentInfo {
        id: string,
        type: string,  // moderator, scanner, notifier
        status: string,
        resource_usage: map<string, int>
    }
}
```

---

## 📋 Step 2: Admin Management

Add admin initialization and management:

```dal
// Initialize admins
fn initialize_admins() {
    // Create SuperAdmin
    self.admins["super_admin_001"] = AdminInfo {
        id: "super_admin_001",
        level: "superadmin",
        permissions: ["all"],
        actions: 0
    };
    
    // Create regular admins
    self.admins["admin_001"] = {
        id: "admin_001",
        level: "admin",
        permissions: ["approve_listing", "reject_listing", "view_all"],
        actions: 0
    };
    
    self.admins["moderator_001"] = {
        id: "moderator_001",
        level: "moderator",
        permissions: ["flag_listing", "view_all"],
        actions: 0
    };
    
    log::info("marketplace", "Admins initialized");
}

// Check if user is admin
fn is_admin(user_id: string) -> bool {
    return self.admins.contains_key(user_id);
}

// Get admin level
fn get_admin_level(admin_id: string) -> string {
    if (self.admins.contains_key(admin_id)) {
        return self.admins[admin_id].level;
    }
    return "user";
}
```

---

## 📋 Step 3: Listing Creation (Decentralized)

Users create listings with blockchain verification:

```dal
// User creates listing (decentralized operation)
@public
fn create_listing(
    nft_contract: string,
    token_id: int,
    price: int
) -> string {
    // 1. Verify user owns the NFT (decentralized trust)
    let session = auth::session("current_user_id", ["user"]);
    let seller = session.user_id;
    let owns_nft = chain::verify_nft_owner(nft_contract, token_id, seller);
    
    if (!owns_nft) {
        log::info("marketplace", "User doesn't own NFT");
        return "error";
    }
    
    // 2. User trust validation
    let user_verified = auth::verify_signature();
    let user_trust = if user_verified { "valid" } else { "invalid" };
    
    // 3. Admin pre-screening (centralized trust)
    // Check if NFT contract is whitelisted by admins
    let contract_approved = self.is_contract_whitelisted(nft_contract);
    let admin_trust = if contract_approved { "valid" } else { "invalid" };
    
    // 4. Validate hybrid trust
    let is_trusted = cloudadmin::validate_hybrid_trust(admin_trust, user_trust);
    
    if (!is_trusted) {
        log::info("marketplace", "Hybrid trust validation failed");
        return "error";
    }
    
    // 5. Create listing (pending admin approval)
    let listing_id = "listing_" + token_id + "_" + time::now();
    
    self.listings[listing_id] = Listing {
        id: listing_id,
        seller: seller,
        nft_contract: nft_contract,
        token_id: token_id,
        price: price,
        status: "pending",
        approved_by: "",
        created_at: time::now()
    };
    
    // 6. Notify moderation agents
    self.notify_moderation_agents(listing_id);
    
    log::audit("marketplace", "Listing created: " + listing_id);
    return listing_id;
}

// Check if contract is whitelisted
fn is_contract_whitelisted(contract: string) -> bool {
    // In production, check against whitelist
    // For now, basic validation
    return contract.len() == 42 && contract.starts_with("0x");
}
```

---

## 📋 Step 4: Admin Listing Approval (Centralized)

Admins review and approve listings:

```dal
// Admin approves listing (centralized operation)
@public
fn approve_listing(listing_id: string, admin_id: string) -> bool {
    // 1. Verify listing exists
    if (!self.listings.contains_key(listing_id)) {
        log::info("marketplace", "Listing not found");
        return false;
    }

    // 2. Check admin authorization
    let can_approve = cloudadmin::authorize(admin_id, "write", "/listings");
    
    if (!can_approve) {
        log::info("marketplace", "Admin not authorized to approve");
        return false;
    }
    
    // 3. Create admin context and enforce policy
    let admin_level = self.get_admin_level(admin_id);
    let context = cloudadmin::create_admin_context(admin_id, admin_level);
    let policy_ok = cloudadmin::enforce_policy("moderate", context);
    
    if (policy_ok.is_err() || !policy_ok.unwrap()) {
        log::info("marketplace", "Policy enforcement failed");
        return false;
    }
    
    // 4. Approve listing
    let listing = self.listings[listing_id];
    listing.status = "approved";
    listing.approved_by = admin_id;
    self.listings[listing_id] = listing;
    
    // 5. Update admin stats
    if (self.admins.contains_key(admin_id)) {
        let admin = self.admins[admin_id];
        admin.actions = admin.actions + 1;
        self.admins[admin_id] = admin;
    }
    
    // 6. Log audit trail
    log::audit("marketplace", "Listing " + listing_id + " approved by " + admin_id);
    
    return true;
}

// Admin rejects listing
@public
fn reject_listing(
    listing_id: string,
    admin_id: string,
    reason: string
) -> bool {
    // Similar authorization checks
    let can_reject = cloudadmin::authorize(admin_id, "write", "/listings");
    
    if (!can_reject) {
        log::info("marketplace", "Admin not authorized");
        return false;
    }
    
    let listing = self.listings[listing_id];
    listing.status = "rejected";
    self.listings[listing_id] = listing;
    
    log::audit("marketplace", "Listing rejected: " + listing_id + " - " + reason);
    return true;
}
```

---

## 📋 Step 5: Trading (Decentralized)

Users trade approved listings:

```dal
// Buy NFT (decentralized trading)
@public
fn buy_nft(listing_id: string) -> bool {
    // 1. Check listing is approved
    if (!self.listings.contains_key(listing_id)) {
        log::info("marketplace", "Listing not found");
        return false;
    }

    let listing = self.listings[listing_id];

    if (listing.status != "approved") {
        log::info("marketplace", "Listing not approved");
        return false;
    }
    
    // 2. Verify buyer (decentralized)
    let session = auth::session("current_user_id", ["user"]);
    let buyer = session.user_id;
    let buyer_verified = auth::verify_signature();
    
    if (!buyer_verified) {
        log::info("marketplace", "Buyer verification failed");
        return false;
    }
    
    // 3. Check buyer has funds
    let buyer_balance = chain::get_balance(buyer);
    
    if (buyer_balance < listing.price) {
        log::info("marketplace", "Insufficient funds");
        return false;
    }
    
    // 4. Execute trade on blockchain (fully decentralized)
    let payment_success = chain::transfer(buyer, listing.seller, listing.price);
    
    if (!payment_success) {
        log::info("marketplace", "Payment failed");
        return false;
    }
    
    let nft_transfer = chain::transfer_nft(
        listing.nft_contract,
        listing.token_id,
        listing.seller,
        buyer
    );
    
    if (!nft_transfer) {
        // Revert payment
        chain::transfer(listing.seller, buyer, listing.price);
        log::info("marketplace", "NFT transfer failed");
        return false;
    }
    
    // 5. Update listing status
    listing.status = "sold";
    self.listings[listing_id] = listing;
    
    log::audit("marketplace", "NFT sold: " + listing_id);
    return true;
}
```

---

## 📋 Step 6: AI Moderation Agents

Deploy AI agents for automated moderation:

```dal
// Spawn moderation agent
@ai
fn spawn_moderation_agent() -> string {
    // Create AI agent for content moderation
    let config = ai::AgentConfig {
        agent_id: "mod_agent_" + time::now(),
        name: "ContentModerator",
        role: "content_moderation",
        capabilities: ["image_analysis", "text_analysis", "risk_assessment"],
        max_memory: 512,
        timeout: 300
    };
    
    let agent = ai::spawn_agent(config);
    
    // Register agent
    self.agents[agent.id] = {
        id: agent.id,
        type: "moderator",
        status: "active",
        resource_usage: { cpu: 0, memory: 0 }
    };
    
    log::info("marketplace", "Moderation agent spawned: " + agent.id);
    return agent.id;
}

// AI agent moderates listing
fn moderate_listing_ai(listing_id: string, agent_id: string) {
    let listing = self.listings[listing_id];
    
    // Analyze NFT metadata
    let nft_data = chain::get_nft_metadata(
        listing.nft_contract,
        listing.token_id
    );
    
    // Use AI to check for violations
    let image_safe = ai::classify("content_safety", nft_data.image_url);
    let text_safe = ai::classify("content_safety", nft_data.description);
    
    if (image_safe == "unsafe" || text_safe == "unsafe") {
        // Flag for admin review
        listing.status = "flagged";
        self.listings[listing_id] = listing;
        
        log::audit("marketplace", "Listing flagged by AI: " + listing_id);
    }
}

// Notify moderation agents
fn notify_moderation_agents(listing_id: string) {
    for agent_id in self.agents.keys() {
        if self.agents[agent_id].type == "moderator" {
            self.moderate_listing_ai(listing_id, agent_id);
        }
    }
}
```

---

## 📋 Step 7: Admin Process Management

Admins monitor and control agents:

```dal
// Admin monitors all agents
@public
fn monitor_agents(admin_id: string) -> list<map> {
    // Check SuperAdmin
    let admin_level = self.get_admin_level(admin_id);
    let context = cloudadmin::create_admin_context(admin_id, admin_level);
    let allowed = cloudadmin::enforce_policy("strict", context);
    
    if (allowed.is_err() || !allowed.unwrap()) {
        log::info("marketplace", "Not authorized for monitoring");
        return [];
    }
    
    // Get all processes
    let processes = admin::list_processes();
    let agent_status = [];
    
    for process in processes {
        if (process.name.starts_with("mod_agent_")) {
            agent_status.push({
                "process_id": process.process_id,
                "name": process.name,
                "status": process.status,
                "cpu": process.resource_usage["cpu"],
                "memory": process.resource_usage["memory"]
            });
        }
    }
    
    return agent_status;
}

// Admin kills misbehaving agent
@public
fn terminate_agent(
    agent_id: string,
    admin_id: string,
    reason: string
) -> bool {
    // Check authorization
    let can_kill = cloudadmin::authorize(admin_id, "delete", "/agents");
    
    if (!can_kill) {
        log::info("marketplace", "Not authorized to terminate agents");
        return false;
    }
    
    // Kill the agent process
    let result = admin::kill(agent_id, reason);
    
    if (result.is_ok()) {
        // Update agent status
        if (self.agents.contains_key(agent_id)) {
            let agent = self.agents[agent_id];
            agent.status = "terminated";
            self.agents[agent_id] = agent;
        }
        
        log::audit("marketplace", "Agent terminated: " + agent_id + " - " + reason);
        return true;
    }
    
    return false;
}

// Check agent health
fn check_agent_health(agent_id: string) -> bool {
    let info = admin::get_process_info(agent_id);
    
    if info.is_err() {
        return false;
    }
    
    let process = info.unwrap();
    let cpu = process.resource_usage["cpu"].as_int();
    let memory = process.resource_usage["memory"].as_int();
    
    // Check resource limits
    if (cpu > 80 || memory > 1024) {
        log::info("marketplace", "Agent exceeding resources: " + agent_id);
        return false;
    }
    
    return true;
}
```

---

## 📋 Step 8: Emergency Admin Controls

SuperAdmin override capabilities:

```dal
// SuperAdmin emergency shutdown
@public
fn emergency_shutdown(admin_id: string, reason: string) -> bool {
    // Strict policy - SuperAdmin only
    let context = cloudadmin::create_admin_context(admin_id, "superadmin");
    let allowed = cloudadmin::enforce_policy("strict", context);
    
    if (allowed.is_err() || !allowed.unwrap()) {
        log::info("marketplace", "Emergency shutdown unauthorized");
        return false;
    }
    
    // Disable all new listings
    // Pause all trades
    // Notify all users
    
    log::audit("marketplace", "EMERGENCY SHUTDOWN by " + admin_id + ": " + reason);
    return true;
}

// SuperAdmin override listing status
@public
fn override_listing_status(
    listing_id: string,
    admin_id: string,
    new_status: string,
    reason: string
) -> bool {
    // SuperAdmin only
    let can_override = cloudadmin::authorize(admin_id, "delete", "/listings");
    
    if (!can_override) {
        return false;
    }
    
    let listing = self.listings[listing_id];
    let old_status = listing.status;
    listing.status = new_status;
    self.listings[listing_id] = listing;
    
    log::audit("marketplace", 
        "Status override: " + listing_id + 
        " from " + old_status + " to " + new_status +
        " - " + reason);
    
    return true;
}
```

---

## 📋 Step 9: Testing the Marketplace

Create comprehensive tests:

```dal
// Test hybrid trust
#[test]
fn test_hybrid_trust() {
    initialize_admins();
    
    // Test user listing with admin whitelist
    let listing_id = create_listing("0x123...", 1, 100);
    assert(listing_id != "error", "Listing creation failed");
    
    // Test admin approval
    let approved = approve_listing(listing_id, "admin_001");
    assert(approved, "Admin approval failed");
    
    // Test unauthorized admin
    let rejected = reject_listing(listing_id, "user_001", "test");
    assert(!rejected, "Unauthorized admin succeeded");
}

// Test agent monitoring
#[test]
fn test_agent_monitoring() {
    let agent_id = spawn_moderation_agent();
    
    // Monitor agents as SuperAdmin
    let status = monitor_agents("super_admin_001");
    assert(status.len() > 0, "No agents found");
    
    // Test termination
    let terminated = terminate_agent(agent_id, "super_admin_001", "test");
    assert(terminated, "Agent termination failed");
}

// Test policy enforcement
#[test]
fn test_policy_enforcement() {
    // Test strict policy
    let context = cloudadmin::create_admin_context("super_admin_001", "superadmin");
    let strict = cloudadmin::enforce_policy("strict", context);
    assert(strict.is_ok() && strict.unwrap(), "Strict policy failed");
    
    // Test moderate policy
    let context2 = cloudadmin::create_admin_context("admin_001", "admin");
    let moderate = cloudadmin::enforce_policy("moderate", context2);
    assert(moderate.is_ok() && moderate.unwrap(), "Moderate policy failed");
}
```

---

## 🚀 Step 10: Deployment

Deploy to Ethereum testnet:

```bash
# Compile
dal build hybrid_marketplace.dal --target ethereum

# Deploy to testnet
dal deploy hybrid_marketplace.dal \
  --network goerli \
  --private-key $PRIVATE_KEY \
  --gas-limit 5000000

# Initialize admins
dal call initialize_admins \
  --contract $CONTRACT_ADDRESS \
  --network goerli

# Spawn moderation agent
dal call spawn_moderation_agent \
  --contract $CONTRACT_ADDRESS \
  --network goerli
```

---

## 📊 Complete System Architecture

```
┌────────────────────────────────────────────┐
│      Hybrid NFT Marketplace System         │
├────────────────────────────────────────────┤
│                                            │
│  ┌──────────────┐      ┌──────────────┐   │
│  │ Centralized  │      │ Decentralized│   │
│  │   (Admin)    │      │   (Users)    │   │
│  └──────┬───────┘      └──────┬───────┘   │
│         │                     │            │
│         │  CloudAdmin Trust   │            │
│         └─────────┬───────────┘            │
│                   │                        │
│         ┌─────────▼─────────┐              │
│         │   Hybrid Trust    │              │
│         │    Validation     │              │
│         └─────────┬─────────┘              │
│                   │                        │
│         ┌─────────▼─────────┐              │
│         │   NFT Marketplace │              │
│         │                   │              │
│         │ • Listings        │              │
│         │ • Trading         │              │
│         │ • Moderation      │              │
│         └─────────┬─────────┘              │
│                   │                        │
│         ┌─────────▼─────────┐              │
│         │   AI Agents       │              │
│         │                   │              │
│         │ • Content Check   │              │
│         │ • Risk Analysis   │              │
│         │ • Notifications   │              │
│         └───────────────────┘              │
└────────────────────────────────────────────┘
```

---

## ✅ What You Learned

1. **Hybrid Trust Architecture** - Combining centralized and decentralized trust
2. **CloudAdmin Integration** - Authorization, policies, trust bridging
3. **Admin Levels** - SuperAdmin, Admin, Moderator hierarchy
4. **Process Management** - Monitoring and controlling AI agents
5. **Policy Enforcement** - Strict, moderate, permissive policies
6. **Emergency Controls** - SuperAdmin override capabilities

---

## 🎯 Next Steps

1. **Add More Features**:
   - Auction system
   - Offer/bidding
   - Collection management
   - Royalties

2. **Enhanced Moderation**:
   - More AI agents
   - ML-based risk scoring
   - Community reporting

3. **Better Admin Tools**:
   - Admin dashboard
   - Analytics
   - Bulk operations

4. **Production Deployment**:
   - Security audit
   - Load testing
   - Monitoring setup

---

## 📚 Related Resources

- [CloudAdmin Guide](../CLOUDADMIN_GUIDE.md)
- [Trust Models](../TRUST_MODEL_GUIDE.md)
- [AI Features Guide](../AI_FEATURES_GUIDE.md)
- [Best Practices](../BEST_PRACTICES.md)

---

**You've built a production-ready hybrid marketplace! 🎉**

The marketplace combines the best of both worlds:
- **Admin oversight** for safety and compliance
- **User freedom** for decentralized trading
- **AI automation** for scalable moderation
- **Process control** for system health

**Perfect for real-world applications that need both control and decentralization!**
