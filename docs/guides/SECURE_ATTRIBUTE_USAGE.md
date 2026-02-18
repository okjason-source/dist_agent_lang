# Using `@secure` in DAL: Development Guide

## Overview

The `@secure` attribute provides **automatic runtime enforcement** of authentication, reentrancy protection, and audit logging for DAL services. This guide explains how to use it effectively based on DAL's design philosophy and your development needs.

---

## Core Principles

### 0. **`@secure` vs "Private" - Important Distinction**

**`@secure` is NOT the same as "private":**

- **`@secure`** = **Authentication/Authorization + Reentrancy Protection**
  - Method is **callable** if user is authenticated
  - Requires `current_caller` to be set
  - Enforces authentication + reentrancy protection + audit logging
  
- **`@public`** = **No Authentication Required** (anyone can call it)
  - Method is **callable** without authentication
  - No auth checks performed
  
- **"Private"** (if it existed) = **Visibility** (not callable from outside at all)
  - Method would be **not callable** externally
  - Internal-only access

**In DAL:**
- `@secure` = "Requires authentication to call" (still callable, just needs auth)
- `@public` = "No authentication needed" (callable by anyone)
- DAL doesn't have a traditional "private" visibility modifier for services/methods

**Think of it this way:**
```dal
@secure    // = "Callable if authenticated" (like a protected API endpoint)
@public    // = "Callable by anyone" (like a public website)
// No @private in DAL - methods are always callable if you have access to the service
```

### 1. **Declarative Security by Default**

DAL emphasizes **declarative security** through attributes rather than manual checks. Use `@secure` to automatically enforce security requirements:

```dal
// ✅ GOOD: Declarative security
@secure
@trust("hybrid")
@chain("ethereum")
service UserManagementService {
    fn update_profile(user_id: string, data: map<string, any>) {
        // Authentication automatically enforced
        // Reentrancy protection automatically enabled
        // Audit logging automatically enabled
    }
}

// ❌ AVOID: Manual security checks (redundant with @secure)
@secure
service RedundantService {
    fn update_profile(user_id: string, data: map<string, any>) {
        // Don't manually check auth - @secure handles it
        if (!auth::is_authenticated()) {  // Redundant!
            return;
        }
    }
}
```

### 2. **Mutual Exclusivity: `@secure` vs `@public`**

These attributes are **mutually exclusive** - choose based on your access model:

```dal
// ✅ Public API - no authentication required
@public
@trust("hybrid")
@chain("ethereum")
service PublicAPIService {
    fn get_public_data() -> map<string, any> {
        // Anyone can call this
        return {"status": "ok"};
    }
}

// ✅ Secure API - authentication required
@secure
@trust("hybrid")
@chain("ethereum")
service SecureAPIService {
    fn get_user_data(user_id: string) -> map<string, any> {
        // Only authenticated users can call this
        // Audit log automatically created
        return {"user_id": user_id};
    }
}

// ❌ INVALID: Cannot combine both
@secure
@public  // Parser error: mutually exclusive
service InvalidService {
}
```

### 3. **Function-Level `@secure` (Granular Control)**

You can apply `@secure` at the function level for fine-grained security control. Function-level attributes **override** service-level attributes:

**Precedence Rules:**
1. **Function-level `@public`** overrides service-level `@secure` (makes function public)
2. **Function-level `@secure`** overrides service-level (makes function secure even if service is public)
3. **If function has no attributes**, it **inherits** from service-level (`@secure` or `@public`)
4. If neither function nor service has `@secure`, no protection is applied

**Examples:**

```dal
// ✅ Mixed security: Some methods secure, some public
service MixedService {
    fn public_read() -> map<string, any> {
        // ✅ Public access (no @secure)
        return {"data": "public"};
    }
    
    @secure
    fn admin_write(data: map<string, any>) {
        // ✅ Requires authentication (function-level @secure)
        // Reentrancy protection enabled
        // Audit logging enabled
    }
}

// ✅ Service-level secure with function-level override
@secure  // All methods secure by default (inherited by functions without attributes)
service SecureService {
    fn secure_method() {
        // ✅ Protected (inherits from service @secure - no function attribute needed)
    }
    
    fn another_secure_method() {
        // ✅ Also protected (inherits from service @secure)
    }
    
    @public
    fn public_method() {
        // ✅ Public (function-level @public overrides service @secure)
    }
    
    @secure
    fn explicitly_secure() {
        // ✅ Protected (explicit function-level, same as inheriting from service)
        // Note: Redundant but allowed for clarity
    }
}

// ❌ INVALID: Function cannot have both @secure and @public
service InvalidService {
    @secure
    @public  // Parser error: mutually exclusive
    fn invalid_method() {
    }
}
```

**When to Use Function-Level `@secure`:**
- ✅ You need **mixed security** in a single service (some public, some secure methods)
- ✅ You want to **override** service-level security for specific methods
- ✅ You prefer **granular control** over creating separate services

**When to Use Service-Level `@secure`:**
- ✅ **All methods** in the service require authentication
- ✅ You want **simpler code** (one attribute instead of many)
- ✅ The service has a **single security model**

---

## When to Use `@secure`

### ✅ **Use `@secure` for:**

1. **User Data Operations**
   ```dal
   @secure
   @trust("hybrid")
   @chain("ethereum")
   service UserService {
       fn update_email(user_id: string, email: string) {
           // Sensitive user data changes
       }
       
       fn delete_account(user_id: string) {
           // Destructive operations
       }
   }
   ```

2. **Financial Transactions**
   ```dal
   @secure
   @trust("decentralized")
   @chain("ethereum", "polygon")
   service PaymentService {
       fn transfer_funds(from: string, to: string, amount: int) {
           // Financial operations require authentication
       }
   }
   ```

3. **Admin Operations**
   ```dal
   @secure
   @trust("centralized")
   @chain("ethereum")
   service AdminService {
       fn configure_system(config: map<string, any>) {
           // Administrative changes
       }
   }
   ```

4. **Data Modification**
   ```dal
   @secure
   @trust("hybrid")
   @chain("ethereum")
   service DatabaseService {
       fn create_record(data: map<string, any>) {
           // Write operations
       }
       
       fn update_record(id: string, data: map<string, any>) {
           // Updates require authentication
       }
   }
   ```

### ❌ **Don't Use `@secure` for:**

1. **Public Read Operations** (use `@public` instead)
   ```dal
   @public  // Not @secure
   @trust("hybrid")
   @chain("ethereum")
   service PublicDataService {
       fn get_product_catalog() -> list<any> {
           // Public read access
       }
   }
   ```

2. **Internal/Private Services** (no attribute needed if not exposed)
   ```dal
   // No @secure needed for internal-only services
   @trust("hybrid")
   @chain("ethereum")
   service InternalService {
       fn internal_helper() {
           // Not exposed externally
       }
   }
   ```

---

## Combining `@secure` with Other Attributes

### Standard Secure Service Pattern

```dal
@secure                    // Authentication + audit logging
@trust("hybrid")          // Trust model
@chain("ethereum")        // Required with @trust
@compile_target("webassembly")  // Deployment target
@interface("typescript")  // Client interface generation
service SecureWebService {
    // All methods inherit @secure
}
```

### DeFi/Blockchain Services

```dal
@secure
@trust("decentralized")   // Fully decentralized
@chain("ethereum", "polygon", "arbitrum")  // Multi-chain
@compile_target("blockchain")
service DeFiService {
    fn swap_tokens(from: string, to: string, amount: int) {
        // Secure, multi-chain token swaps
    }
}
```

### Hybrid Trust Services

```dal
@secure
@trust("hybrid")          // Centralized + decentralized
@chain("ethereum")
@ai                       // AI capabilities
service HybridAIService {
    fn analyze_sensitive_data(data: string) -> map<string, any> {
        // Secure AI operations with hybrid trust
    }
}
```

---

## Trust Model Considerations

### Centralized Services (`@trust("centralized")`)

**Best for:** Traditional web APIs, admin panels, enterprise systems

```dal
@secure
@trust("centralized")
@chain("ethereum")
service EnterpriseService {
    // Centralized authentication
    // Audit logs stored centrally
}
```

**Characteristics:**
- Single source of truth for authentication
- Centralized audit logging
- Traditional session management

### Hybrid Services (`@trust("hybrid")`)

**Best for:** Most DAL applications - combines benefits of both models

```dal
@secure
@trust("hybrid")
@chain("ethereum")
service HybridService {
    // Can use both centralized and decentralized auth
    // Flexible audit logging
}
```

**Characteristics:**
- Supports both centralized and blockchain-based auth
- Flexible trust model
- Best of both worlds

### Decentralized Services (`@trust("decentralized")`)

**Best for:** DeFi, DAOs, fully decentralized applications

```dal
@secure
@trust("decentralized")
@chain("ethereum", "polygon")
service DeFiService {
    // Blockchain-based authentication
    // On-chain audit logs
}
```

**Characteristics:**
- Wallet-based authentication (msg.sender)
- On-chain audit trail
- No central authority

---

## Development Workflow

### 1. **Start with `@public` for Development**

During development, use `@public` to avoid authentication overhead:

```dal
@public  // Development mode
@trust("hybrid")
@chain("ethereum")
service DevelopmentService {
    fn test_endpoint() {
        // No auth needed during development
    }
}
```

### 2. **Switch to `@secure` Before Production**

Before deploying, change to `@secure`:

```dal
@secure  // Production mode
@trust("hybrid")
@chain("ethereum")
service ProductionService {
    fn production_endpoint() {
        // Auth enforced in production
    }
}
```

### 3. **Set Authentication Context**

When calling `@secure` services, ensure `current_caller` is set:

```dal
// In your runtime/client code:
runtime.set_current_caller("0x1234...");  // Set authenticated user
let result = runtime.call_service_method("SecureService", "updateProfile", args);
```

### 4. **Monitor Audit Logs**

All `@secure` service access is automatically logged:

```dal
// Audit logs are automatically created:
// - secure_service_access: Successful authenticated access
// - secure_service_access_denied: Unauthorized access attempts
```

**Persistent File Logging**: Audit logs are automatically written to files when `LOG_SINK=file` or `LOG_SINK=both` is set. See [Audit Logging Guide](./AUDIT_LOGGING.md) for details.

```bash
# Enable persistent audit logging
export LOG_SINK=both
export LOG_DIR=./logs
dal run your_service.dal

# View audit logs
cat logs/audit.log | jq '.'
```

---

## Common Patterns

### Pattern 1: Secure CRUD Service

```dal
@secure
@trust("hybrid")
@chain("ethereum")
service SecureCRUDService {
    data_store: map<string, any>;
    
    fn create(key: string, value: any) {
        // Authentication automatically enforced
        self.data_store[key] = value;
    }
    
    fn read(key: string) -> any {
        // Authentication automatically enforced
        return self.data_store[key];
    }
    
    fn update(key: string, value: any) {
        // Authentication automatically enforced
        self.data_store[key] = value;
    }
    
    fn delete(key: string) {
        // Authentication automatically enforced
        self.data_store.remove(key);
    }
}
```

### Pattern 2: Mixed Public/Secure Service

```dal
// Use separate services for different access levels
@public
@trust("hybrid")
@chain("ethereum")
service PublicService {
    fn get_public_info() -> map<string, any> {
        return {"version": "1.0"};
    }
}

@secure
@trust("hybrid")
@chain("ethereum")
service SecureService {
    fn get_user_info(user_id: string) -> map<string, any> {
        return {"user_id": user_id, "private": true};
    }
}
```

### Pattern 3: Secure Service with Transaction Support

```dal
@secure
@trust("hybrid")
@chain("ethereum")
service SecureTransactionService {
    @txn  // Transaction support
    fn transfer_assets(from: string, to: string, amount: int) {
        // Secure + transactional
        // Both auth and transaction safety enforced
    }
}
```

---

## Best Practices

### ✅ **DO:**

1. **Use `@secure` by default for write operations**
   ```dal
   @secure  // Default for modifications
   service WriteService {
       fn modify_data() { }
   }
   ```

2. **Combine with appropriate trust model**
   ```dal
   @secure
   @trust("hybrid")  // Match your trust requirements
   @chain("ethereum")
   service SecureService { }
   ```

3. **Always specify `@chain` when using `@trust`**
   ```dal
   @secure
   @trust("hybrid")
   @chain("ethereum")  // Required!
   service SecureService { }
   ```

4. **Use `@public` explicitly for public endpoints**
   ```dal
   @public  // Explicitly mark public APIs
   service PublicService { }
   ```

### ❌ **DON'T:**

1. **Don't manually check authentication in `@secure` services**
   ```dal
   @secure
   service Service {
       fn method() {
           // ❌ Redundant - @secure already enforces this
           if (!auth::is_authenticated()) {
               return;
           }
       }
   }
   ```

2. **Don't use `@secure` and `@public` together**
   ```dal
   @secure
   @public  // ❌ Parser error - mutually exclusive
   service InvalidService { }
   ```

3. **Don't forget `@chain` with `@trust`**
   ```dal
   @secure
   @trust("hybrid")
   // ❌ Missing @chain - parser error
   service InvalidService { }
   ```

---

## Migration Guide

### Migrating from Manual Auth Checks

**Before (Manual):**
```dal
service OldService {
    fn update_data(data: any) {
        if (!auth::is_authenticated()) {
            return;
        }
        log::audit("update_data", {"data": data}, Some("service"));
        // ... update logic
    }
}
```

**After (Declarative):**
```dal
@secure  // Automatic auth + audit
@trust("hybrid")
@chain("ethereum")
service NewService {
    fn update_data(data: any) {
        // Auth automatically enforced
        // Audit automatically logged
        // ... update logic
    }
}
```

---

## Troubleshooting

### Issue: "Access Denied" errors

**Cause:** `current_caller` not set or set to default/null address

**Solution:**
```dal
// Ensure caller is set before calling @secure service
runtime.set_current_caller("0x1234...");  // Valid address
// Not: "0x0000000000000000000000000000000000000000"
```

### Issue: Missing audit logs

**Check:**
- Verify `@secure` attribute is present
- Check log level configuration (`LOG_LEVEL=audit`)
- Ensure log sink is enabled (`LOG_SINK=console`)

### Issue: Parser errors with `@secure`

**Common causes:**
- `@secure` and `@public` used together (mutually exclusive)
- Missing `@chain` when using `@trust`
- Invalid trust model value

---

## Summary

- **Use `@secure`** for services requiring authentication
- **Use `@public`** for public APIs (mutually exclusive)
- **Always combine** with `@trust` and `@chain`
- **Let DAL handle** authentication and audit logging automatically
- **Set `current_caller`** when calling `@secure` services
- **Monitor audit logs** for security insights

The `@secure` attribute provides **declarative, automatic security enforcement** that aligns with DAL's philosophy of making security simple and consistent across your distributed applications.
