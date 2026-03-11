//! IDE backend: orchestration API, run backend, agent API, and LSP diagnostics.
//! See docs/development/IDE design/.

pub mod diagnostics;
pub mod lsp_bridge;
pub mod lsp_client;
pub mod orchestration;
pub mod run_backend;
pub mod server;
pub mod symbols;
