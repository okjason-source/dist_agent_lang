//! CLI design: branding, help layout, and AI-ready structure.
//!
//! Organized by use case for intuitive discovery across project types.
//! AI-first: "AI & CODE ASSISTANCE" and "AGENTS & AUTOMATION" appear early.
//! Future: `dal ask "<prompt>"` or `dal "<natural language>"` can be added
//! as primary entry points as AI integration evolves.

/// Brand tagline — used in banner and help (short form for banner)
pub const TAGLINE: &str = "Web, Blockchain & AI";

/// Full tagline for version output
pub const TAGLINE_FULL: &str = "Unified language for Web, Blockchain & AI";

/// Version string from Cargo
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Print branded banner (respects --no-banner, --quiet)
pub fn print_banner(bin: &str, no_banner: bool, quiet: bool) {
    if no_banner {
        return;
    }
    if quiet {
        println!("{} v{} — {}", bin, version(), TAGLINE);
        return;
    }
    println!();
    println!("┌──────────────────────────────────────────────────────────────────────┐");
    println!("│  {} v{}  —  {}  │", bin, version(), TAGLINE_FULL);
    println!("└──────────────────────────────────────────────────────────────────────┘");
    println!();
}

/// Help content organized by use case (project-type agnostic, AI-forward)
pub fn help_content(bin: &str) -> String {
    format!(
        r#"
GET STARTED
  new <name> [--type <type>]  Create a new project
                              Types: ai, iot, agent, chain, web, cli, lib
  init                       Initialize DAL in current directory
  run <file.dal>             Run a DAL file
  test [file.test.dal]       Run tests

BUILD & DEVELOP
  check <file.dal>            Type check without executing
  fmt <file.dal> [--check]    Format code (--check for CI)
  lint <file.dal>             Lint for issues
  watch <file.dal>            Re-run on changes
  repl                        Interactive REPL
  bench [file] [--suite name]  Run benchmarks
  profile <file> [--memory]   Profile execution
  optimize <file> [-o out]    Apply AST optimizations

AI & CODE ASSISTANCE
  ai code "<prompt>"          Generate DAL from natural language
  ai code "<p>" -o <file>     Generate and save to file
  ai explain <file>           Explain what code does
  ai review <file>            Code review with suggestions
  ai audit <file>             Security audit for contracts
  ai test <file>              Generate test cases
  ai fix <file>               Suggest fixes for issues
  ai optimize-gas <file>      Gas optimization suggestions

AGENTS & AUTOMATION
  agent create <type> <name>  Create AI/system/worker agent
  agent send <from> <to> <msg>  Send message between agents
  agent list                  List agents
  agent mold                  Molds and reusable configs

BLOCKCHAIN
  chain list                  List supported chains
  chain gas-price <id>        Get gas price
  chain balance <id> <addr>   Get address balance
  chain mint <name>           Mint asset
  chain asset <id>            Get asset info

DATA & INFRASTRUCTURE
  crypto hash <data> [alg]    Hash (sha256/sha512)
  crypto keygen [alg]         Generate keypair
  db connect <conn>           Test database connection
  db query <conn> "<sql>"     Execute query
  cloud authorize | grant     Cloud RBAC and audit

DEVICES & IoT
  iot register <name>         Register device
  iot status <device_id>      Device status
  iot ai-predict <device>     Predictive maintenance

TOOLS
  web <file.dal>              Run web app
  web get <url>               HTTP GET
  convert <input.sol> [-o out] Solidity to DAL
  doc <file.dal> [--open]     Generate documentation
  completions [bash|zsh|fish]  Shell completions

INFO
  help                        This message
  version                     Version information

QUICK EXAMPLES
  {0} new my-token --type chain
  {0} ai code "Create an ERC20 token with 1M supply"
  {0} chain gas-price 1
  {0} run app.dal

For subcommand help: {0} <command> --help
"#,
        bin
    )
}
