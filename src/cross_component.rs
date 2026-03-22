//! Cross-component CLI: bond, pipe, invoke.
//!
//! bond flows (auth-to-web, oracle-to-chain); pipe; invoke.

/// Result of running a cross-component command. CLI uses this to print and set exit code.
#[derive(Debug)]
pub struct RunResult {
    pub success: bool,
    pub message: String,
    pub exit_code: i32,
}

/// Options for cross-component commands (bond, pipe, invoke).
#[derive(Debug, Default)]
pub struct RunOptions {
    pub dry_run: bool,
    pub format_json: bool,
    /// Resolved auth token (from --token, --token-env, or --auth-file).
    pub token: Option<String>,
}

/// Strip --dry-run, --format, --token, --token-env, --auth-file and their values from args.
/// These are global CLI flags that may appear in rest when user puts them after subcommand.
fn strip_global_flags(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "--dry-run" {
            i += 1;
        } else if a == "--format" || a == "--token" || a == "--token-env" || a == "--auth-file" {
            i += 1;
            if i < args.len() && !args[i].starts_with('-') {
                i += 1; // skip value
            }
        } else {
            out.push(a.clone());
            i += 1;
        }
    }
    out
}

const BOND_FLOWS: &[&str] = &[
    "oracle-to-chain",
    "chain-to-sync",
    "iot-to-db",
    "iot-to-web",
    "db-to-sync",
    "sync-to-db",
    "ai-to-service",
    "service-to-chain",
    "auth-to-web",
    "log-to-sync",
];

/// Run bond flow. Args: [flow, ...rest]. Supports --dry-run, auth token, --format json.
fn run_bond(args: &[String], opts: &RunOptions) -> RunResult {
    let args = strip_global_flags(args);
    if args.is_empty() {
        return RunResult {
            success: false,
            message: "bond: missing flow name. Use: dal bond <flow> [args...]".to_string(),
            exit_code: 1,
        };
    }
    let flow = &args[0];
    let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();

    if !BOND_FLOWS.contains(&flow.as_str()) {
        return RunResult {
            success: false,
            message: format!(
                "bond: unknown flow '{}'. Flows: {}",
                flow,
                BOND_FLOWS.join(", ")
            ),
            exit_code: 1,
        };
    }

    let dry_run = opts.dry_run;
    match flow.as_str() {
        "auth-to-web" => run_bond_auth_to_web(&rest, dry_run, opts),
        "oracle-to-chain" => run_bond_oracle_to_chain(&rest, dry_run),
        "chain-to-sync" => run_bond_chain_to_sync(&rest, dry_run),
        "iot-to-db" => run_bond_iot_to_db(&rest, dry_run),
        "iot-to-web" => run_bond_iot_to_web(&rest, dry_run, opts),
        "db-to-sync" => run_bond_db_to_sync(&rest, dry_run),
        "sync-to-db" => run_bond_sync_to_db(&rest, dry_run),
        "ai-to-service" => run_bond_ai_to_service(&rest, dry_run, opts),
        "service-to-chain" => run_bond_service_to_chain(&rest, dry_run, opts),
        "log-to-sync" => run_bond_log_to_sync(&rest, dry_run),
        _ => RunResult {
            success: false,
            message: format!(
                "bond {}: not yet implemented (stub). Use --dry-run to see planned steps.",
                flow
            ),
            exit_code: 0,
        },
    }
}

/// bond auth-to-web: `{token} {url} {method}` or --token/--token-env/--auth-file with `{url} {method}`.
fn run_bond_auth_to_web(args: &[&str], dry_run: bool, opts: &RunOptions) -> RunResult {
    // Token: from opts (--token/--token-env/--auth-file) or first positional arg
    let token = if let Some(ref t) = opts.token {
        t.as_str()
    } else if args.len() >= 2 {
        args[0]
    } else {
        return RunResult {
            success: false,
            message: "bond auth-to-web: usage: dal bond auth-to-web <token> <url> [method] (or use --token/--token-env/--auth-file with <url> [method])".to_string(),
            exit_code: 1,
        };
    };
    let (url, method) = if opts.token.is_some() {
        if args.is_empty() {
            return RunResult {
                success: false,
                message:
                    "bond auth-to-web: usage: dal bond auth-to-web <url> [method] (with --token)"
                        .to_string(),
                exit_code: 1,
            };
        }
        (
            args[0],
            args.get(1).copied().unwrap_or("GET").to_uppercase(),
        )
    } else {
        if args.len() < 2 {
            return RunResult {
                success: false,
                message:
                    "bond auth-to-web: usage: dal bond auth-to-web <token> <url> [GET|POST|...]"
                        .to_string(),
                exit_code: 1,
            };
        }
        (
            args[1],
            args.get(2).copied().unwrap_or("GET").to_uppercase(),
        )
    };

    if dry_run {
        let masked = if token.len() > 8 {
            format!("{}***", &token[..4])
        } else {
            "***".to_string()
        };
        return RunResult {
            success: true,
            message: format!(
                "bond auth-to-web (dry-run): {} {} with Authorization: Bearer {}",
                method, url, masked
            ),
            exit_code: 0,
        };
    }

    #[cfg(feature = "http-interface")]
    {
        match do_auth_to_web_request(token, url, &method) {
            Ok(status) => RunResult {
                success: (200..300).contains(&status),
                message: format!("bond auth-to-web: {} {} → {}", method, url, status),
                exit_code: if (200..300).contains(&status) { 0 } else { 1 },
            },
            Err(e) => RunResult {
                success: false,
                message: format!("bond auth-to-web failed at web: {}", e),
                exit_code: 1,
            },
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = (token, url, method);
        RunResult {
            success: false,
            message: "bond auth-to-web requires http-interface feature (reqwest). Build with default features.".to_string(),
            exit_code: 1,
        }
    }
}

/// bond oracle-to-chain: `{source} {query} {chain_id}` [--use-as gas|price|param]. Fetch from oracle, pass to chain (e.g. gas estimate).
fn run_bond_oracle_to_chain(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 3 {
        return RunResult {
            success: false,
            message: "bond oracle-to-chain: usage: dal bond oracle-to-chain <source_url> <query_type> <chain_id> [--use-as gas|price|param]".to_string(),
            exit_code: 1,
        };
    }
    let source = args[0];
    let query_type = args[1];
    let chain_id: i64 = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            return RunResult {
                success: false,
                message: format!("bond oracle-to-chain: invalid chain_id '{}'", args[2]),
                exit_code: 1,
            };
        }
    };
    let use_as = args
        .windows(2)
        .find(|w| w[0] == "--use-as")
        .and_then(|w| w.get(1))
        .copied()
        .unwrap_or("price");

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond oracle-to-chain (dry-run): fetch {} query {} → chain {} use-as {}",
                source, query_type, chain_id, use_as
            ),
            exit_code: 0,
        };
    }

    #[cfg(feature = "http-interface")]
    {
        use dist_agent_lang::stdlib::chain;
        use dist_agent_lang::stdlib::oracle::{self, OracleQuery};

        let query = OracleQuery::new(query_type.to_string()).require_signature(false);
        match oracle::fetch(source, query) {
            Ok(_response) => {
                let gas_price = chain::get_gas_price(chain_id);
                let msg = format!(
                    "bond oracle-to-chain: oracle data → chain {} (use-as {}); chain gas_price={}",
                    chain_id, use_as, gas_price
                );
                RunResult {
                    success: true,
                    message: msg,
                    exit_code: 0,
                }
            }
            Err(e) => RunResult {
                success: false,
                message: format!("bond oracle-to-chain failed at oracle: {}", e),
                exit_code: 1,
            },
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = (source, query_type, chain_id, use_as);
        RunResult {
            success: false,
            message:
                "bond oracle-to-chain requires http-interface feature. Build with default features."
                    .to_string(),
            exit_code: 1,
        }
    }
}

/// bond chain-to-sync: `{chain_id} {tx_hash} {sync_url}`. Chain tx status → sync push.
fn run_bond_chain_to_sync(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 3 {
        return RunResult {
            success: false,
            message:
                "bond chain-to-sync: usage: dal bond chain-to-sync <chain_id> <tx_hash> <sync_url>"
                    .to_string(),
            exit_code: 1,
        };
    }
    let chain_id: i64 = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            return RunResult {
                success: false,
                message: format!("bond chain-to-sync: invalid chain_id '{}'", args[0]),
                exit_code: 1,
            };
        }
    };
    let tx_hash = args[1];
    let sync_url = args[2];

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond chain-to-sync (dry-run): chain {} tx {} → sync {}",
                chain_id, tx_hash, sync_url
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::runtime::values::Value as DalValue;
    use dist_agent_lang::stdlib::chain;
    use dist_agent_lang::stdlib::sync::{self, SyncTarget};

    let status = chain::get_transaction_status(chain_id, tx_hash.to_string());
    let mut data = std::collections::HashMap::new();
    data.insert("chain_id".to_string(), DalValue::Int(chain_id));
    data.insert("tx_hash".to_string(), DalValue::String(tx_hash.to_string()));
    data.insert("status".to_string(), DalValue::String(status.clone()));
    let protocol = if sync_url.starts_with("https://") {
        "https"
    } else {
        "http"
    };
    let target = SyncTarget::new(sync_url.to_string(), protocol.to_string());
    match sync::push(data, target) {
        Ok(true) => RunResult {
            success: true,
            message: format!(
                "bond chain-to-sync: chain {} tx {} status={} → sync ok",
                chain_id, tx_hash, status
            ),
            exit_code: 0,
        },
        Ok(false) => RunResult {
            success: false,
            message: "bond chain-to-sync failed at sync: push returned false".to_string(),
            exit_code: 1,
        },
        Err(e) => RunResult {
            success: false,
            message: format!("bond chain-to-sync failed at sync: {}", e),
            exit_code: 1,
        },
    }
}

/// bond iot-to-db: `{device_id} {conn_str}` [--table name]. IoT read → DB insert.
fn run_bond_iot_to_db(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message:
                "bond iot-to-db: usage: dal bond iot-to-db <device_id> <conn_str> [--table name]"
                    .to_string(),
            exit_code: 1,
        };
    }
    let device_id = args[0];
    let conn_str = args[1];
    let table = args
        .windows(2)
        .find(|w| w[0] == "--table")
        .and_then(|w| w.get(1))
        .copied()
        .unwrap_or("sensor_readings");

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond iot-to-db (dry-run): iot read {} → db {} table {}",
                device_id, conn_str, table
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::database;
    use dist_agent_lang::stdlib::iot;

    let reading = match iot::read_sensor_data(device_id) {
        Ok(r) => r,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond iot-to-db failed at iot: {}", e),
                exit_code: 1,
            };
        }
    };
    let db = match database::connect(conn_str.to_string()) {
        Ok(d) => d,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond iot-to-db failed at db: {}", e),
                exit_code: 1,
            };
        }
    };
    let sql = format!(
        "INSERT INTO {} (sensor_id, timestamp, value) VALUES ('{}', '{}', {})",
        table,
        device_id,
        reading.timestamp,
        match reading.value {
            dist_agent_lang::runtime::values::Value::Float(f) => format!("{}", f),
            dist_agent_lang::runtime::values::Value::Int(i) => format!("{}", i),
            _ => "NULL".to_string(),
        }
    );
    match database::query(&db, sql, vec![]) {
        Ok(_) => RunResult {
            success: true,
            message: format!("bond iot-to-db: iot {} → db {} ok", device_id, conn_str),
            exit_code: 0,
        },
        Err(e) => RunResult {
            success: false,
            message: format!("bond iot-to-db failed at db query: {}", e),
            exit_code: 1,
        },
    }
}

/// bond iot-to-web: `{device_id} {url}` [--auth token]. IoT read → web POST.
fn run_bond_iot_to_web(args: &[&str], dry_run: bool, opts: &RunOptions) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message: "bond iot-to-web: usage: dal bond iot-to-web <device_id> <url> [--auth token]"
                .to_string(),
            exit_code: 1,
        };
    }
    let device_id = args[0];
    let url = args[1];
    let token = opts.token.as_deref().or_else(|| {
        args.windows(2)
            .find(|w| w[0] == "--auth")
            .and_then(|w| w.get(1))
            .copied()
    });

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond iot-to-web (dry-run): iot read {} → POST {} {}",
                device_id,
                url,
                if token.is_some() { "(with auth)" } else { "" }
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::iot;

    let reading = match iot::read_sensor_data(device_id) {
        Ok(r) => r,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond iot-to-web failed at iot: {}", e),
                exit_code: 1,
            };
        }
    };
    let payload = serde_json::json!({
        "device_id": device_id,
        "timestamp": reading.timestamp,
        "value": reading.value,
    });

    #[cfg(feature = "http-interface")]
    {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| e.to_string());
        let client = match client {
            Ok(c) => c,
            Err(e) => {
                return RunResult {
                    success: false,
                    message: format!("bond iot-to-web failed: {}", e),
                    exit_code: 1,
                };
            }
        };
        let mut req = client.post(url).json(&payload);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        match req.send() {
            Ok(resp) => {
                let status = resp.status();
                RunResult {
                    success: status.is_success(),
                    message: format!(
                        "bond iot-to-web: iot {} → POST {} → {}",
                        device_id, url, status
                    ),
                    exit_code: if status.is_success() { 0 } else { 1 },
                }
            }
            Err(e) => RunResult {
                success: false,
                message: format!("bond iot-to-web failed at web: {}", e),
                exit_code: 1,
            },
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = (reading, payload, token);
        RunResult {
            success: false,
            message: "bond iot-to-web requires http-interface feature.".to_string(),
            exit_code: 1,
        }
    }
}

/// bond db-to-sync: `{conn_str} {sync_url}` [--query sql]. DB query → sync push.
fn run_bond_db_to_sync(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message:
                "bond db-to-sync: usage: dal bond db-to-sync <conn_str> <sync_url> [--query sql]"
                    .to_string(),
            exit_code: 1,
        };
    }
    let conn_str = args[0];
    let sync_url = args[1];
    let query_sql = args
        .windows(2)
        .find(|w| w[0] == "--query")
        .and_then(|w| w.get(1))
        .copied()
        .unwrap_or("SELECT 1");

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond db-to-sync (dry-run): db {} query → sync {}",
                conn_str, sync_url
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::runtime::values::Value as DalValue;
    use dist_agent_lang::stdlib::database;
    use dist_agent_lang::stdlib::sync::{self, SyncTarget};

    let db = match database::connect(conn_str.to_string()) {
        Ok(d) => d,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond db-to-sync failed at db: {}", e),
                exit_code: 1,
            };
        }
    };
    let qr = match database::query(&db, query_sql.to_string(), vec![]) {
        Ok(r) => r,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond db-to-sync failed at query: {}", e),
                exit_code: 1,
            };
        }
    };
    let mut data = std::collections::HashMap::new();
    data.insert("row_count".to_string(), DalValue::Int(qr.row_count));
    data.insert(
        "rows".to_string(),
        DalValue::List(
            qr.rows
                .iter()
                .map(|r| {
                    let mut m = std::collections::HashMap::new();
                    for (k, v) in r {
                        m.insert(k.clone(), v.clone());
                    }
                    DalValue::Map(m)
                })
                .collect(),
        ),
    );
    let protocol = if sync_url.starts_with("https://") {
        "https"
    } else {
        "http"
    };
    let target = SyncTarget::new(sync_url.to_string(), protocol.to_string());
    match sync::push(data, target) {
        Ok(true) => RunResult {
            success: true,
            message: format!("bond db-to-sync: db query {} rows → sync ok", qr.row_count),
            exit_code: 0,
        },
        Ok(false) => RunResult {
            success: false,
            message: "bond db-to-sync failed at sync: push returned false".to_string(),
            exit_code: 1,
        },
        Err(e) => RunResult {
            success: false,
            message: format!("bond db-to-sync failed at sync: {}", e),
            exit_code: 1,
        },
    }
}

/// bond sync-to-db: `{sync_url} {conn_str}` [--table name]. Sync pull → DB insert.
fn run_bond_sync_to_db(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message:
                "bond sync-to-db: usage: dal bond sync-to-db <sync_url> <conn_str> [--table name]"
                    .to_string(),
            exit_code: 1,
        };
    }
    let sync_url = args[0];
    let conn_str = args[1];
    let table = args
        .windows(2)
        .find(|w| w[0] == "--table")
        .and_then(|w| w.get(1))
        .copied()
        .unwrap_or("synced_data");

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond sync-to-db (dry-run): sync {} → db {} table {}",
                sync_url, conn_str, table
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::database;
    use dist_agent_lang::stdlib::sync::{self, SyncFilters};

    let (data, _) = match sync::pull(sync_url, SyncFilters::new()) {
        Ok(r) => r,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond sync-to-db failed at sync: {}", e),
                exit_code: 1,
            };
        }
    };
    let db = match database::connect(conn_str.to_string()) {
        Ok(d) => d,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond sync-to-db failed at db: {}", e),
                exit_code: 1,
            };
        }
    };
    let keys: Vec<String> = data.keys().cloned().collect();
    let sql = format!(
        "INSERT INTO {} (key, value) VALUES ('{}', 'synced')",
        table,
        keys.first().cloned().unwrap_or_else(|| "data".to_string())
    );
    match database::query(&db, sql, vec![]) {
        Ok(_) => RunResult {
            success: true,
            message: format!("bond sync-to-db: sync {} keys → db ok", data.len()),
            exit_code: 0,
        },
        Err(e) => RunResult {
            success: false,
            message: format!("bond sync-to-db failed at db: {}", e),
            exit_code: 1,
        },
    }
}

/// bond ai-to-service: `{prompt} {service_url}` [--model]. AI generate → web POST.
fn run_bond_ai_to_service(args: &[&str], dry_run: bool, opts: &RunOptions) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message: "bond ai-to-service: usage: dal bond ai-to-service <prompt> <service_url> [--model name]".to_string(),
            exit_code: 1,
        };
    }
    let prompt = args[0];
    let service_url = args[1];
    let _model = args
        .windows(2)
        .find(|w| w[0] == "--model")
        .and_then(|w| w.get(1))
        .copied();

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond ai-to-service (dry-run): ai generate → POST {}",
                service_url
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::ai;

    let generated = match ai::generate_text(prompt.to_string()) {
        Ok(s) => s,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("bond ai-to-service failed at ai: {}", e),
                exit_code: 1,
            };
        }
    };

    #[cfg(feature = "http-interface")]
    {
        let payload = serde_json::json!({ "content": generated, "prompt": prompt });
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| e.to_string());
        let client = match client {
            Ok(c) => c,
            Err(e) => {
                return RunResult {
                    success: false,
                    message: format!("bond ai-to-service failed: {}", e),
                    exit_code: 1,
                };
            }
        };
        let mut req = client.post(service_url).json(&payload);
        if let Some(ref t) = opts.token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        match req.send() {
            Ok(resp) => {
                let status = resp.status();
                RunResult {
                    success: status.is_success(),
                    message: format!("bond ai-to-service: ai → POST {} → {}", service_url, status),
                    exit_code: if status.is_success() { 0 } else { 1 },
                }
            }
            Err(e) => RunResult {
                success: false,
                message: format!("bond ai-to-service failed at web: {}", e),
                exit_code: 1,
            },
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = generated;
        RunResult {
            success: false,
            message: "bond ai-to-service requires http-interface feature.".to_string(),
            exit_code: 1,
        }
    }
}

/// bond service-to-chain: `{service_url} {chain_id} {addr} {fn_name}`. Service result → chain call.
fn run_bond_service_to_chain(args: &[&str], dry_run: bool, opts: &RunOptions) -> RunResult {
    if args.len() < 4 {
        return RunResult {
            success: false,
            message: "bond service-to-chain: usage: dal bond service-to-chain <service_url> <chain_id> <addr> <fn>".to_string(),
            exit_code: 1,
        };
    }
    let service_url = args[0];
    let chain_id: i64 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            return RunResult {
                success: false,
                message: format!("bond service-to-chain: invalid chain_id '{}'", args[1]),
                exit_code: 1,
            };
        }
    };
    let addr = args[2];
    let fn_name = args[3];

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond service-to-chain (dry-run): GET {} → chain {} call {} at {}",
                service_url, chain_id, fn_name, addr
            ),
            exit_code: 0,
        };
    }

    #[cfg(feature = "http-interface")]
    {
        use dist_agent_lang::stdlib::chain;

        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| e.to_string());
        let client = match client {
            Ok(c) => c,
            Err(e) => {
                return RunResult {
                    success: false,
                    message: format!("bond service-to-chain failed: {}", e),
                    exit_code: 1,
                };
            }
        };
        let mut req = client.get(service_url);
        if let Some(ref t) = opts.token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        let resp = match req.send() {
            Ok(r) => r,
            Err(e) => {
                return RunResult {
                    success: false,
                    message: format!("bond service-to-chain failed at web: {}", e),
                    exit_code: 1,
                };
            }
        };
        if !resp.status().is_success() {
            return RunResult {
                success: false,
                message: format!("bond service-to-chain failed at service: {}", resp.status()),
                exit_code: 1,
            };
        }
        let _body = resp.text().unwrap_or_default();
        let gas = chain::estimate_gas(chain_id, fn_name.to_string());
        RunResult {
            success: true,
            message: format!(
                "bond service-to-chain: service → chain {} call {} at {} (gas_est={})",
                chain_id, fn_name, addr, gas
            ),
            exit_code: 0,
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = (service_url, chain_id, addr, fn_name);
        RunResult {
            success: false,
            message: "bond service-to-chain requires http-interface feature.".to_string(),
            exit_code: 1,
        }
    }
}

/// bond log-to-sync: `{sync_url}` [--level info|warn|error|audit]. Log entries → sync push.
fn run_bond_log_to_sync(args: &[&str], dry_run: bool) -> RunResult {
    if args.is_empty() {
        return RunResult {
            success: false,
            message: "bond log-to-sync: usage: dal bond log-to-sync <sync_url> [--level info|warn|error|audit]".to_string(),
            exit_code: 1,
        };
    }
    let sync_url = args[0];
    let level_filter = args
        .windows(2)
        .find(|w| w[0] == "--level")
        .and_then(|w| w.get(1))
        .copied();

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "bond log-to-sync (dry-run): log {} → sync {}",
                level_filter.unwrap_or("all"),
                sync_url
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::runtime::values::Value as DalValue;
    use dist_agent_lang::stdlib::log::{self, LogLevel};
    use dist_agent_lang::stdlib::sync::{self, SyncTarget};

    let entries = if let Some(level_str) = level_filter {
        let lvl = match level_str.to_lowercase().as_str() {
            "warn" | "warning" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "audit" => LogLevel::Audit,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        };
        log::get_entries_by_level(lvl)
    } else {
        log::get_entries()
    };
    let mut data = std::collections::HashMap::new();
    data.insert("count".to_string(), DalValue::Int(entries.len() as i64));
    data.insert(
        "entries".to_string(),
        DalValue::List(
            entries
                .iter()
                .take(100)
                .map(|e| {
                    let mut m = std::collections::HashMap::new();
                    m.insert("message".to_string(), DalValue::String(e.message.clone()));
                    m.insert(
                        "level".to_string(),
                        DalValue::String(format!("{:?}", e.level)),
                    );
                    m.insert("source".to_string(), DalValue::String(e.source.clone()));
                    DalValue::Map(m)
                })
                .collect(),
        ),
    );
    let protocol = if sync_url.starts_with("https://") {
        "https"
    } else {
        "http"
    };
    let target = SyncTarget::new(sync_url.to_string(), protocol.to_string());
    match sync::push(data, target) {
        Ok(true) => RunResult {
            success: true,
            message: format!("bond log-to-sync: log {} entries → sync ok", entries.len()),
            exit_code: 0,
        },
        Ok(false) => RunResult {
            success: false,
            message: "bond log-to-sync failed at sync: push returned false".to_string(),
            exit_code: 1,
        },
        Err(e) => RunResult {
            success: false,
            message: format!("bond log-to-sync failed at sync: {}", e),
            exit_code: 1,
        },
    }
}

#[cfg(feature = "http-interface")]
fn do_auth_to_web_request(token: &str, url: &str, method: &str) -> Result<u16, String> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => return Err(format!("unsupported method: {}", method)),
    };
    request = request.header("Authorization", format!("Bearer {}", token));
    let response = request.send().map_err(|e| e.to_string())?;
    let status = response.status();
    Ok(status.as_u16())
}

/// Run pipe. Args: [step1, "->", step2, ...] or joined. Split on "->", run steps (or dry-run).
fn run_pipe(args: &[String], opts: &RunOptions) -> RunResult {
    let args = strip_global_flags(args);
    let dry_run = opts.dry_run;
    if args.is_empty() {
        return RunResult {
            success: false,
            message: "pipe: usage: dal pipe <source> -> <sink> [-> <step3> ...]. Example: dal pipe oracle fetch coingecko btc -> chain estimate 1 deploy".to_string(),
            exit_code: 1,
        };
    }
    // Split on "->" token to get steps (each step = list of arg strings)
    let steps: Vec<Vec<String>> = args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .split(|&s| s == "->")
        .map(|s| s.iter().map(|&x| x.to_string()).collect())
        .filter(|s: &Vec<String>| !s.is_empty())
        .collect();
    if steps.is_empty() {
        return RunResult {
            success: false,
            message: "pipe: no steps (use '->' to separate source and sink)".to_string(),
            exit_code: 1,
        };
    }
    if dry_run {
        let steps_str: Vec<String> = steps.iter().map(|s| s.join(" ")).collect();
        return RunResult {
            success: true,
            message: format!(
                "pipe (dry-run): {} steps: {}",
                steps.len(),
                steps_str.join(" → ")
            ),
            exit_code: 0,
        };
    }
    // Execute steps: run each, pass output as input to next (first step gets no input).
    let mut input: Option<String> = None;
    let mut last_output = String::new();
    for (i, step) in steps.iter().enumerate() {
        let step_ref: Vec<&str> = step.iter().map(String::as_str).collect();
        match execute_pipe_step(&step_ref, input.as_deref()) {
            Ok(out) => {
                last_output = out.clone();
                input = Some(out);
            }
            Err(e) => {
                return RunResult {
                    success: false,
                    message: format!("pipe failed at step {} ({}): {}", i + 1, step.join(" "), e),
                    exit_code: 1,
                };
            }
        }
    }
    RunResult {
        success: true,
        message: if last_output.is_empty() {
            format!("pipe: {} step(s) completed.", steps.len())
        } else {
            last_output
        },
        exit_code: 0,
    }
}

/// Execute a single pipe step. Step = [component, subcommand, ...args]. Returns payload string for next step.
/// Supported: oracle fetch `{source} {query}`, chain estimate `{chain_id} {op}`, chain gas-price `{chain_id}`, web get `{url}`.
fn execute_pipe_step(step: &[&str], _input: Option<&str>) -> Result<String, String> {
    if step.len() < 2 {
        return Err("step must be at least [component, subcommand]".to_string());
    }
    let component = step[0];
    let subcommand = step[1];
    let args = &step[2..];

    match (component, subcommand) {
        ("oracle", "fetch") => {
            if args.len() < 2 {
                return Err("oracle fetch requires <source> <query>".to_string());
            }
            #[cfg(feature = "http-interface")]
            {
                use dist_agent_lang::stdlib::oracle::{self, OracleQuery};
                let query = OracleQuery::new(args[1].to_string()).require_signature(false);
                let response = oracle::fetch(args[0], query)?;
                Ok(format!("oracle_ok:{}", response.source))
            }
            #[cfg(not(feature = "http-interface"))]
            Err("oracle fetch requires http-interface feature".to_string())
        }
        ("chain", "estimate") => {
            if args.len() < 2 {
                return Err("chain estimate requires <chain_id> <operation>".to_string());
            }
            let chain_id: i64 = args[0].parse().map_err(|_| "invalid chain_id")?;
            let op = args[1].to_string();
            let gas = dist_agent_lang::stdlib::chain::estimate_gas(chain_id, op);
            Ok(format!("gas:{}", gas))
        }
        ("chain", "gas-price") => {
            if args.is_empty() {
                return Err("chain gas-price requires <chain_id>".to_string());
            }
            let chain_id: i64 = args[0].parse().map_err(|_| "invalid chain_id")?;
            let price = dist_agent_lang::stdlib::chain::get_gas_price(chain_id);
            Ok(format!("gas_price:{}", price))
        }
        ("web", "get") => {
            if args.is_empty() {
                return Err("web get requires <url>".to_string());
            }
            #[cfg(feature = "http-interface")]
            {
                let client = reqwest::blocking::Client::builder()
                    .build()
                    .map_err(|e| e.to_string())?;
                let resp = client.get(args[0]).send().map_err(|e| e.to_string())?;
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                let snippet = if body.len() > 200 {
                    format!("{}...", &body[..200])
                } else {
                    body
                };
                Ok(format!("status:{} body:{}", status.as_u16(), snippet))
            }
            #[cfg(not(feature = "http-interface"))]
            Err("web get requires http-interface feature".to_string())
        }
        ("web", "post") => {
            if args.is_empty() {
                return Err("web post requires <url>".to_string());
            }
            #[cfg(feature = "http-interface")]
            {
                let client = reqwest::blocking::Client::builder()
                    .build()
                    .map_err(|e| e.to_string())?;
                let body = _input.unwrap_or("{}").to_string();
                let json: serde_json::Value =
                    serde_json::from_str(&body).unwrap_or(serde_json::json!({ "data": body }));
                let resp = client
                    .post(args[0])
                    .json(&json)
                    .send()
                    .map_err(|e| e.to_string())?;
                let status = resp.status();
                let out = resp.text().unwrap_or_default();
                Ok(format!(
                    "status:{} body:{}",
                    status.as_u16(),
                    if out.len() > 200 {
                        format!("{}...", &out[..200])
                    } else {
                        out
                    }
                ))
            }
            #[cfg(not(feature = "http-interface"))]
            Err("web post requires http-interface feature".to_string())
        }
        ("db", "query") => {
            if args.len() < 2 {
                return Err("db query requires <conn_str> <sql>".to_string());
            }
            use dist_agent_lang::stdlib::database;
            let db = database::connect(args[0].to_string()).map_err(|e| e.to_string())?;
            let qr =
                database::query(&db, args[1].to_string(), vec![]).map_err(|e| e.to_string())?;
            Ok(format!("rows:{}", qr.row_count))
        }
        ("sync", "push") => {
            if args.is_empty() {
                return Err("sync push requires <url>".to_string());
            }
            use dist_agent_lang::runtime::values::Value as DalValue;
            use dist_agent_lang::stdlib::sync::{self, SyncTarget};
            let mut data = std::collections::HashMap::new();
            data.insert(
                "data".to_string(),
                DalValue::String(_input.unwrap_or("").to_string()),
            );
            let protocol = if args[0].starts_with("https://") {
                "https"
            } else {
                "http"
            };
            let target = SyncTarget::new(args[0].to_string(), protocol.to_string());
            match sync::push(data, target) {
                Ok(true) => Ok("sync_ok".to_string()),
                Ok(false) => Err("sync push failed".to_string()),
                Err(e) => Err(e),
            }
        }
        ("iot", "read-sensor") | ("iot", "read") => {
            if args.is_empty() {
                return Err("iot read-sensor requires <device_id>".to_string());
            }
            use dist_agent_lang::stdlib::iot;
            let reading = iot::read_sensor_data(args[0]).map_err(|e| e.to_string())?;
            Ok(format!(
                "sensor_ok:{} value:{:?}",
                reading.timestamp, reading.value
            ))
        }
        ("log", "info") => {
            use dist_agent_lang::runtime::values::Value as DalValue;
            use dist_agent_lang::stdlib::log;
            use std::collections::HashMap;
            let msg = _input.unwrap_or("pipe_log").to_string();
            let mut data = HashMap::new();
            data.insert("message".to_string(), DalValue::String(msg.clone()));
            log::info("pipe", data, Some("cross_component"));
            Ok(msg)
        }
        _ => Err(format!("unknown pipe step: {} {}", component, subcommand)),
    }
}

const INVOKE_WORKFLOWS: &[&str] = &[
    "price-to-deploy",
    "iot-ingest",
    "ai-audit",
    "compliance-check",
];

/// Run invoke workflow. Args: [workflow, ...rest].
fn run_invoke(args: &[String], opts: &RunOptions) -> RunResult {
    let args = strip_global_flags(args);
    let dry_run = opts.dry_run;
    if args.is_empty() {
        return RunResult {
            success: false,
            message: format!(
                "invoke: missing workflow. Workflows: {}",
                INVOKE_WORKFLOWS.join(", ")
            ),
            exit_code: 1,
        };
    }
    let workflow = &args[0];
    let rest: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    if !INVOKE_WORKFLOWS.contains(&workflow.as_str()) {
        return RunResult {
            success: false,
            message: format!(
                "invoke: unknown workflow '{}'. Workflows: {}",
                workflow,
                INVOKE_WORKFLOWS.join(", ")
            ),
            exit_code: 1,
        };
    }
    match workflow.as_str() {
        "price-to-deploy" => run_invoke_price_to_deploy(&rest, dry_run),
        "iot-ingest" => run_invoke_iot_ingest(&rest, dry_run),
        "ai-audit" => run_invoke_ai_audit(&rest, dry_run),
        "compliance-check" => run_invoke_compliance_check(&rest, dry_run),
        _ => RunResult {
            success: true,
            message: format!(
                "invoke {}: not yet implemented (stub). Args: {:?}",
                workflow, rest
            ),
            exit_code: 0,
        },
    }
}

/// invoke price-to-deploy: `{oracle_source} {chain_id} {contract}`. Oracle price → chain deploy (or estimate).
fn run_invoke_price_to_deploy(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 3 {
        return RunResult {
            success: false,
            message: "invoke price-to-deploy: usage: dal invoke price-to-deploy <oracle_source> <chain_id> <contract>".to_string(),
            exit_code: 1,
        };
    }
    let source = args[0];
    let chain_id = match args[1].parse::<i64>() {
        Ok(n) => n,
        Err(_) => {
            return RunResult {
                success: false,
                message: format!("invoke price-to-deploy: invalid chain_id '{}'", args[1]),
                exit_code: 1,
            };
        }
    };
    let contract = args[2];

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "invoke price-to-deploy (dry-run): oracle {} → chain {} → deploy/estimate {}",
                source, chain_id, contract
            ),
            exit_code: 0,
        };
    }

    #[cfg(feature = "http-interface")]
    {
        use dist_agent_lang::stdlib::chain;
        use dist_agent_lang::stdlib::oracle::{self, OracleQuery};

        let query = OracleQuery::new("price".to_string()).require_signature(false);
        match oracle::fetch(source, query) {
            Ok(_response) => {
                let gas_price = chain::get_gas_price(chain_id);
                let gas_est = chain::estimate_gas(chain_id, "deploy".to_string());
                RunResult {
                    success: true,
                    message: format!(
                        "invoke price-to-deploy: oracle → chain {} gas_price={} estimate_gas={}; contract {} (deploy not executed)",
                        chain_id, gas_price, gas_est, contract
                    ),
                    exit_code: 0,
                }
            }
            Err(e) => RunResult {
                success: false,
                message: format!("invoke price-to-deploy failed at oracle: {}", e),
                exit_code: 1,
            },
        }
    }

    #[cfg(not(feature = "http-interface"))]
    {
        let _ = (source, chain_id, contract);
        RunResult {
            success: false,
            message: "invoke price-to-deploy requires http-interface feature.".to_string(),
            exit_code: 1,
        }
    }
}

/// invoke iot-ingest: `{device_id} {conn_str}` [--window secs]. IoT read → DB → Sync.
fn run_invoke_iot_ingest(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message: "invoke iot-ingest: usage: dal invoke iot-ingest <device_id> <conn_str> [--window secs]".to_string(),
            exit_code: 1,
        };
    }
    let device_id = args[0];
    let conn_str = args[1];

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "invoke iot-ingest (dry-run): iot read {} → db {} → sync",
                device_id, conn_str
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::database;
    use dist_agent_lang::stdlib::iot;
    use dist_agent_lang::stdlib::sync::{self, SyncTarget};

    // 1) IoT read (use device_id as sensor_id for simplicity; read_sensor_data may need a registered sensor)
    let reading = match iot::read_sensor_data(device_id) {
        Ok(r) => format!("sensor_ok:{}", r.timestamp),
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("invoke iot-ingest failed at iot: {}", e),
                exit_code: 1,
            };
        }
    };
    // 2) DB connect and optional query (validate connection)
    let db = match database::connect(conn_str.to_string()) {
        Ok(d) => d,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("invoke iot-ingest failed at db connect: {}", e),
                exit_code: 1,
            };
        }
    };
    let _ = database::ping_database(&db);
    // 3) Sync push (minimal payload)
    use dist_agent_lang::runtime::values::Value as DalValue;
    let mut data = std::collections::HashMap::new();
    data.insert("iot_ingest".to_string(), DalValue::String(reading.clone()));
    let target = SyncTarget::new(conn_str.to_string(), "http".to_string());
    let sync_ok = sync::push(data, target).unwrap_or(false);

    RunResult {
        success: true,
        message: format!(
            "invoke iot-ingest: iot {} → db ok → sync {}",
            reading,
            if sync_ok { "ok" } else { "push_failed" }
        ),
        exit_code: 0,
    }
}

/// invoke ai-audit: `{conn_str}` `"{query}"`. DB query → AI analysis → log.
fn run_invoke_ai_audit(args: &[&str], dry_run: bool) -> RunResult {
    if args.len() < 2 {
        return RunResult {
            success: false,
            message: "invoke ai-audit: usage: dal invoke ai-audit <conn_str> \"<query>\""
                .to_string(),
            exit_code: 1,
        };
    }
    let conn_str = args[0];
    let query = args[1];

    if dry_run {
        return RunResult {
            success: true,
            message: format!("invoke ai-audit (dry-run): db query → ai analyze → log"),
            exit_code: 0,
        };
    }

    use dist_agent_lang::runtime::values::Value as DalValue;
    use dist_agent_lang::stdlib::ai;
    use dist_agent_lang::stdlib::database;
    use dist_agent_lang::stdlib::log;
    use std::collections::HashMap;

    let db = match database::connect(conn_str.to_string()) {
        Ok(d) => d,
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("invoke ai-audit failed at db: {}", e),
                exit_code: 1,
            };
        }
    };
    let result = match database::query(&db, query.to_string(), vec![]) {
        Ok(qr) => format!("rows:{}", qr.rows.len()),
        Err(e) => {
            return RunResult {
                success: false,
                message: format!("invoke ai-audit failed at query: {}", e),
                exit_code: 1,
            };
        }
    };
    let prompt = format!(
        "Audit summary for query result ({}): brief assessment.",
        result
    );
    let analysis = ai::generate_text(prompt).unwrap_or_else(|_| "ai_unavailable".to_string());
    let mut log_data = HashMap::new();
    log_data.insert("query".to_string(), DalValue::String(query.to_string()));
    log_data.insert("analysis".to_string(), DalValue::String(analysis.clone()));
    log::info("ai_audit", log_data, Some("invoke"));

    RunResult {
        success: true,
        message: format!(
            "invoke ai-audit: db ok → ai → log; analysis_len={}",
            analysis.len()
        ),
        exit_code: 0,
    }
}

/// invoke compliance-check: `{addr}` [--chain_id N] [--aml]. Chain balance + AML check, combined report.
fn run_invoke_compliance_check(args: &[&str], dry_run: bool) -> RunResult {
    if args.is_empty() {
        return RunResult {
            success: false,
            message: "invoke compliance-check: usage: dal invoke compliance-check <addr> [--chain_id N] [--aml]".to_string(),
            exit_code: 1,
        };
    }
    let addr = args[0];
    let chain_id: i64 = args
        .windows(2)
        .find(|w| w[0] == "--chain_id")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(1);
    let do_aml = args.contains(&"--aml") || !args.contains(&"--chain_id");

    if dry_run {
        return RunResult {
            success: true,
            message: format!(
                "invoke compliance-check (dry-run): chain {} balance {} + aml {}",
                chain_id, addr, do_aml
            ),
            exit_code: 0,
        };
    }

    use dist_agent_lang::stdlib::aml;
    use dist_agent_lang::stdlib::chain;
    use std::collections::HashMap;

    let balance = chain::get_balance(chain_id, addr.to_string());
    let mut report = format!(
        "compliance-check: addr={} chain_id={} balance={}",
        addr, chain_id, balance
    );

    if do_aml {
        let mut user_data = HashMap::new();
        user_data.insert("address".to_string(), addr.to_string());
        let aml_result = aml::perform_check(
            "default".to_string(),
            addr.to_string(),
            "sanctions".to_string(),
            user_data,
        );
        let status = aml_result
            .get("status")
            .and_then(|v| match v {
                dist_agent_lang::runtime::values::Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("unknown");
        report.push_str(&format!(" aml_status={}", status));
    }

    RunResult {
        success: true,
        message: report,
        exit_code: 0,
    }
}

/// Entry: run cross-component command and return result for CLI to print and exit.
pub fn run(cmd: &str, args: &[String], opts: &RunOptions) -> RunResult {
    let result = match cmd {
        "bond" => run_bond(args, opts),
        "pipe" => run_pipe(args, opts),
        "invoke" => run_invoke(args, opts),
        _ => RunResult {
            success: false,
            message: format!("cross-component: unknown command '{}'", cmd),
            exit_code: 1,
        },
    };
    if opts.format_json {
        // Machine-readable output for scripting
        let json = serde_json::json!({
            "success": result.success,
            "message": result.message,
            "exit_code": result.exit_code,
        });
        RunResult {
            success: result.success,
            message: serde_json::to_string(&json).unwrap_or(result.message),
            exit_code: result.exit_code,
        }
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_global_flags() {
        let args: Vec<String> = vec![
            "auth-to-web".into(),
            "token".into(),
            "https://example.com".into(),
            "--format".into(),
            "json".into(),
            "--dry-run".into(),
        ];
        let out = strip_global_flags(&args);
        assert_eq!(out, ["auth-to-web", "token", "https://example.com"]);
    }

    #[test]
    fn test_strip_format_before_url() {
        let args: Vec<String> = vec![
            "auth-to-web".into(),
            "token".into(),
            "--format".into(),
            "json".into(),
            "https://example.com".into(),
            "--dry-run".into(),
        ];
        let out = strip_global_flags(&args);
        assert_eq!(out, ["auth-to-web", "token", "https://example.com"]);
    }
}
