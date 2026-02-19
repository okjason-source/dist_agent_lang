//! CLI framework using clap (Phase 11: 09_CLI_EXPANSION_PLAN.md).
//! Provides structured parsing, help layout, and §21 design hooks.

use clap::{Parser, Subcommand, ValueEnum};

/// dist_agent_lang (dal) — Unified language for Web, Blockchain & AI
#[derive(Parser, Debug)]
#[command(
    name = "dal",
    about = "Unified language for Web, Blockchain & AI",
    version,
    long_about = None,
    term_width = 80,
    disable_help_flag = true,
    disable_version_flag = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Minimal output (errors only)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable banner in help/version
    #[arg(long, global = true)]
    pub no_banner: bool,

    /// Color output: always, never, auto
    #[arg(long, default_value = "auto", value_name = "WHEN", global = true)]
    pub color: ColorChoice,

    /// Print help information
    #[arg(short = 'h', long = "help", global = true)]
    pub help: bool,

    /// Print version information
    #[arg(short = 'V', long = "version", global = true)]
    pub version_flag: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Always,
    Never,
    #[default]
    Auto,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a DAL file
    Run { file: String },

    /// Run tests (discovers *.test.dal files)
    Test {
        #[arg(required = false)]
        file: Option<String>,
    },

    /// Run internal system health checks
    Selftest,

    /// Web: run DAL/JS file (web <file.dal> | web <file.js> [args...]) or HTTP get/post/parse-url
    Web {
        #[arg(required = true)]
        sub: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Serve a DAL file with HTTP handlers (e.g. todo backend) on a port
    Serve {
        /// DAL file with handler functions (e.g. get_todos, create_todo)
        file: String,
        /// Port to listen on (default: 8080)
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// HTML file to serve at / (e.g. examples/frontend_todo_app.html)
        #[arg(long)]
        frontend: Option<String>,
        /// Allowed CORS origin (default: * for all origins). Use a specific URL for production.
        #[arg(long, default_value = "*")]
        cors_origin: String,
    },

    /// Convert Solidity to DAL
    Convert {
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Analyze Solidity contract
    Analyze { input: String },

    /// Parse and validate syntax
    Parse { file: String },

    /// Type check without running
    Check { file: String },

    /// Format DAL code
    Fmt {
        file: String,
        #[arg(long)]
        check: bool,
    },

    /// Lint DAL code
    Lint { file: String },

    /// Create new project
    New {
        name: String,
        #[arg(long)]
        project_type: Option<String>,
    },

    /// Initialize project in current directory
    Init,

    /// Interactive REPL
    Repl,

    /// Watch file and re-run on change
    Watch { file: String },

    /// Add package dependency
    Add { package: String },

    /// Install dependencies
    Install,

    /// Run benchmarks
    Bench {
        #[arg(required = false)]
        file: Option<String>,
        #[arg(long)]
        suite: Option<String>,
    },

    /// Profile execution
    Profile {
        file: String,
        #[arg(long)]
        memory: bool,
    },

    /// Apply AST optimizations
    Optimize {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long, default_value = "1")]
        level: u8,
    },

    /// Show memory stats
    MemoryStats,

    /// Blockchain operations
    Chain {
        #[command(subcommand)]
        subcommand: ChainSubcommand,
    },

    /// Cryptographic utilities
    Crypto {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Database operations
    Db {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// AI / ML operations
    Ai {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Cloud & enterprise commands
    Cloud {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Oracle operations (fetch, verify, stream external data)
    Oracle {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Start LSP server
    Lsp {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Generate documentation
    Doc {
        #[arg(required = true)]
        target: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        open: bool,
    },

    /// Generate shell completions
    Completions { shell: String },

    /// Debug a DAL file
    Debug {
        file: String,
        #[arg(long)]
        breakpoint: Option<u32>,
    },

    /// Agent lifecycle & coordination
    Agent {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// IoT device operations
    Iot {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Log operations
    Log {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Config operations
    Config {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Admin operations
    Admin {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Key / capability operations
    Key {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// AML operations
    Aml {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// KYC operations
    Kyc {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Mold operations (alias for agent mold)
    Mold {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Generate boilerplate
    Scaffold {
        scaffold_type: String,
        #[arg(required = false)]
        name: Option<String>,
    },

    /// Build project
    Build {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Remove build artifacts
    Clean {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Create distribution package
    Dist {
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },

    /// Connect components (bond, pipe, invoke)
    Bond {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },
    Pipe {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },
    Invoke {
        #[arg(required = true)]
        subcommand: String,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    },
}

/// Chain subcommands — fully defined for Phase 11 migration
#[derive(Subcommand, Debug)]
pub enum ChainSubcommand {
    /// List supported chains
    List,

    /// Show chain configuration
    Config { chain_id: i64 },

    /// Get current gas price
    GasPrice { chain_id: i64 },

    /// Get address balance
    Balance { chain_id: i64, address: String },

    /// Get transaction status
    TxStatus { chain_id: i64, tx_hash: String },

    /// Get latest block timestamp
    BlockTime { chain_id: i64 },

    /// Deploy contract
    Deploy {
        chain_id: i64,
        contract: String,
        #[arg(long)]
        args: Option<String>,
    },

    /// Call contract function
    Call {
        chain_id: i64,
        address: String,
        function: String,
        #[arg(long)]
        args: Option<String>,
    },

    /// Mint asset (local/sim)
    Mint {
        name: String,
        #[arg(long)]
        meta: Option<String>,
    },

    /// Get asset info
    Asset { id: String },
}

impl Cli {
    /// Parse args; handles --help and --version via clap
    pub fn parse() -> Self {
        Self::parse_from(std::env::args())
    }
}

/// Convert chain subcommand to args slice for existing handlers
pub fn chain_subcommand_to_args(sub: &ChainSubcommand) -> Vec<String> {
    use ChainSubcommand::*;
    match sub {
        List => vec!["list".to_string()],
        Config { chain_id } => vec!["config".to_string(), chain_id.to_string()],
        GasPrice { chain_id } => vec!["gas-price".to_string(), chain_id.to_string()],
        Balance { chain_id, address } => {
            vec!["balance".to_string(), chain_id.to_string(), address.clone()]
        }
        TxStatus { chain_id, tx_hash } => vec![
            "tx-status".to_string(),
            chain_id.to_string(),
            tx_hash.clone(),
        ],
        BlockTime { chain_id } => vec!["block-time".to_string(), chain_id.to_string()],
        Deploy {
            chain_id,
            contract,
            args,
        } => {
            let mut v = vec!["deploy".to_string(), chain_id.to_string(), contract.clone()];
            if let Some(a) = args {
                v.push("--args".to_string());
                v.push(a.clone());
            }
            v
        }
        Call {
            chain_id,
            address,
            function,
            args,
        } => {
            let mut v = vec![
                "call".to_string(),
                chain_id.to_string(),
                address.clone(),
                function.clone(),
            ];
            if let Some(a) = args {
                v.push("--args".to_string());
                v.push(a.clone());
            }
            v
        }
        Mint { name, meta } => {
            let mut v = vec!["mint".to_string(), name.clone()];
            if let Some(m) = meta {
                v.push("--meta".to_string());
                v.push(m.clone());
            }
            v
        }
        Asset { id } => vec!["asset".to_string(), id.clone()],
    }
}
