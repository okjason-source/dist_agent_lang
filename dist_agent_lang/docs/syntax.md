# dist_agent_lang Syntax Reference

Complete syntax reference for the dist_agent_lang programming language.

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
- [Comments](#comments)

---

## Basic Syntax

### Program Structure

A dist_agent_lang program consists of service declarations:

```rust
@trust("hybrid")
service MyService {
    // Service fields and methods
}
```

### Statements

Statements end with semicolons (optional in some contexts):

```rust
let x = 10;
print("Hello");
return result;
```

---

## Data Types

### Primitive Types

- `int` - Integer numbers: `42`, `-10`, `1000`
- `float` - Floating point numbers: `3.14`, `-0.5`, `2.0`
- `string` - Text strings: `"Hello"`, `"World"`
- `bool` - Boolean values: `true`, `false`
- `null` - Null value: `null`

### Complex Types

- `vector<T>` - Arrays/Lists: `[1, 2, 3]`, `["a", "b"]`
- `map<K, V>` - Maps/Dictionaries: `{"key": "value"}`
- `any` - Any type (dynamic)

**Examples:**
```rust
let numbers: vector<int> = [1, 2, 3];
let user: map<string, any> = {"name": "Alice", "age": 30};
```

---

## Variables

### Variable Declaration

```rust
let name = "Alice";
let age: int = 30;
let is_active: bool = true;
```

### Variable Assignment

```rust
let x = 10;
x = 20;  // Reassignment
```

### Service Fields

```rust
service MyService {
    count: int = 0;
    name: string = "default";
    
    fn increment() {
        self.count = self.count + 1;
    }
}
```

---

## Functions

### Function Declaration

```rust
fn function_name(param1: type, param2: type) -> return_type {
    // Function body
    return value;
}
```

### Function Examples

```rust
// Simple function
fn greet(name: string) -> string {
    return "Hello, " + name;
}

// Function with no return
fn print_message(msg: string) {
    print(msg);
}

// Function with no parameters
fn get_current_time() -> int {
    return 1234567890;
}
```

### Async Functions

```rust
async fn fetch_data(url: string) -> string {
    // Async operations
    return await http::get(url);
}
```

---

## Services

### Service Declaration

```rust
@trust("hybrid")
@chain("ethereum")
service MyService {
    // Fields
    balance: int = 0;
    
    // Methods
    fn deposit(amount: int) {
        self.balance = self.balance + amount;
    }
    
    fn get_balance() -> int {
        return self.balance;
    }
}
```

### Service Fields

```rust
service Example {
    // Public field
    public_name: string = "public";
    
    // Private field
    private_secret: string = "secret";
    
    // Field with initial value
    counter: int = 0;
}
```

---

## Agents

### Spawn Statement

```rust
spawn agent_name:ai {
    role: "assistant",
    capabilities: ["analysis", "reasoning"]
} {
    // Agent body
    log::info("agent", "Agent started");
}
```

### Agent Declaration

```rust
agent MyAgent:ai {
    role: "worker",
    capabilities: ["processing"]
} {
    fn process() {
        // Agent logic
    }
}
```

### Message Passing

```rust
msg agent_name {
    type: "task",
    data: {"task": "process"}
};

event "task_completed" {
    result: "success"
};
```

---

## Control Flow

### If Statements

```rust
if condition {
    // Then block
} else {
    // Else block
}

// Example
if x > 10 {
    print("Large");
} else {
    print("Small");
}
```

### While Loops

```rust
while condition {
    // Loop body
}

// Example
let i = 0;
while i < 10 {
    print(i);
    i = i + 1;
}
```

### For Loops

```rust
for item in collection {
    // Loop body
}

// Example
for num in [1, 2, 3] {
    print(num);
}
```

### Try-Catch-Finally

```rust
try {
    // Risky code
    risky_operation();
} catch (error) {
    // Error handling
    log::error("main", error);
} finally {
    // Cleanup
    cleanup();
}
```

### Return Statement

```rust
fn calculate() -> int {
    return 42;
}

// Early return
if condition {
    return;
}
```

---

## Operators

### Arithmetic Operators

- `+` - Addition
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division
- `%` - Modulo

```rust
let sum = 10 + 5;      // 15
let diff = 10 - 5;     // 5
let prod = 10 * 5;     // 50
let quot = 10 / 5;     // 2
let mod = 10 % 3;      // 1
```

### Comparison Operators

- `==` - Equal
- `!=` - Not equal
- `<` - Less than
- `<=` - Less than or equal
- `>` - Greater than
- `>=` - Greater than or equal

```rust
if x == 10 { }
if x != 10 { }
if x < 10 { }
if x <= 10 { }
if x > 10 { }
if x >= 10 { }
```

### Logical Operators

- `&&` - Logical AND
- `||` - Logical OR
- `!` - Logical NOT

```rust
if x > 0 && x < 10 { }
if x < 0 || x > 10 { }
if !is_empty { }
```

### Assignment Operators

- `=` - Assignment
- `+=` - Add and assign
- `-=` - Subtract and assign
- `*=` - Multiply and assign
- `/=` - Divide and assign

```rust
let x = 10;
x += 5;   // x = 15
x -= 3;   // x = 12
x *= 2;   // x = 24
x /= 4;   // x = 6
```

---

## Expressions

### Literals

```rust
42              // Integer
3.14            // Float
"Hello"         // String
true            // Boolean
false           // Boolean
null            // Null
```

### Arrays/Vectors

```rust
[1, 2, 3]                           // Integer array
["a", "b", "c"]                      // String array
[]                                   // Empty array
```

### Maps/Objects

```rust
{"key": "value"}                     // Simple map
{"name": "Alice", "age": 30}         // Map with multiple keys
{}                                    // Empty map
```

### Function Calls

```rust
function_name(arg1, arg2)
print("Hello")
chain::deploy(1, "Contract", {})
```

### Field Access

```rust
self.field                          // Access service field
object.property                      // Access object property
map["key"]                          // Access map value
```

### Binary Operations

```rust
a + b
a - b
a * b
a / b
a == b
a && b
```

### Unary Operations

```rust
!condition
-amount
await async_function()
```

---

## Comments

### Single-line Comments

```rust
// This is a comment
let x = 10; // Inline comment
```

### Multi-line Comments

```rust
/* This is a
   multi-line comment */
```

---

## Keywords

### Declaration Keywords

- `service` - Service declaration
- `fn` - Function declaration
- `let` - Variable declaration
- `agent` - Agent declaration
- `spawn` - Spawn agent
- `msg` - Send message
- `event` - Emit event

### Control Flow Keywords

- `if` - Conditional statement
- `else` - Else clause
- `while` - While loop
- `for` - For loop
- `return` - Return statement
- `try` - Try block
- `catch` - Catch block
- `finally` - Finally block
- `throw` - Throw exception
- `break` - Break loop
- `continue` - Continue loop

### Async Keywords

- `async` - Async function
- `await` - Await async operation

### Type Keywords

- `int`, `float`, `string`, `bool`, `null`
- `vector`, `map`, `any`

---

## Examples

### Complete Example

```rust
@trust("hybrid")
@chain("ethereum")
service TokenService {
    total_supply: int = 1000000;
    balances: map<string, int>;
    
    fn initialize() {
        let owner = auth::session().user_id;
        self.balances[owner] = self.total_supply;
    }
    
    fn transfer(to: string, amount: int) -> bool {
        let from = auth::session().user_id;
        
        if self.balances[from] < amount {
            return false;
        }
        
        self.balances[from] = self.balances[from] - amount;
        self.balances[to] = self.balances[to] + amount;
        
        return true;
    }
}
```

---

**See also:** [Attributes Reference](attributes.md) | [API Reference](api_reference.md)

