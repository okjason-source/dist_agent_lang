use crate::http_server_security::{security_headers_middleware, RateLimiter};
use crate::http_server_security_middleware::{
    input_validation_middleware, rate_limit_middleware, request_size_middleware,
};
use crate::runtime::engine::Runtime;
use crate::stdlib::web::HttpServer;
use axum::routing::Router;
use axum::{
    http::Method,
    middleware,
    response::{Html, Json},
    Extension,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub query_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct WebServerState {
    pub server: HttpServer,
    pub handlers: HashMap<String, Box<dyn Fn(WebRequest) -> WebResponse + Send + Sync>>,
    // Arc-wrapped so we can clone for spawn_blocking (keeps async runtime responsive)
    pub runtime_factory: Option<Arc<Box<dyn Fn() -> Runtime + Send + Sync>>>,
    /// Optional callback to persist scope changes after handler execution (shared state).
    pub scope_writeback: Option<Arc<Box<dyn Fn(&crate::runtime::scope::Scope) + Send + Sync>>>,
}

pub async fn start_http_server(server: HttpServer) -> Result<(), Box<dyn std::error::Error>> {
    // Runtime factory for creating new runtime instances per request
    let runtime_factory =
        Arc::new(Box::new(|| Runtime::new()) as Box<dyn Fn() -> Runtime + Send + Sync>);

    let state = Arc::new(RwLock::new(WebServerState {
        server: server.clone(),
        handlers: HashMap::new(),
        runtime_factory: Some(runtime_factory),
        scope_writeback: None,
    }));

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    // Create router from server configuration (uses routes from server.routes)
    // Security middleware order (from outer to inner):
    // 1. CORS
    // 2. Security headers (applied to all responses)
    // 3. Rate limiting
    // 4. Request size limiting
    // 5. Input validation
    // 6. User middleware (applied in handle_with_middleware)

    // Create router with routes from server configuration
    // Note: We need to add layers before with_state() to maintain correct types
    // So we'll build the router manually here to have control over layer ordering
    let mut router = Router::new();

    // Add routes from server configuration (same logic as create_router_with_middleware)
    for (route_key, route) in &server.routes {
        if let Some((method_str, path)) = route_key.split_once(':') {
            let method = method_str.to_uppercase();
            let handler_name = route.handler.clone();
            let state_clone = state.clone();

            let handler = move |request: axum::extract::Request| {
                let state = state_clone.clone();
                let handler_name = handler_name.clone();
                async move {
                    crate::http_server_handlers::handle_with_middleware(
                        axum::extract::State(state),
                        request,
                        &handler_name,
                    )
                    .await
                }
            };

            match method.as_str() {
                "GET" => router = router.route(path, axum::routing::get(handler)),
                "POST" => router = router.route(path, axum::routing::post(handler)),
                "PUT" => router = router.route(path, axum::routing::put(handler)),
                "DELETE" => router = router.route(path, axum::routing::delete(handler)),
                _ => router = router.route(path, axum::routing::get(handler)),
            }
        }
    }

    // Add default routes if no routes configured
    if server.routes.is_empty() {
        router = router
            .route("/", axum::routing::get(home_handler))
            .route("/health", axum::routing::get(health_handler));
    }

    // Shared rate limiter so all requests are counted (default 100 req/min).
    // Without this, the middleware would create a new limiter per request and rate limiting would never trigger.
    let rate_limiter = Arc::new(RateLimiter::new(100, 60));

    // Add security middleware layers, then state.
    // Extension(rate_limiter) is added last so it runs first on the request and rate_limit_middleware sees it in extensions.
    let app = router
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(middleware::from_fn(request_size_middleware))
        .layer(middleware::from_fn(input_validation_middleware))
        .layer(cors)
        .layer(Extension(rate_limiter))
        .with_state(state);

    // Start server
    let addr = format!("127.0.0.1:{}", server.port);
    println!("🚀 Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn home_handler() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>KEYS Web Application - dist_agent_lang</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: #333;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }

        .header {
            text-align: center;
            margin-bottom: 40px;
            color: white;
        }

        .header h1 {
            font-size: 3rem;
            margin-bottom: 10px;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
        }

        .header p {
            font-size: 1.2rem;
            opacity: 0.9;
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .card {
            background: white;
            border-radius: 15px;
            padding: 25px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.2);
            transition: transform 0.3s ease;
        }

        .card:hover {
            transform: translateY(-5px);
        }

        .card h3 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 1.5rem;
        }

        .form-group {
            margin-bottom: 15px;
        }

        .form-group label {
            display: block;
            margin-bottom: 5px;
            font-weight: 600;
            color: #555;
        }

        .form-group input {
            width: 100%;
            padding: 12px;
            border: 2px solid #e1e5e9;
            border-radius: 8px;
            font-size: 16px;
            transition: border-color 0.3s ease;
        }

        .form-group input:focus {
            outline: none;
            border-color: #667eea;
        }

        .btn {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 16px;
            font-weight: 600;
            transition: transform 0.2s ease;
            width: 100%;
        }

        .btn:hover {
            transform: translateY(-2px);
        }

        .btn:active {
            transform: translateY(0);
        }

        .status {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 15px;
            margin-top: 15px;
            border-left: 4px solid #667eea;
        }

        .status h4 {
            color: #667eea;
            margin-bottom: 10px;
        }

        .status p {
            color: #666;
            line-height: 1.5;
        }

        .footer {
            text-align: center;
            color: white;
            margin-top: 40px;
            opacity: 0.8;
        }

        .language-badge {
            display: inline-block;
            background: rgba(255,255,255,0.2);
            padding: 5px 15px;
            border-radius: 20px;
            font-size: 0.9rem;
            margin-top: 10px;
        }

        .success {
            background: #d4edda;
            border-color: #28a745;
            color: #155724;
        }

        .error {
            background: #f8d7da;
            border-color: #dc3545;
            color: #721c24;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔑 KEYS Web Application</h1>
            <p>Built with dist_agent_lang - Unified Language for Web & Blockchain</p>
            <div class="language-badge">dist_agent_lang v1.0.2</div>
        </div>

        <div class="grid">
            <!-- Wallet Connection -->
            <div class="card">
                <h3>🔗 Connect Wallet</h3>
                <p>Connect your MetaMask or other Web3 wallet to interact with the KEYS token.</p>
                <button class="btn" onclick="connectWallet()">Connect Wallet</button>
                <div id="wallet-status" class="status" style="display: none;">
                    <h4>Wallet Status</h4>
                    <p id="wallet-info">Not connected</p>
                </div>
            </div>

            <!-- Balance Check -->
            <div class="card">
                <h3>💰 Check Balance</h3>
                <p>View your current KEYS token balance and account information.</p>
                <button class="btn" onclick="checkBalance()">Check Balance</button>
                <div id="balance-status" class="status" style="display: none;">
                    <h4>Balance Information</h4>
                    <p id="balance-info">Loading...</p>
                </div>
            </div>

            <!-- Token Transfer -->
            <div class="card">
                <h3>📤 Transfer Tokens</h3>
                <p>Send KEYS tokens to another address.</p>
                <div class="form-group">
                    <label for="recipient">Recipient Address:</label>
                    <input type="text" id="recipient" placeholder="0x...">
                </div>
                <div class="form-group">
                    <label for="amount">Amount:</label>
                    <input type="number" id="amount" placeholder="100" step="0.01">
                </div>
                <button class="btn" onclick="transferTokens()">Transfer</button>
                <div id="transfer-status" class="status" style="display: none;">
                    <h4>Transfer Status</h4>
                    <p id="transfer-info">Processing...</p>
                </div>
            </div>

            <!-- Airdrop -->
            <div class="card">
                <h3>🎁 Claim Airdrop</h3>
                <p>Claim your free KEYS tokens from the airdrop program.</p>
                <button class="btn" onclick="claimAirdrop()">Claim Airdrop</button>
                <div id="airdrop-status" class="status" style="display: none;">
                    <h4>Airdrop Status</h4>
                    <p id="airdrop-info">Processing claim...</p>
                </div>
            </div>
        </div>

        <div class="footer">
            <p>🚀 Powered by dist_agent_lang - The Unified Programming Language</p>
            <p>Web + Blockchain + AI in One Language</p>
        </div>
    </div>

    <script>
        // API Functions
        async function connectWallet() {
            try {
                const response = await fetch('/api/connect', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' }
                });
                const data = await response.json();
                
                const status = document.getElementById('wallet-status');
                const info = document.getElementById('wallet-info');
                
                status.style.display = 'block';
                status.className = 'status success';
                info.textContent = \`Connected: \${data.address}\`;
            } catch (error) {
                showError('wallet-status', 'Failed to connect wallet');
            }
        }

        async function checkBalance() {
            try {
                const response = await fetch('/api/balance');
                const data = await response.json();
                
                const status = document.getElementById('balance-status');
                const info = document.getElementById('balance-info');
                
                status.style.display = 'block';
                status.className = 'status success';
                info.innerHTML = \`
                    <strong>Balance:</strong> \${data.balance} KEYS<br>
                    <strong>Address:</strong> \${data.address}<br>
                    <strong>Status:</strong> \${data.message}
                \`;
            } catch (error) {
                showError('balance-status', 'Failed to check balance');
            }
        }

        async function transferTokens() {
            const recipient = document.getElementById('recipient').value;
            const amount = document.getElementById('amount').value;
            
            if (!recipient || !amount) {
                showError('transfer-status', 'Please fill in all fields');
                return;
            }

            try {
                const response = await fetch('/api/transfer', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ to: recipient, amount: amount })
                });
                const data = await response.json();
                
                const status = document.getElementById('transfer-status');
                const info = document.getElementById('transfer-info');
                
                status.style.display = 'block';
                status.className = 'status success';
                info.innerHTML = \`
                    <strong>Success:</strong> \${data.message}<br>
                    <strong>Transaction:</strong> \${data.transaction}
                \`;
            } catch (error) {
                showError('transfer-status', 'Transfer failed');
            }
        }

        async function claimAirdrop() {
            try {
                const response = await fetch('/api/airdrop', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' }
                });
                const data = await response.json();
                
                const status = document.getElementById('airdrop-status');
                const info = document.getElementById('airdrop-info');
                
                status.style.display = 'block';
                status.className = 'status success';
                info.innerHTML = \`
                    <strong>Success:</strong> \${data.message}<br>
                    <strong>Amount:</strong> \${data.amount} KEYS<br>
                    <strong>Transaction:</strong> \${data.transaction}
                \`;
            } catch (error) {
                showError('airdrop-status', 'Airdrop claim failed');
            }
        }

        function showError(elementId, message) {
            const status = document.getElementById(elementId);
            const info = status.querySelector('p');
            
            status.style.display = 'block';
            status.className = 'status error';
            info.textContent = message;
        }

        // Auto-check balance on page load
        window.addEventListener('load', () => {
            setTimeout(checkBalance, 1000);
        });
    </script>
</body>
</html>
    "#;

    Html(html.to_string())
}

#[allow(dead_code)]
async fn balance_handler() -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "success": true,
        "balance": "1000.0",
        "address": "0x1234567890abcdef",
        "message": "Balance retrieved successfully"
    });

    Json(response)
}

#[allow(dead_code)]
async fn connect_handler() -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "success": true,
        "message": "Wallet connected successfully",
        "address": "0x1234567890abcdef"
    });

    Json(response)
}

#[allow(dead_code)]
async fn transfer_handler() -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "success": true,
        "message": "Transfer completed successfully",
        "transaction": "0xabcdef1234567890"
    });

    Json(response)
}

#[allow(dead_code)]
async fn airdrop_handler() -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "success": true,
        "message": "Airdrop claimed successfully",
        "amount": "100.0",
        "transaction": "0x1234567890abcdef"
    });

    Json(response)
}

async fn health_handler() -> Json<serde_json::Value> {
    let response = serde_json::json!({
        "status": "healthy",
        "service": "KEYS Web Application",
        "language": "dist_agent_lang",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Json(response)
}
