# dist_agent_lang Syntax Reference

DAL (dist_agent_lang) is a **networking-oriented language** that is built with **Rust**: services, agents, messages, and spawn are first-class; syntax uses **`fn`**, **`let`**, blocks, **`::`** namespacing, and type annotations in a Rust-friendly way. This document reflects the **parser and AST** as implemented and how DAL connects to **libraries and `src/`**: standard library calls use **`namespace::function(args)`**, implemented in **`src/stdlib/`** and dispatched in **`src/runtime/engine.rs`**. See [Testing Quick Reference](TESTING_QUICK_REFERENCE.md) and [Attributes Reference](attributes.md) for patterns and pitfalls.

## Table of Contents

- [Basic Syntax](#basic-syntax)
- [Data Types](#data-types)
- [Variables](#variables)
- [Functions](#functions)
- [Services](#services)
- [Agents](#agents)
- [Control Flow](#control-flow)
- [Operators](#operators)
- [Expressions](#expressions)
- [Libraries and Standard Namespaces](#libraries-and-standard-namespaces)
- [Comments](#comments)
- [Keywords](#keywords)

---

## Basic Syntax

### Program Structure

A DAL program is a sequence of **top-level statements**; there is no single “main” entry point. The language is built around **networking** and **Rust-like** structure:

- Service declarations (`service Name { ... }`)
- Function declarations (`fn name(...) { ... }`)
- Variable declarations (`let x = ...;`)
- Agent declarations (`agent name:type { ... } { ... }`)
- Spawn statements (`spawn name [: type] [{ config }] { body }`)
- Message and event statements (`msg` / `event`)
- Control flow (`if`, `while`, `for`, `try`)
- Expression statements (e.g. calls, assignments)

### Statements and Semicolons

- **`let`** and **`return`** statements **must** end with a semicolon.
- Expression statements (including assignments and calls) are terminated by semicolons when used as statements.
- Block bodies `{ ... }` do not use a semicolon after the closing `}`.

```dal
let x = 10;
print("Hello");
return result;
```

---

## Data Types

### Primitive Types

- **`int`** – integers: `42`, `-10`, `1000`
- **`float`** – floating point: `3.14`, `-0.5`, `2.0`
- **`string`** – text: `"Hello"`, `"World"`
- **`bool`** – booleans: `true`, `false`
- **`null`** – null value: `null`

### Collection and Generic Types

Type annotations use identifiers or type keywords with optional generic parameters:

- **`vector<T>`** or **`list<T>`** – lists (e.g. `list<int>`, `vector<string>`). The parser recognizes the **`list`** keyword; **`vector`** is accepted as an identifier type name.
- **`map<K, V>`** – maps (e.g. `map<string, int>`). **`map`** is a keyword.
- **`any`** – dynamic type (identifier).

**Examples:**

```dal
let numbers: list<int> = [1, 2, 3];
let items: vector<string> = ["a", "b"];
let user: map<string, any> = { name: "Alice", age: 30 };
```

---

## Variables

### Variable Declaration

```dal
let name = "Alice";
let age: int = 30;
let is_active: bool = true;
```

The parser allows an optional **`mut`** after `let` (it is consumed but not reflected in the AST).

### Variable Assignment

Only **simple assignment** is supported: **`=`**. Compound assignment operators (`+=`, `-=`, `*=`, `/=`, etc.) are **not** parsed as assignments.

```dal
let x = 10;
x = 20;
```

### Service Fields

Fields are declared with **name**, **type**, optional **initial value**, and a **semicolon**. Visibility modifiers (e.g. `public` / `private`) are **not** parsed; all fields are treated as public.

```dal
service MyService {
    count: int = 0;
    name: string = "default";

    fn increment() {
        self.count = self.count + 1;
    }
}
```

### Service Instantiation

**Syntax 1 – type name as namespace:**

```dal
let instance = MyService::new();
instance.increment();
```

**Syntax 2 – `service` namespace:**

```dal
let instance = service::new("MyService");
instance.increment();
```

Both create a new instance of the service.

### Service Method Calls and Field Access

Within service methods, use **`self.field`** to read and assign fields. Method calls use **`instance.method(args)`** or **`namespace::function(args)`**.

```dal
service TokenContract {
    balances: map<string, int> = {};

    fn initialize(owner: string) {
        self.balances[owner] = 1000;
    }

    fn transfer(to: string, amount: int) {
        self.balances[to] = self.balances[to] + amount;
    }
}

let token = TokenContract::new();
token.initialize("0x123...");
token.transfer("0x456...", 100);
```

### Indexed Access and Assignment

Map and array indexing use **`expr[index]`**. Assignment to an index is **`expr[index] = value`** (parsed as a special assignment form).

```dal
self.balances[address] = amount;
return self.balances[address];
self.items[0] = item;
```

---

## Functions

### Function Declaration

```dal
fn function_name(param1: type, param2: type) -> return_type {
    return value;
}
```

- Parameters: **`name`** or **`name: type`**.
- Return type is optional: **`-> return_type`** after the closing `)`.
- Body is a block **`{ ... }**.

### Function Examples

```dal
fn greet(name: string) -> string {
    return "Hello, " + name;
}

fn print_message(msg: string) {
    print(msg);
}

fn get_current_time() -> int {
    return 1234567890;
}
```

### Async Functions

**`async fn`** is supported at the top level or inside a service. The parser accepts **`async fn name(...) [ -> type ] { body }`**. Attributes (e.g. **`@async`**) may appear before **`fn`** or **`async fn`** when allowed by the parser.

```dal
async fn fetch_data(url: string) -> string {
    return await http::get(url);
}
```

---

## Services

### Service Declaration

Services can be preceded by **attributes** (e.g. **`@trust`**, **`@chain`**, **`@secure`**, **`@compile_target`**). See [Attributes Reference](attributes.md).

```dal
@trust("hybrid")
@chain("ethereum")
service MyService {
    balance: int = 0;

    fn deposit(amount: int) {
        self.balance = self.balance + amount;
    }

    fn get_balance() -> int {
        return self.balance;
    }
}
```

### Service Members

- **Fields:** **`name: type [ = value ];`**
- **Methods:** **`fn name(params) [ -> type ] { body }`**
- **Events:** Optional **`event`** declarations (see parser for exact form).

Field visibility is not parsed; all fields are effectively public.

---

## Agents

Agents can be created in two ways: **language syntax** (`spawn` / `agent`) and **stdlib API** (`ai::create_agent`).

### Create agent (stdlib API)

The **`ai::create_agent(config)`** function creates an agent instance at runtime. **`config`** is a literal (e.g. role, capabilities). This is the usual pattern in examples and scripts.

```dal
let agent_config = {
    role: "assistant",
    capabilities: ["analysis", "reasoning"]
};
let agent_instance = ai::create_agent(agent_config);
```

For coordinators: **`ai::create_agent_coordinator()`**.

### Spawn Statement

**`spawn`** as a **statement** has one of these forms:

1. **`spawn agent_name { body }`** – body block is **required**.
2. **`spawn agent_name : type { body }`** – optional type (e.g. `ai`, `worker`).
3. **`spawn agent_name : type { config } { body }`** – config is a struct literal; body required.

Config follows literal rules: keys are identifiers or string literals, colon, then expression.

```dal
spawn agent_name:ai {
    role: "assistant",
    capabilities: ["analysis", "reasoning"]
} {
    log::info("agent", "Agent started");
}
```

### Spawn Expression

**`spawn`** can also be used as an **expression**: **`spawn expr`**. For example, spawning the result of a call:

```dal
let handle = spawn worker_process(i);
```

The parser treats **`spawn`** in expression context as a unary operator: **`spawn`** followed by an expression (e.g. a function call).

### Agent Declaration

**`agent name : type { config } [ with capabilities ] { body }`** defines an agent **type** (name, type, config, and body). It does not create a running instance.

- **`type`** is required (e.g. **`ai`**, **`system`**, **`worker`**, or a custom identifier).
- **`config`** is a literal **`{ key: value, ... }`**.
- Optional **`with`** clause for capabilities (parser-dependent).
- **`body`** is a block **`{ ... }`**.

```dal
agent MyAgent:ai {
    role: "worker",
    capabilities: ["processing"]
} {
    fn process() {
        // Agent logic
    }
}
```

### Message Statement

**`msg recipient { data } ;`**

- **`recipient`** is an identifier.
- **`data`** is a single **`{ ... }`** block. Keys in the data block are **identifiers**; each key is followed by **`:`** and an expression. Comma-separated.

```dal
msg agent_name {
    type: "task",
    data: { key: "value" }
};
```

### Event Statement

**`event event_name { data } ;`**

- **`event_name`** is an **identifier** (not a string literal in the current parser).
- **`data`** is the same as for **`msg`**: **`{ key: value, ... }`** with identifier keys and colons.

```dal
event task_completed {
    result: "success",
    count: 42
};
```

---

## Control Flow

### If Statements

**Parentheses around the condition are required.** Use **`if (condition) { ... }`**, not **`if condition`**.

```dal
if (x > 10) {
    print("Large");
} else if (x > 0) {
    print("Positive");
} else {
    print("Small");
}
```

- **`else if (condition) { block }`** is supported (parsed as **`else`** followed by an **`if`** statement in a block).
- **`else { block }`** is optional.

### For Loops

Only **for-in** is supported: **`for variable in iterable { body }`**. **`variable`** can be an identifier or keyword used as identifier; **`iterable`** is an expression.

```dal
for item in [1, 2, 3] {
    print(item);
}
```

### While Loops

**`while (condition) { body }`**. Parentheses around the condition are required.

```dal
while (count > 0) {
    count = count - 1;
    process(count);
}
```

### Try-Catch-Finally

```dal
try {
    risky_operation();
} catch (error_type error_var) {
    log::info("main", error_var);
} finally {
    cleanup();
}
```

- **`try`** is followed by a single block **`{ ... }`**.
- Zero or more **`catch`** blocks. **`catch`** can be **`catch { body }`** or **`catch (ErrorType var) { body }`** (error type and variable are optional in the grammar).
- Optional single **`finally { body }`** after all **`catch`** blocks.

### Break Statement

**`break [ expression ] ;`**

Exit the innermost loop (`while`, `for`, or `loop`). Optionally returns a value from the loop.

```dal
let count = 0;
while (count < 10) {
    count = count + 1;
    if (count == 5) {
        break;  // Exit loop
    }
}

// Break with value
let result = 0;
loop {
    result = result + 1;
    if (result == 42) {
        break result;  // Exit loop and return value
    }
}
```

### Continue Statement

**`continue;`**

Skip to the next iteration of the innermost loop (`while`, `for`, or `loop`).

```dal
let sum = 0;
for item in [1, 2, 3, 4, 5] {
    if (item % 2 == 0) {
        continue;  // Skip even numbers
    }
    sum = sum + item;
}
```

### Loop Statement

**`loop { body }`**

Infinite loop that can be exited with `break`. Includes timeout protection to prevent infinite execution.

```dal
let count = 0;
loop {
    count = count + 1;
    if (count == 10) {
        break;
    }
}
```

### Match Statement

**`match expression { case1 => body1, case2 => body2, default => body }`**

Pattern matching expression. Matches the expression against patterns and executes the first matching case.

**Patterns:**
- **Literal**: `42`, `"hello"`, `true`, `null` - matches exact value
- **Identifier**: `x` - matches anything and binds to variable `x`
- **Wildcard**: `_` - matches anything (no binding)
- **Range**: `start..end` - matches numeric values in range (inclusive)

```dal
// Literal patterns
let status = "success";
match status {
    "success" => 1,
    "error" => 0,
    default => -1
}

// Identifier pattern (binds value)
let value = 42;
match value {
    x => x * 2  // x is bound to 42, returns 84
}

// Range pattern
let score = 85;
match score {
    90..100 => "A",
    80..89 => "B",
    70..79 => "C",
    default => "F"
}

// Wildcard pattern
match value {
    _ => "matched"
}
```

**Notes:**
- Cases are evaluated in order; first match wins
- `default` case is optional; if no case matches and no default, returns `null`
- Pattern bindings (identifier patterns) are scoped to the case body
- Range patterns work with integer values only

### Return Statement

**`return [ expression ] ;`**

```dal
fn calculate() -> int {
    return 42;
}

if (condition) {
    return;
}
```

---

## Operators

### Arithmetic

- **`+`** **`-`** **`*`** **`/`** **`%`**

```dal
let sum = 10 + 5;
let diff = 10 - 5;
let prod = 10 * 5;
let quot = 10 / 5;
let rem = 10 % 3;
```

### Comparison

- **`==`** **`!=`** **`<`** **`<=`** **`>`** **`>=`**

```dal
if (x == 10) { }
if (x != 10) { }
if (x < 10) { }
if (x <= 10) { }
if (x > 10) { }
if (x >= 10) { }
```

### Logical

- **`&&`** **`||`** **`!`**

```dal
if (x > 0 && x < 10) { }
if (x < 0 || x > 10) { }
if (!is_empty) { }
```

### Assignment

- **`=`** – simple assignment. **Compound assignment** (**`+=`**, **`-=`**, **`*=`**, **`/=`**, etc.) is **not** parsed as assignment; use **`x = x + 1`** etc.

```dal
let x = 10;
x = 20;
```

---

## Expressions

### Literals

- Integers, floats, strings (double-quoted), **`true`** / **`false`**, **`null`**.

```dal
42
3.14
"Hello"
true
false
null
```

### Array Literals

**`[ expr1, expr2, ... ]`**

```dal
[1, 2, 3]
["a", "b", "c"]
[]
```

### Literals

**`{ key: value, ... }`**

DAL uses **key-value** pairs inside **`{ }`** for literals: agent/spawn config, message/event data, and any expression. The form is always **key : value** (colon between key and value).

**Is the colon required?** **Yes.** The parser requires a **colon** after each key; without it you get an error (except in one narrow compatibility case when the value starts with `this`).

| Context | Key allowed | Colon | Example |
|--------|--------------|-------|--------|
| **Literal** (expression) | Identifier or string literal | Required | `{ name: "Alice", age: 30 }` or `{ "name": "Alice" }` |
| **`msg` / `event` data** | Identifier only | Required | `msg x { type: "task", id: 1 };` |

- **Keys**: In **literals** (e.g. config, payloads), keys can be an **identifier** (`name`) or a **string literal** (`"name"`). In **`msg`** and **`event`** data blocks, keys must be **identifiers**.
- **Values**: Any expression after the colon.
- **Commas**: Between pairs, optional in literals; in `msg`/`event` data, a comma is required between entries.

```dal
{ key: "value" }
{ name: "Alice", age: 30 }
{}
```

### Function Calls

- **`name(args)`** – direct call.
- **`namespace::function(args)`** – namespace call into the standard library (e.g. **`chain::deploy(1, "Contract", {})`**, **`log::info("tag", "message")`**). Namespaces are implemented in **`src/stdlib/`** and dispatched in **`src/runtime/engine.rs`**; see [Libraries and Standard Namespaces](#libraries-and-standard-namespaces).
- **`expr.method(args)`** – method call (parsed as a call with a compound name).

```dal
print("Hello");
chain::deploy(1, "Contract", {});
instance.method(arg1, arg2);
```

### Inline Closures (Call-Argument Only)

Inside a **function argument list**, the parser accepts a **single-parameter closure** with block body:

**`( param => { body } )`**

- **Single parameter** (identifier).
- **Block body** only; **expression bodies** (e.g. **`r => r.success`**) are **not** supported. Use **`param => { return expr; }`**.

```dal
handler.process(items, r => { return r.success; });
```

Closures are only recognized as the last (or only) argument in a **`(...)`** list, immediately after a single identifier and **`=>`**.

### Field and Index Access

- **`expr.field`** – field access (literal or service).
- **`expr[index]`** – index access (maps/arrays).
- **`self.field`** – service field (identifier **`self`** plus field).

```dal
self.field
value.field
map["key"]
arr[i]
```

### Unary and Binary Operations

- Unary: **`!expr`**, **`-expr`**, **`await expr`**, **`spawn expr`**, **`throw expr`**.
- Binary: arithmetic, comparison, and logical operators as above.

```dal
!condition
-amount
await async_call()
spawn worker(i)
throw error_expr
```

---

## Libraries and Standard Namespaces

DAL code calls into the **standard library** using the **`namespace::function(args)`** syntax. The runtime resolves these calls in **`src/runtime/engine.rs`** (see `call_namespace_function`) and delegates to the corresponding module under **`src/stdlib/`**. Any **DAL-specific syntax** you use for libraries is this call form plus the argument shapes each namespace expects.

### Namespace → implementation mapping

| Namespace   | Implementation        | Typical DAL usage |
|------------|------------------------|-------------------|
| **`service`** | `src/stdlib/service.rs`  | `service::new("ServiceName")` – create instance by name |
| **`ai`**      | `src/stdlib/ai.rs`       | `ai::create_agent(config)`, `ai::create_agent_coordinator()` |
| **`agent`**   | `src/stdlib/agent.rs`    | `agent::create_agent_message(...)`, `agent::create_agent_task(...)` |
| **`log`**     | `src/stdlib/log.rs`      | `log::info("tag", message)`, `log::audit("tag", message)` |
| **`auth`**    | `src/stdlib/auth.rs`     | `auth::session(user_id, roles)` – returns a session value; use its fields (e.g. `user_id`) as in [API Reference](api_reference.md) |
| **`chain`**   | `src/stdlib/chain.rs`    | `chain::deploy(chain_id, contract_name, {})`, `chain::estimate_gas(...)` |
| **`oracle`**  | `src/stdlib/oracle.rs`   | `oracle::fetch(...)`, `oracle::create_query(...)` |
| **`crypto`**  | `src/stdlib/crypto.rs`   | `crypto::hash(...)`, `crypto::sign(...)`, `crypto::verify(...)` |
| **`database`** | `src/stdlib/database.rs` | DB helpers (see [API Reference](api_reference.md)) |
| **`web`**     | `src/stdlib/web.rs`      | Web/server helpers |
| **`sync`**, **`key`**, **`kyc`**, **`aml`**, **`admin`**, **`cloudadmin`**, **`config`**, **`trust`**, **`mobile`**, **`desktop`**, **`iot`** | `src/stdlib/<name>.rs` | Same pattern: `namespace::function(args)` |

### DAL-specific library syntax

- **Service instances**
  - **`ServiceName::new()`** – parser allows **`identifier::identifier`**; runtime treats a known service name as namespace and **`new`** as constructor. Implemented in `engine.rs` (`call_service_instance_method` / `call_service_function`).
  - **`service::new("ServiceName")`** – string-based constructor; same runtime path for creating an instance.
- **Logging**
  - **`log::info("tag", message)`** – two arguments (tag string, message). **`log::audit("tag", message)`** – same shape. Other `log::*` functions are only available if implemented in the engine.
- **Auth**
  - **`auth::session(user_id, roles)`** – `roles` is an array of strings. Returns a session value; use its fields (e.g. `user_id`) as documented in [API Reference](api_reference.md).
- **Chain**
  - **`chain::deploy(chain_id, contract_name, constructor_args)`** – `chain_id` int, `contract_name` string, third argument a literal. Chain access is subject to trust validation in the runtime.
- **AI / agents**
  - **`ai::create_agent(config)`** – `config` is a literal (e.g. `role`, `capabilities`). **`ai::create_agent_coordinator()`** – no arguments. Full list of `ai::*` and `agent::*` functions is in [API Reference](api_reference.md).

For full signatures and behavior, see **[API Reference](api_reference.md)**. For where each namespace is wired and executed, see **`src/runtime/engine.rs`** (e.g. `call_ai_function`, `call_log_function`, `call_auth_function`, `call_chain_function`) and the corresponding **`src/stdlib/*.rs`** modules.

---

## Comments

- **Single-line:** **`// ...`** to end of line.
- **Multi-line:** **`/* ... */`**.

```dal
// This is a comment
let x = 10; // Inline comment

/* Multi-line
   comment */
```

---

## Keywords

### Declaration and Structure

- **`service`** **`fn`** **`let`** **`agent`** **`spawn`** **`msg`** **`event`**

### Control Flow

- **`if`** **`else`** **`while`** **`for`** **`in`** **`return`** **`try`** **`catch`** **`finally`** **`throw`**
- **`break`** **`continue`** **`loop`** **`match`** **`default`** – implemented and functional

### Async

- **`async`** **`await`**

### Types and Modifiers

- **`list`** **`map`** – type keywords for generics.
- **`int`** **`float`** **`string`** **`bool`** **`null`** – used in type annotations (as identifiers or literals where applicable).
- **`mut`** – consumed after **`let`** but not stored in the AST.

### Attributes and Targets

- **`@trust`** **`@chain`** **`@secure`** **`@compile_target`** **`@interface`** etc. – see [Attributes Reference](attributes.md).

---

## Cross-Reference

- **[Libraries and Standard Namespaces](#libraries-and-standard-namespaces)** – DAL **`namespace::function(args)`** syntax, mapping to **`src/stdlib/`** and **`src/runtime/engine.rs`**.
- **[Testing Quick Reference](TESTING_QUICK_REFERENCE.md)** – test commands and patterns; required syntax (e.g. parentheses in **`if`**) and common mistakes are in this document.
- **[Attributes Reference](attributes.md)** – service and function attributes.
- **[API Reference](api_reference.md)** – full standard library APIs.
