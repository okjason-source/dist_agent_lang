// Solidity Parser - Parses Solidity source code into AST

/// Solidity AST structures
#[derive(Debug, Clone)]
pub struct SolidityAST {
    pub contracts: Vec<Contract>,
    pub pragma: Option<String>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub kind: ContractKind,
    pub state_variables: Vec<StateVariable>,
    pub functions: Vec<Function>,
    pub events: Vec<Event>,
    pub modifiers: Vec<Modifier>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub inheritance: Vec<String>,
    /// Nested contracts/interfaces/libraries (e.g. `contract X { contract Y { } }`).
    pub nested_contracts: Vec<Contract>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractKind {
    Contract,
    Interface,
    Abstract,
    Library,
}

#[derive(Debug, Clone)]
pub struct StateVariable {
    pub name: String,
    pub var_type: String,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub returns: Vec<Parameter>,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub modifiers: Vec<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    External,
    Internal,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    View,
    Pure,
    Payable,
    NonPayable,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub anonymous: bool,
}

#[derive(Debug, Clone)]
pub struct Modifier {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>,
}

/// Solidity Parser
pub struct SolidityParser {
    // Parser state
}

impl SolidityParser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse Solidity source code
    pub fn parse(&self, source: &str) -> Result<SolidityAST, String> {
        // Extract pragma
        let pragma = self.extract_pragma(source);
        
        // Extract imports
        let imports = self.extract_imports(source);
        
        // Extract contracts
        let contracts = self.extract_contracts(source)?;
        
        Ok(SolidityAST {
            contracts,
            pragma,
            imports,
        })
    }
    
    fn extract_pragma(&self, source: &str) -> Option<String> {
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("pragma solidity") {
                return Some(line.to_string());
            }
        }
        None
    }
    
    fn extract_imports(&self, source: &str) -> Vec<String> {
        let mut imports = Vec::new();
        for line in source.lines() {
            let line = line.trim();
            if line.starts_with("import") {
                imports.push(line.to_string());
            }
        }
        imports
    }
    
    fn extract_contracts(&self, source: &str) -> Result<Vec<Contract>, String> {
        let mut contracts = Vec::new();
        let mut in_contract = false;
        let mut contract_start = 0;
        let mut brace_count = 0;
        let mut current_contract: Option<Contract> = None;
        
        let lines: Vec<&str> = source.lines().collect();
        let mut nested_stack: Vec<Contract> = Vec::new();
        let mut nested_brace_counts: Vec<i32> = Vec::new();
        let mut i = 0usize;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Detect contract/interface/library start (top-level or nested)
            let is_contract_start = trimmed.starts_with("contract ")
                || trimmed.starts_with("interface ")
                || trimmed.starts_with("abstract contract ")
                || trimmed.starts_with("library ");

            if is_contract_start && (current_contract.is_some() || !nested_stack.is_empty()) {
                // Nested contract: push onto stack (brace count for this line added below)
                let (name, kind) = self.parse_contract_declaration(trimmed)?;
                nested_stack.push(Contract {
                    name,
                    kind,
                    state_variables: Vec::new(),
                    functions: Vec::new(),
                    events: Vec::new(),
                    modifiers: Vec::new(),
                    structs: Vec::new(),
                    enums: Vec::new(),
                    inheritance: Vec::new(),
                    nested_contracts: Vec::new(),
                });
                nested_brace_counts.push(0);
            } else if is_contract_start {
                // Top-level contract
                let (name, kind) = self.parse_contract_declaration(trimmed)?;
                contract_start = i;
                in_contract = true;
                brace_count = 0;
                current_contract = Some(Contract {
                    name,
                    kind,
                    state_variables: Vec::new(),
                    functions: Vec::new(),
                    events: Vec::new(),
                    modifiers: Vec::new(),
                    structs: Vec::new(),
                    enums: Vec::new(),
                    inheritance: Vec::new(),
                    nested_contracts: Vec::new(),
                });
            }

            if in_contract {
                // Multi-line declarations: accumulate full declaration when at a keyword start
                let at_decl_start = Self::is_declaration_start(trimmed);
                let (content_to_parse, next_i) = if at_decl_start {
                    let (logical, next_i) = self.get_logical_line_for_declaration(&lines, i);
                    // Update brace counts for all lines we're consuming
                    for j in i..next_i {
                        let t = lines[j].trim();
                        brace_count += t.matches('{').count() as i32;
                        brace_count -= t.matches('}').count() as i32;
                        if !nested_brace_counts.is_empty() {
                            let last_idx = nested_brace_counts.len() - 1;
                            nested_brace_counts[last_idx] += t.matches('{').count() as i32;
                            nested_brace_counts[last_idx] -= t.matches('}').count() as i32;
                        }
                    }
                    (logical.trim().to_string(), next_i)
                } else {
                    (trimmed.to_string(), i + 1)
                };

                // Count braces for outer contract (single line only; multi-line already done above)
                if !at_decl_start {
                    brace_count += trimmed.matches('{').count() as i32;
                    brace_count -= trimmed.matches('}').count() as i32;
                    if !nested_brace_counts.is_empty() {
                        let last_idx = nested_brace_counts.len() - 1;
                        nested_brace_counts[last_idx] += trimmed.matches('{').count() as i32;
                        nested_brace_counts[last_idx] -= trimmed.matches('}').count() as i32;
                    }
                }

                // Update and close nested contracts
                while let Some(0) = nested_brace_counts.last() {
                    nested_brace_counts.pop();
                    if let Some(closed) = nested_stack.pop() {
                        if let Some(ref mut parent) = nested_stack.last_mut() {
                            parent.nested_contracts.push(closed);
                        } else if let Some(ref mut outer) = current_contract {
                            outer.nested_contracts.push(closed);
                        }
                    }
                }

                // Parse content for innermost contract (skip the line that started this or a nested contract)
                if !is_contract_start {
                    if nested_stack.is_empty() {
                        if let Some(ref mut contract) = current_contract {
                            self.parse_contract_line(&content_to_parse, contract)?;
                        }
                    } else if nested_brace_counts.last().copied().unwrap_or(0) > 0 {
                        if let Some(ref mut contract) = nested_stack.last_mut() {
                            self.parse_contract_line(&content_to_parse, contract)?;
                        }
                    }
                }

                // Advance index (skip consumed lines when we used get_logical_line)
                i = next_i;

                // Outer contract end
                if brace_count == 0 && i > contract_start {
                    if let Some(contract) = current_contract.take() {
                        contracts.push(contract);
                    }
                    in_contract = false;
                }
            } else {
                i += 1;
            }
        }
        
        // Handle unclosed contract
        if let Some(contract) = current_contract {
            contracts.push(contract);
        }
        
        Ok(contracts)
    }

    /// True if this line starts a declaration that may span multiple lines (function, constructor, receive, fallback, event, modifier).
    fn is_declaration_start(trimmed: &str) -> bool {
        trimmed.starts_with("function ")
            || trimmed.starts_with("constructor")
            || trimmed.starts_with("receive()")
            || trimmed.starts_with("receive ()")
            || trimmed.starts_with("fallback")
            || trimmed.starts_with("event ")
            || trimmed.starts_with("modifier ")
    }
    
    fn parse_contract_declaration(&self, line: &str) -> Result<(String, ContractKind), String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        let (kind, name_idx) = if line.starts_with("interface ") {
            (ContractKind::Interface, 1)
        } else if line.starts_with("abstract contract ") {
            (ContractKind::Abstract, 2)
        } else if line.starts_with("library ") {
            (ContractKind::Library, 1)
        } else {
            (ContractKind::Contract, 1)
        };
        
        if parts.len() <= name_idx {
            return Err("Invalid contract declaration".to_string());
        }
        
        let name = parts[name_idx].trim_end_matches('{').to_string();
        
        Ok((name, kind))
    }
    
    fn parse_contract_line(&self, line: &str, contract: &mut Contract) -> Result<(), String> {
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            return Ok(());
        }
        
        // Parse constructor, receive, fallback, or function
        if trimmed.starts_with("constructor") {
            if let Ok(func) = self.parse_constructor(line) {
                contract.functions.push(func);
            }
        } else if trimmed.starts_with("receive()") || trimmed.starts_with("receive ()") {
            if let Ok(func) = self.parse_receive(line) {
                contract.functions.push(func);
            }
        } else if trimmed.starts_with("fallback") {
            if let Ok(func) = self.parse_fallback(line) {
                contract.functions.push(func);
            }
        } else if trimmed.contains("function ") {
            if let Ok(func) = self.parse_function(line) {
                contract.functions.push(func);
            }
        }
        
        // Parse event
        if trimmed.starts_with("event ") {
            if let Ok(event) = self.parse_event(line) {
                contract.events.push(event);
            }
        }
        
        // Parse modifier
        if trimmed.starts_with("modifier ") {
            if let Ok(modifier) = self.parse_modifier(line) {
                contract.modifiers.push(modifier);
            }
        }
        
        // Parse state variable (simplified - variables without function keyword)
        if !trimmed.starts_with("function ") && 
           !trimmed.starts_with("event ") &&
           !trimmed.starts_with("modifier ") &&
           !trimmed.starts_with("struct ") &&
           !trimmed.starts_with("enum ") &&
           !trimmed.starts_with("constructor") &&
           trimmed.contains(' ') &&
           !trimmed.starts_with("//") {
            if let Ok(var) = self.parse_state_variable(line) {
                contract.state_variables.push(var);
            }
        }
        
        Ok(())
    }
    
    fn parse_function(&self, line: &str) -> Result<Function, String> {
        let mut func = Function {
            name: String::new(),
            parameters: Vec::new(),
            returns: Vec::new(),
            visibility: Visibility::Public,
            mutability: Mutability::NonPayable,
            modifiers: Vec::new(),
            body: None,
        };

        // Find first '(' for params (bracket-aware)
        let params_start = line.find('(').ok_or("function: missing '('")?;
        let params_end = Self::find_matching_paren(line, params_start)
            .ok_or("function: unclosed '(' in parameters")?;
        let params_str = line[params_start + 1..params_end].trim();
        func.parameters = self.parse_parameters(params_str)?;

        // Function name: between "function " and first '('
        if let Some(start) = line.find("function ") {
            let after_function = line[start + 9..params_start].trim();
            func.name = after_function.to_string();
        }

        // Visibility
        if line.contains(" public") {
            func.visibility = Visibility::Public;
        } else if line.contains(" external") {
            func.visibility = Visibility::External;
        } else if line.contains(" internal") {
            func.visibility = Visibility::Internal;
        } else if line.contains(" private") {
            func.visibility = Visibility::Private;
        }

        // Mutability
        if line.contains(" view") {
            func.mutability = Mutability::View;
        } else if line.contains(" pure") {
            func.mutability = Mutability::Pure;
        } else if line.contains(" payable") {
            func.mutability = Mutability::Payable;
        }

        // Returns: "returns(" ... ")" (bracket-aware)
        let after_params = &line[params_end + 1..];
        let returns_close_opt = if let Some(returns_open) = after_params.find("returns(") {
            let returns_abs = params_end + 1 + returns_open;
            let paren_start = returns_abs + 7; // '(' in "returns(" is at index 7
            Self::find_matching_paren(line, paren_start).map(|close| {
                let returns_str = &line[paren_start + 1..close];
                (close, returns_str)
            })
        } else {
            None
        };
        if let Some((_close, returns_str)) = returns_close_opt {
            func.returns = self.parse_parameters(returns_str)?;
        }

        // Modifiers, override, virtual: between returns ')' (or params ')') and '{' or ';'
        let modifiers_start = if let Some(returns_open) = after_params.find("returns(") {
            let returns_abs = params_end + 1 + returns_open;
            let paren_start = returns_abs + 7;
            Self::find_matching_paren(line, paren_start).map(|i| i + 1).unwrap_or(params_end + 1)
        } else {
            params_end + 1
        };
        let rest = line[modifiers_start..].trim();
        let modifiers_str = rest
            .split(&['{', ';'][..])
            .next()
            .unwrap_or("")
            .trim();
        for word in modifiers_str.split_whitespace() {
            let w = word.trim_end_matches(',');
            match w {
                "public" | "external" | "internal" | "private" | "view" | "pure" | "payable" => {}
                "override" | "virtual" => {
                    func.modifiers.push(w.to_string());
                }
                _ if !w.is_empty() => {
                    func.modifiers.push(w.to_string());
                }
                _ => {}
            }
        }

        // Extract function body when present (between first '{' and matching '}')
        if let Some(body_open) = line.find('{') {
            if let Some(body_close) = Self::find_matching_brace(line, body_open) {
                let body = line[body_open + 1..body_close].trim();
                if !body.is_empty() {
                    func.body = Some(body.to_string());
                }
            }
        }

        Ok(func)
    }

    fn parse_constructor(&self, line: &str) -> Result<Function, String> {
        let kw = line.find("constructor").ok_or("constructor: missing keyword")?;
        let from = &line[kw + 11..];
        let open_in_from = from.find('(').ok_or("constructor: missing '('")?;
        let params_start_abs = kw + 11 + open_in_from;
        let params_end = Self::find_matching_paren(line, params_start_abs).ok_or("constructor: unclosed '('")?;
        let params_str = line[params_start_abs + 1..params_end].trim();
        let parameters = self.parse_parameters(params_str)?;
        let rest = line[params_end + 1..].trim();
        let modifiers_str = rest.split(&['{', ';'][..]).next().unwrap_or("").trim();
        let mut modifiers = Vec::new();
        for word in modifiers_str.split_whitespace() {
            let w = word.trim_end_matches(',');
            if !["public", "external", "internal", "private", "view", "pure", "payable"].contains(&w) && !w.is_empty() {
                modifiers.push(w.to_string());
            }
        }
        Ok(Function {
            name: "constructor".to_string(),
            parameters,
            returns: Vec::new(),
            visibility: if line.contains(" public") { Visibility::Public } else if line.contains(" external") { Visibility::External } else { Visibility::Internal },
            mutability: if line.contains(" payable") { Mutability::Payable } else { Mutability::NonPayable },
            modifiers,
            body: None,
        })
    }

    fn parse_receive(&self, _line: &str) -> Result<Function, String> {
        Ok(Function {
            name: "receive".to_string(),
            parameters: Vec::new(),
            returns: Vec::new(),
            visibility: Visibility::External,
            mutability: Mutability::Payable,
            modifiers: Vec::new(),
            body: None,
        })
    }

    fn parse_fallback(&self, line: &str) -> Result<Function, String> {
        let kw = line.find("fallback").ok_or("fallback: missing keyword")?;
        let from = &line[kw + 8..];
        let open_in_from = from.find('(').ok_or("fallback: missing '('")?;
        let params_start_abs = kw + 8 + open_in_from;
        let params_end = Self::find_matching_paren(line, params_start_abs).ok_or("fallback: unclosed '('")?;
        let params_str = line[params_start_abs + 1..params_end].trim();
        let parameters = self.parse_parameters(params_str)?;
        let rest = line[params_end + 1..].trim();
        let visibility = if rest.contains(" external") { Visibility::External } else if rest.contains(" public") { Visibility::Public } else { Visibility::External };
        let mutability = if rest.contains(" payable") { Mutability::Payable } else { Mutability::NonPayable };
        Ok(Function {
            name: "fallback".to_string(),
            parameters,
            returns: Vec::new(),
            visibility,
            mutability,
            modifiers: Vec::new(),
            body: None,
        })
    }
    
    /// Find the index of the closing `)` that matches the `(` at `open_idx`.
    fn find_matching_paren(s: &str, open_idx: usize) -> Option<usize> {
        let mut depth = 1u32;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate().skip(open_idx + 1) {
            match b {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Find the index of the closing `}` that matches the `{` at `open_idx`.
    fn find_matching_brace(s: &str, open_idx: usize) -> Option<usize> {
        let mut depth = 1u32;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate().skip(open_idx + 1) {
            match b {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Returns (paren_depth, brace_depth) for the whole string.
    fn paren_brace_balance(s: &str) -> (i32, i32) {
        let mut p = 0i32;
        let mut b = 0i32;
        for &byte in s.as_bytes() {
            match byte {
                b'(' => p += 1,
                b')' => p -= 1,
                b'{' => b += 1,
                b'}' => b -= 1,
                _ => {}
            }
        }
        (p, b)
    }

    /// Accumulate lines from `lines[start..]` until we have a complete declaration (balanced parens/braces, ending with `;` or `{...}`).
    fn get_logical_line_for_declaration(&self, lines: &[&str], start: usize) -> (String, usize) {
        let mut s = lines[start].to_string();
        let mut i = start + 1;
        while i < lines.len() {
            let (p, b) = Self::paren_brace_balance(&s);
            let complete = p == 0 && b == 0 && (s.trim_end().ends_with(';') || s.contains('{'));
            if complete {
                break;
            }
            s.push('\n');
            s.push_str(lines[i]);
            i += 1;
        }
        (s, i)
    }

    /// Split by comma only at depth 0 (not inside `()`, `<>`, or `[]`) for complex types.
    fn split_parameters_at_top_level(s: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut depth_paren = 0i32;
        let mut depth_angle = 0i32;
        let mut depth_bracket = 0i32;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            match b {
                b'(' => depth_paren += 1,
                b')' => depth_paren -= 1,
                b'<' => depth_angle += 1,
                b'>' => depth_angle -= 1,
                b'[' => depth_bracket += 1,
                b']' => depth_bracket -= 1,
                b',' if depth_paren == 0 && depth_angle == 0 && depth_bracket == 0 => {
                    result.push(s[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            }
        }
        if start <= bytes.len() {
            result.push(s[start..].trim());
        }
        result
    }

    fn parse_parameters(&self, params_str: &str) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        if params_str.trim().is_empty() {
            return Ok(params);
        }
        for param_str in Self::split_parameters_at_top_level(params_str) {
            let param_str = param_str.trim();
            if param_str.is_empty() {
                continue;
            }
            let parts: Vec<&str> = param_str.split_whitespace().collect();
            if parts.len() >= 2 {
                // Last token is name, rest is type (e.g. "uint256" "x" or "mapping(address => uint256)" "balances")
                let name = parts[parts.len() - 1].to_string();
                let param_type = parts[..parts.len() - 1].join(" ");
                params.push(Parameter {
                    param_type,
                    name,
                });
            } else if parts.len() == 1 {
                params.push(Parameter {
                    param_type: parts[0].to_string(),
                    name: format!("param{}", params.len()),
                });
            }
        }
        Ok(params)
    }
    
    fn parse_event(&self, line: &str) -> Result<Event, String> {
        let mut event = Event {
            name: String::new(),
            parameters: Vec::new(),
            anonymous: line.contains("anonymous"),
        };
        
        if let Some(start) = line.find("event ") {
            let after_event = &line[start + 6..];
            if let Some(name_end) = after_event.find('(') {
                event.name = after_event[..name_end].trim().to_string();
            }
            
            if let Some(params_start) = line.find('(') {
                if let Some(params_end) = line.find(')') {
                    let params_str = &line[params_start + 1..params_end];
                    event.parameters = self.parse_parameters(params_str)?;
                }
            }
        }
        
        Ok(event)
    }
    
    fn parse_modifier(&self, line: &str) -> Result<Modifier, String> {
        let mut modifier = Modifier {
            name: String::new(),
            parameters: Vec::new(),
            body: None,
        };
        
        if let Some(start) = line.find("modifier ") {
            let after_modifier = &line[start + 9..];
            if let Some(name_end) = after_modifier.find('(') {
                modifier.name = after_modifier[..name_end].trim().to_string();
            }
            
            if let Some(params_start) = line.find('(') {
                if let Some(params_end) = line.find(')') {
                    let params_str = &line[params_start + 1..params_end];
                    modifier.parameters = self.parse_parameters(params_str)?;
                }
            }
        }
        
        Ok(modifier)
    }
    
    /// Find the last identifier (alphanumeric + underscore) in `s`, scanning from the end.
    /// Returns (name, start_index) so that s[start..end] is the name; everything before is type/modifiers.
    fn last_identifier(s: &str) -> Option<(&str, usize)> {
        let s = s.trim_end();
        let bytes = s.as_bytes();
        let mut end = bytes.len();
        while end > 0 {
            let c = bytes[end - 1];
            if c.is_ascii_whitespace() || c == b')' || c == b']' || c == b';' {
                end -= 1;
            } else {
                break;
            }
        }
        let mut start = end;
        while start > 0 {
            let c = bytes[start - 1];
            if c == b'_' || c.is_ascii_alphanumeric() {
                start -= 1;
            } else {
                break;
            }
        }
        if start < end && end <= bytes.len() {
            Some((&s[start..end], start))
        } else {
            None
        }
    }

    fn parse_state_variable(&self, line: &str) -> Result<StateVariable, String> {
        let trimmed = line.trim_end_matches(';').trim();

        // Before '=' (if any) is: type [visibility] [constant|immutable] name
        let (before_eq, initial_value) = if let Some(eq_pos) = trimmed.find('=') {
            (
                trimmed[..eq_pos].trim(),
                Some(trimmed[eq_pos + 1..].trim().trim_end_matches(';').trim().to_string()),
            )
        } else {
            (trimmed, None)
        };

        let (name, var_type) = if let Some((name_slice, name_start)) = Self::last_identifier(before_eq) {
            let name = name_slice.to_string();
            let before_name = before_eq[..name_start].trim();
            // Strip trailing visibility and mutability keywords to get the type
            const MODIFIERS: &[&str] = &["public", "internal", "private", "constant", "immutable"];
            let mut type_parts: Vec<&str> = before_name.split_whitespace().collect();
            while let Some(last) = type_parts.last() {
                let t = last.trim_end_matches(';');
                if MODIFIERS.contains(&t) {
                    type_parts.pop();
                } else {
                    break;
                }
            }
            let var_type = type_parts.join(" ");
            (name, var_type)
        } else {
            return Err("Invalid state variable: no variable name".to_string());
        };

        if var_type.is_empty() {
            return Err("Invalid state variable: missing type".to_string());
        }

        let visibility = if line.contains(" public") {
            Visibility::Public
        } else if line.contains(" internal") {
            Visibility::Internal
        } else if line.contains(" private") {
            Visibility::Private
        } else {
            Visibility::Internal
        };

        let mutability = if line.contains(" constant") || line.contains(" immutable") {
            Mutability::View
        } else {
            Mutability::NonPayable
        };

        Ok(StateVariable {
            name,
            var_type,
            visibility,
            mutability,
            initial_value,
        })
    }
}

