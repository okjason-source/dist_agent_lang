// DAL Code Generator - Generates DAL source code from DAL AST

use super::converter::{DALEvent, DALFunction, DALParameter, Field, Service, DALAST};

/// DAL Code Generator
pub struct DALGenerator {
    #[allow(dead_code)]
    indent_level: usize,
}

impl DALGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    /// Generate DAL source code from AST
    pub fn generate(&self, ast: DALAST) -> Result<String, String> {
        let mut code = String::new();

        for service in ast.services {
            code.push_str(&self.generate_service(service)?);
            code.push_str("\n\n");
        }

        Ok(code)
    }

    fn generate_service(&self, service: Service) -> Result<String, String> {
        let mut code = String::new();

        // Generate attributes
        for attr in &service.attributes {
            code.push_str(attr);
            code.push('\n');
        }

        // Generate service declaration
        code.push_str(&format!("service {} {{\n", service.name));

        // Generate fields
        if !service.fields.is_empty() {
            code.push_str("\n    // State variables\n");
            for field in &service.fields {
                code.push_str(&self.generate_field(field)?);
            }
        }

        // Generate functions (pass field names for storage→self in body conversion)
        let field_names: Vec<String> = service.fields.iter().map(|f| f.name.clone()).collect();
        if !service.functions.is_empty() {
            code.push_str("\n    // Functions\n");
            for func in &service.functions {
                code.push_str(&self.generate_function(func, &field_names)?);
            }
        }

        // Generate events
        if !service.events.is_empty() {
            code.push_str("\n    // Events\n");
            for event in &service.events {
                code.push_str(&self.generate_event(event)?);
            }
        }

        code.push_str("}\n");

        Ok(code)
    }

    fn generate_field(&self, field: &Field) -> Result<String, String> {
        let mut code = String::new();

        code.push_str(&format!("    {}: {} = ", field.name, field.field_type));

        if let Some(ref init) = field.initial_value {
            code.push_str(init);
        } else {
            // Default values based on type
            code.push_str(&self.default_value_for_type(&field.field_type));
        }

        code.push_str(",\n");

        Ok(code)
    }

    fn default_value_for_type(&self, dal_type: &str) -> String {
        match dal_type {
            "int" => "0".to_string(),
            "bool" => "false".to_string(),
            "string" => "\"\"".to_string(),
            "vector<u8>" => "[]".to_string(),
            _ if dal_type.starts_with("vector<") => "[]".to_string(),
            _ if dal_type.starts_with("map<") => "{}".to_string(),
            _ => "null".to_string(),
        }
    }

    fn generate_function(
        &self,
        func: &DALFunction,
        field_names: &[String],
    ) -> Result<String, String> {
        let mut code = String::new();

        // Generate attributes
        for attr in &func.attributes {
            code.push_str(&format!("    {}\n", attr));
        }

        // Generate function signature
        code.push_str(&format!(
            "    fn {}({})",
            func.name,
            self.generate_parameters(&func.parameters)?
        ));

        // Generate return type
        if let Some(ref return_type) = func.return_type {
            code.push_str(&format!(" -> {}", return_type));
        }

        code.push_str(" {\n");

        if let Some(ref c) = func.comment {
            code.push_str(&format!("        // {}\n", c));
        }

        // Generate function body (field_names used for storage→self in Solidity body)
        if let Some(ref body) = func.body {
            let dal_body = self.convert_solidity_body_to_dal(body, field_names);
            code.push_str(&dal_body);
        } else {
            code.push_str("        // Function implementation\n");
        }

        code.push_str("    }\n\n");

        Ok(code)
    }

    /// Solidity→DAL body conversion: storage→self; require/revert→if/throw; transfer/call→chain::; msg.sender→chain::caller(); assignments pass-through; rest as comments.
    fn convert_solidity_body_to_dal(&self, body: &str, field_names: &[String]) -> String {
        let body_with_self = Self::replace_state_vars_with_self(body, field_names);
        let mut out = String::new();
        for line in body_with_self.lines() {
            let trimmed = line.trim();
            let dal = self
                .convert_require_line(trimmed)
                .or_else(|| self.convert_revert_line(trimmed))
                .or_else(|| self.convert_transfer_call_line(trimmed))
                .or_else(|| self.convert_emit_line(trimmed))
                .or_else(|| self.convert_control_flow_line(trimmed))
                .or_else(|| self.convert_generic_line(trimmed));
            match dal {
                Some(s) => out.push_str(&format!("        {}\n", s)),
                None => out.push_str(&format!("        // {}\n", line.trim())),
            }
        }
        if out.is_empty() {
            out.push_str("        // (no statements converted)\n");
        }
        out
    }

    /// Replace whole-word occurrences of each field name with self.<name> (Solidity storage → DAL self.field).
    fn replace_state_vars_with_self(body: &str, field_names: &[String]) -> String {
        let mut out = body.to_string();
        for name in field_names {
            if name.is_empty() {
                continue;
            }
            out = Self::replace_word_boundary(&out, name, &format!("self.{}", name));
        }
        out
    }

    /// Replace whole-word occurrences of `word` with `replacement` (word boundary: not part of a longer identifier).
    fn replace_word_boundary(s: &str, word: &str, replacement: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let bytes = s.as_bytes();
        let w = word.as_bytes();
        let mut i = 0;
        while i <= bytes.len() {
            if i + w.len() <= bytes.len() && &bytes[i..i + w.len()] == w {
                let word_start_ok =
                    i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
                let word_end_ok = i + w.len() == bytes.len()
                    || !bytes[i + w.len()].is_ascii_alphanumeric() && bytes[i + w.len()] != b'_';
                if word_start_ok && word_end_ok {
                    result.push_str(replacement);
                    i += w.len();
                    continue;
                }
            }
            if i < bytes.len() {
                result.push(bytes[i] as char);
            }
            i += 1;
        }
        result
    }

    /// If line is require(cond); or require(cond, "msg"); emit DAL if (!cond) { throw ... }; else None.
    fn convert_require_line(&self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(';').trim();
        if !line.starts_with("require(") {
            return None;
        }
        let inner = line["require(".len()..].trim();
        let (cond, msg) = if let Some(comma_pos) = inner.find(", ") {
            let (c, m) = inner.split_at(comma_pos);
            (c.trim(), m[1..].trim().trim_matches('"'))
        } else {
            (
                inner.trim_end_matches(';').trim_end_matches(')').trim(),
                "require failed",
            )
        };
        if cond.is_empty() {
            return None;
        }
        Some(format!("if (!({})) {{ throw \"{}\"; }}", cond, msg))
    }

    /// If line is addr.transfer(amount) or addr.call(...) emit DAL chain::call(...); else None.
    fn convert_transfer_call_line(&self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(';').trim();
        if let Some(transfer_pos) = line.find(".transfer(") {
            let receiver = line[..transfer_pos].trim();
            let open_paren = transfer_pos + ".transfer(".len() - 1;
            if let Some(close) = Self::find_matching_paren(line, open_paren) {
                let arg = line[open_paren + 1..close].trim();
                return Some(format!(
                    "chain::call(1, {}, \"transfer\", {{\"amount\": {}}});  // chain_id=1; adjust if needed",
                    receiver, arg
                ));
            }
        }
        if line.contains(".call(") || line.contains(".call{") {
            let dot_call = line
                .find(".call(")
                .or_else(|| line.find(".call{"))
                .unwrap_or(0);
            let addr = line[..dot_call].trim();
            return Some(format!(
                "chain::call(1, {}, \"call\", {{}});  // Solidity .call - fill function name and args as needed",
                addr
            ));
        }
        None
    }

    /// Index of the closing ')' that matches the '(' at open_idx.
    fn find_matching_paren(s: &str, open_idx: usize) -> Option<usize> {
        let bytes = s.as_bytes();
        if open_idx >= bytes.len() || bytes[open_idx] != b'(' {
            return None;
        }
        let mut depth = 1u32;
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

    /// If line is emit EventName(...); pass through (DAL uses same emit syntax). Else None.
    fn convert_emit_line(&self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(';').trim();
        if line.starts_with("emit ") {
            let mut s = line.to_string();
            if !s.ends_with(';') {
                s.push(';');
            }
            return Some(s);
        }
        None
    }

    /// Pass-through for control-flow: if/for/while/else, do-while, try/catch, unchecked/assembly, and lone `{`/`}` (body already has storage→self and msg.sender replaced).
    fn convert_control_flow_line(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed == "}" || trimmed == "{" {
            return Some(trimmed.to_string());
        }
        if trimmed.starts_with("if ")
            || trimmed.starts_with("if(")
            || trimmed.starts_with("for ")
            || trimmed.starts_with("for(")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("while(")
            || trimmed.starts_with("do ")
            || trimmed.starts_with("} while(")
            || trimmed.starts_with("else ")
            || trimmed.starts_with("else{")
            || trimmed.starts_with("else {")
        {
            return Some(trimmed.to_string());
        }
        if trimmed.starts_with("} else") {
            return Some(trimmed.to_string());
        }
        if trimmed.starts_with("try ")
            || trimmed.starts_with("} catch")
            || trimmed.starts_with("catch ")
            || trimmed.starts_with("catch(")
        {
            return Some(trimmed.to_string());
        }
        if trimmed.starts_with("unchecked ")
            || trimmed.starts_with("unchecked{")
            || trimmed.starts_with("unchecked {")
        {
            return Some(trimmed.to_string());
        }
        if trimmed.starts_with("assembly ")
            || trimmed.starts_with("assembly{")
            || trimmed.starts_with("assembly {")
        {
            return Some(trimmed.to_string());
        }
        None
    }

    /// If line is revert(); or revert("msg"); emit DAL throw "msg"; else None.
    fn convert_revert_line(&self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(';').trim();
        if !line.starts_with("revert(") {
            return None;
        }
        let inner = line["revert(".len()..]
            .trim_end_matches(')')
            .trim()
            .trim_matches('"');
        let msg = if inner.is_empty() { "reverted" } else { inner };
        Some(format!("throw \"{}\";", msg))
    }

    /// Pass-through for assignments and return; replace msg.sender with chain::caller().
    /// Returns None to emit as comment if line is empty, only braces, or not a simple statement.
    fn convert_generic_line(&self, line: &str) -> Option<String> {
        let line = line.trim_end_matches(';').trim();
        if line.is_empty()
            || line == "{"
            || line == "}"
            || line.starts_with("//")
            || line.starts_with("/*")
        {
            return None;
        }
        let mut dal = line.to_string();
        dal = dal.replace("msg.sender", "chain::caller()");
        let is_msg_sender = dal != line;
        let is_return = line.starts_with("return ");
        let is_assignment = Self::looks_like_assignment(line);
        if is_msg_sender || is_return || is_assignment {
            if !dal.ends_with(';') && !dal.ends_with('}') {
                dal.push(';');
            }
            Some(dal)
        } else {
            None
        }
    }

    /// True if line looks like a single assignment (lhs = rhs), not a comparison (==, !=, <=, >=).
    fn looks_like_assignment(line: &str) -> bool {
        if let Some(idx) = line.find(" = ") {
            let before = line[..idx].trim_end();
            let after = line[idx + 3..].trim_start();
            !before.ends_with('=')
                && !before.ends_with('!')
                && !before.ends_with('<')
                && !before.ends_with('>')
                && !after.starts_with('=')
        } else {
            false
        }
    }

    fn generate_parameters(&self, params: &[DALParameter]) -> Result<String, String> {
        if params.is_empty() {
            return Ok(String::new());
        }

        let param_strings: Vec<String> = params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.param_type))
            .collect();

        Ok(param_strings.join(", "))
    }

    fn generate_event(&self, event: &DALEvent) -> Result<String, String> {
        let mut code = String::new();

        code.push_str(&format!(
            "    event {}({});\n",
            event.name,
            self.generate_parameters(&event.parameters)?
        ));

        Ok(code)
    }
}

impl Default for DALGenerator {
    fn default() -> Self {
        Self::new()
    }
}
