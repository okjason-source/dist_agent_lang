# Compliance Architecture in dist_agent_lang

**Part of the planned update:** This architecture is part of the same rollout as the **cross-component CLI** (bond, pipe, invoke), **agent/assist**, and **sh/evolution** plans. The **invoke compliance-check** workflow wires compliance into the CLI and into agent-driven use; **ai-audit** can run alongside. See [CROSS_COMPONENT_CLI_PLAN.md](../development/CROSS_COMPONENT_CLI_PLAN.md) for how compliance-check fits and [AGENT_SHELL_EVOLUTION_PLAN.md](../development/AGENT_SHELL_EVOLUTION_PLAN.md) for agent context.

---

## 🎯 **Built-in Regulatory Compliance**

## 🏗️ **Architecture Overview**

### **1. Language-Level Abstractions**
```rust
// This is what "built-in" means - the language understands compliance
let kyc_result = kyc::verify_identity("securekyc", user_address, "enhanced", user_data);
let aml_result = aml::perform_check("chainalysis", user_address, "sanctions", user_data);
```

**What's Built-in:**
- ✅ **Syntax**: The language understands `kyc::` and `aml::` as native namespaces
- ✅ **Type Safety**: Compile-time checking of compliance function calls
- ✅ **Error Handling**: Language-level error handling for compliance failures
- ✅ **Integration**: Seamless integration with smart contract logic

**What's NOT Built-in:**
- ❌ **Actual API Calls**: These are simulated/proxied to external services
- ❌ **Client-Side Processing**: Sensitive compliance checks happen server-side
- ❌ **Data Storage**: Compliance data is not stored in the blockchain

## 🔄 **How It Actually Works**

### **1. Hybrid Architecture**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Side   │    │  dist_agent_lang │    │  Server Side    │
│                 │    │   Runtime        │    │                 │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • User Input    │───▶│ • Language       │───▶│ • KYC Providers │
│ • Wallet        │    │   Abstractions   │    │ • AML Services  │
│ • UI            │    │ • Function Calls │    │ • Compliance DB │
│ • Local State   │    │ • Error Handling│    │ • Audit Logs    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **2. Client-Side Components**
```rust
// What happens on the client side
@trust("hybrid")
service DeFiNFT_RWA {
    fn tokenize_asset_with_compliance(owner: string, asset_type: string, value: int) -> int {
        // 1. Client-side: Prepare user data
        let user_data = {
            "full_name": "John Doe",
            "date_of_birth": "1990-01-01",
            "address": "123 Main St, City, Country"
        };
        
        // 2. Client-side: Call compliance functions (language abstraction)
        let kyc_result = kyc::verify_identity("securekyc", owner, "enhanced", user_data);
        
        // 3. Client-side: Handle the response
        if kyc_result["status"] != "verified" {
            throw "KYC verification failed";
        }
        
        // 4. Client-side: Proceed with business logic
        return self.tokenize_asset(owner, asset_type, value);
    }
}
```

### **3. Server-Side Components**
```rust
// What actually happens server-side (simplified)
pub fn verify_identity(provider_id: String, user_address: String, level: String, user_data: HashMap<String, String>) -> HashMap<String, Value> {
    // 1. Server-side: Validate input
    if !KYC_PROVIDERS.contains_key(&provider_id) {
        return { "status": "failed", "error": "Provider not found" };
    }
    
    // 2. Server-side: Make actual API call to external provider
    let provider = KYC_PROVIDERS.get(&provider_id).unwrap();
    let api_response = make_http_request(provider.api_endpoint, provider.api_key, user_data);
    
    // 3. Server-side: Process response
    let verification_id = format!("kyc_{}_{}", user_address, chain::get_block_timestamp(1));
    
    // 4. Server-side: Return structured response
    return {
        "status": "verified",
        "verification_id": verification_id,
        "confidence": 0.95,
        "provider": provider_id,
        "timestamp": chain::get_block_timestamp(1)
    };
}
```

## 🔐 **Privacy and Security Considerations**

### **1. What Stays Client-Side**
- ✅ **User Interface**: Forms, validation, user experience
- ✅ **Wallet Integration**: Address management, transaction signing
- ✅ **Local State**: Temporary data during the verification process
- ✅ **Language Logic**: Business logic, flow control, error handling

### **2. What Goes Server-Side**
- 🔒 **Sensitive Data**: Personal information, documents, biometrics
- 🔒 **External APIs**: KYC provider calls, AML service requests
- 🔒 **Compliance Database**: Verification records, audit trails
- 🔒 **Risk Assessment**: Complex calculations, pattern analysis

### **3. What Gets Stored On-Chain**
- 📝 **Verification Status**: Boolean flags (verified/not verified)
- 📝 **Verification IDs**: Reference numbers for off-chain data
- 📝 **Timestamps**: When verification occurred
- 📝 **Audit Trails**: Immutable records of compliance actions

## 🎯 **Real-World Implementation**

### **1. Development Environment**
```rust
// During development, everything is simulated
let kyc_result = kyc::verify_identity("securekyc", user_address, "enhanced", user_data);
// Returns: { "status": "verified", "verification_id": "kyc_123...", "confidence": 0.95 }
```

### **2. Production Environment**
```rust
// In production, this calls real external services
let kyc_result = kyc::verify_identity("securekyc", user_address, "enhanced", user_data);
// Actually calls: https://api.securekyc.com/v1/verify
// Returns: Real verification result from external provider
```

### **3. Configuration**
```toml
# Cargo.toml
[features]
default = ["simulated"]
simulated = []  # Use simulated responses
production = []  # Use real external APIs
```

## 🔄 **Data Flow Example**

### **1. User Initiates Verification**
```
User clicks "Verify Identity" in dApp
    ↓
dist_agent_lang runtime calls kyc::verify_identity()
    ↓
Runtime makes HTTP request to external KYC provider
    ↓
KYC provider processes verification (server-side)
    ↓
Provider returns result to runtime
    ↓
Runtime returns structured response to smart contract
    ↓
Smart contract stores verification status on-chain
```

### **2. What Gets Stored**
```rust
// On-chain storage (minimal, privacy-preserving)
struct UserProfile {
    address: string,
    kyc_status: bool,           // true/false only
    kyc_verification_id: string, // reference to off-chain data
    kyc_timestamp: int,         // when verified
    aml_status: bool,           // true/false only
    aml_check_id: string,       // reference to off-chain data
    aml_timestamp: int,         // when checked
}

// Off-chain storage (comprehensive, private)
struct KYCVerification {
    verification_id: string,
    user_data: HashMap<String, String>,  // Personal information
    provider_response: HashMap<String, String>, // Full provider response
    audit_trail: Vec<AuditEvent>,       // Complete audit trail
    compliance_score: float,             // Detailed risk assessment
}
```

## 🛡️ **Privacy-Preserving Features**

### **1. Zero-Knowledge Verification**
```rust
// Future enhancement: Privacy-preserving verification
let zk_verification = kyc::verify_identity_zk(
    "securekyc",
    user_address,
    "enhanced",
    user_data,
    zk_proof  // Zero-knowledge proof of identity
);
```

### **2. Selective Disclosure**
```rust
// Only reveal necessary information
let minimal_verification = kyc::verify_identity_minimal(
    "securekyc",
    user_address,
    "basic",
    { "age_over_18": true, "residency": "US" }  // Minimal data
);
```

### **3. On-Chain Privacy**
```rust
// Store only verification status, not personal data
event KYCVerified {
    user_address: string,
    verification_id: string,  // Reference to off-chain data
    status: bool,            // Verified or not
    timestamp: int           // When verified
}
```

## 🎯 **Benefits of This Architecture**

### **1. Developer Experience**
- **Simple API**: `kyc::verify_identity()` instead of complex HTTP calls
- **Type Safety**: Compile-time checking of compliance operations
- **Error Handling**: Language-level error handling
- **Integration**: Seamless integration with smart contract logic

### **2. Privacy and Security**
- **Minimal On-Chain Data**: Only verification status stored on-chain
- **External Processing**: Sensitive data processed by specialized providers
- **Audit Trails**: Comprehensive off-chain audit trails
- **Compliance**: Meets regulatory requirements for data handling

### **3. Scalability**
- **Provider Agnostic**: Can switch between different KYC/AML providers
- **Modular Design**: Easy to add new compliance features
- **Performance**: External processing doesn't slow down blockchain
- **Cost Effective**: Pay only for actual verification services

## 🔮 **Future Enhancements**

### **1. Decentralized Identity**
```rust
// Self-sovereign identity integration
let did_verification = kyc::verify_did_identity(
    user_did,           // Decentralized identifier
    verifiable_credentials,  // W3C verifiable credentials
    proof_of_identity   // Cryptographic proof
);
```

### **2. Cross-Chain Compliance**
```rust
// Multi-chain compliance verification
let cross_chain_verification = kyc::verify_cross_chain(
    user_address,
    ["ethereum", "polygon", "bsc"],  // Multiple chains
    "enhanced"
);
```

### **3. AI-Powered Compliance**
```rust
// Machine learning-based risk assessment (fits ai:: / assist:: namespace)
let ai_risk_assessment = aml::assess_risk_ai(
    user_address,
    transaction_history,
    behavioral_patterns,
    ai_model_parameters
);
```
AI-powered compliance uses the same **ai::** / **assist::** stdlib as the rest of the stack; access can be gated via **key::** (e.g. resource `"aml"`, operation `"assess_risk_ai"`) for unified access control.

## 🤖 **Agent, assist, and CLI**

- **invoke compliance-check** — The cross-component CLI workflow that runs the compliance orchestration (kyc::, aml::, audit). A human or an **agent** can run it (e.g. `dal invoke compliance-check` or as an agent task).
- **ai-audit** — AI-assisted audit (e.g. of a contract) can run before or after compliance-check; both use the same **ai::** / **assist::** namespace where AI is involved.
- **key::** — Who can run compliance operations (or high-risk aml/kyc actions) can be governed by **key.rs** capability checks, consistent with the rest of the stack.

---

## 📋 **Summary**

**"Built-in regulatory compliance"** means:

✅ **Language-Level**: Compliance operations are native to the language syntax
✅ **Developer-Friendly**: Simple API calls instead of complex integrations
✅ **Type-Safe**: Compile-time checking of compliance operations
✅ **Integrated**: Seamless integration with smart contract logic

❌ **NOT Client-Side**: Sensitive compliance checks happen server-side
❌ **NOT Self-Contained**: External providers handle actual verification
❌ **NOT On-Chain Storage**: Personal data is not stored on the blockchain

The architecture provides the **best of both worlds**: developer-friendly language abstractions with enterprise-grade privacy and security! 🚀

---

## 📐 **DAL as Orchestrator**

DAL’s strength is **orchestration**: it coordinates external compliance services (KYC, AML, policy engines, audit sinks) and enforces flow and policy in code. It does **not** implement compliance engines itself — it calls them, branches on results, and produces a single auditable trail.

**Implementation status:** Incorporated into the **cross-component CLI** plan:

- **invoke compliance-check** — ✅ Implemented. Workflow orchestrates **kyc::**, **aml::**, and audit per this architecture. `dal invoke compliance-check <addr> [--chain_id N] [--aml]`. See [CROSS_COMPONENT_IMPLEMENTATION_PLAN.md](../development/implementation/CROSS_COMPONENT_IMPLEMENTATION_PLAN.md).
- **key::** — Implemented. When `DAL_COMPLIANCE_KEY_GATE=1` (or `true`), sensitive **kyc::** and **aml::** operations are gated via **key::check**: principal from `DAL_COMPLIANCE_PRINCIPAL` (default `caller`), resource `"kyc"` or `"aml"`, operation = function name. Gated KYC: `verify_identity`, `validate_document`, `revoke_verification`, `get_compliance_report`. Gated AML: `perform_check`, `get_risk_assessment`, `screen_transaction`, `check_sanctions_list`. Grant access with `key::create("kyc", ["verify_identity", ...])` and `key::grant(cap, principal)` (or equivalent for `"aml"`).
- **COMPLIANCE_IMPLEMENTATION_PLAN.md** — Deferred. Audit trail schema, provider wiring, SOC2/HIPAA/GDPR patterns.
