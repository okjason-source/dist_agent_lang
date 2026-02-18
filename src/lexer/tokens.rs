use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number(String),
    String(String),
    Boolean(bool),
    Identifier(String),
    
    // Keywords
    Let,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Try,
    Catch,
    Throw,
    Spawn,
    Agent,
    Message,
    Event,
    Service,
    Impl,
    Struct,
    Enum,
    Type,
    Namespace,
    Use,
    Pub,
    Mod,
    Trait,
    Where,
    Match,
    Case,
    Default,
    Break,
    Continue,
    Loop,
    Await,
    Async,
    
    // New keywords for enhanced features
    Result,
    Option,
    Some,
    None,
    Ok,
    Err,
    List,
    Map,
    Set,
    Generic,
    Box,
    Ref,
    Mut,
    Const,
    Static,
    Extern,
    Crate,
    Super,
    Self_,
    SelfType,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LeftShift,
    RightShift,
    
    // Assignment operators
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    AndEqual,
    OrEqual,
    CaretEqual,
    LeftShiftEqual,
    RightShiftEqual,
    
    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Arrow,
    FatArrow,
    At,
    Hash,
    Dollar,
    Question,
    Exclamation,
    
    // Attributes
    Trust,
    Secure,
    Limit,
    // Web, // Treat as identifier for namespace calls
    Mobile,
    Desktop,
    Iot,
    Ai,
    Persistent,
    Cached,
    Versioned,
    Deprecated,
    CompileTarget, // NEW: Compilation target attribute
    Chain, // NEW: Chain attribute
    Interface, // NEW: Interface attribute
    
    // Special
    EOF,
    Error(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::Number(n) => write!(f, "Number({})", n),
            TokenType::String(s) => write!(f, "String({})", s),
            TokenType::Boolean(b) => write!(f, "Boolean({})", b),
            TokenType::Identifier(id) => write!(f, "Identifier({})", id),
            TokenType::Let => write!(f, "let"),
            TokenType::Fn => write!(f, "fn"),
            TokenType::Return => write!(f, "return"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::While => write!(f, "while"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Try => write!(f, "try"),
            TokenType::Catch => write!(f, "catch"),
            TokenType::Throw => write!(f, "throw"),
            TokenType::Spawn => write!(f, "spawn"),
            TokenType::Agent => write!(f, "agent"),
            TokenType::Message => write!(f, "msg"),
            TokenType::Event => write!(f, "event"),
            TokenType::Service => write!(f, "service"),
            TokenType::Impl => write!(f, "impl"),
            TokenType::Struct => write!(f, "struct"),
            TokenType::Enum => write!(f, "enum"),
            TokenType::Type => write!(f, "type"),
            TokenType::Namespace => write!(f, "namespace"),
            TokenType::Use => write!(f, "use"),
            TokenType::Pub => write!(f, "pub"),
            TokenType::Mod => write!(f, "mod"),
            TokenType::Trait => write!(f, "trait"),
            TokenType::Where => write!(f, "where"),
            TokenType::Match => write!(f, "match"),
            TokenType::Case => write!(f, "case"),
            TokenType::Default => write!(f, "default"),
            TokenType::Break => write!(f, "break"),
            TokenType::Continue => write!(f, "continue"),
            TokenType::Loop => write!(f, "loop"),
            TokenType::Await => write!(f, "await"),
            TokenType::Async => write!(f, "async"),
            
            // New keywords
            TokenType::Result => write!(f, "Result"),
            TokenType::Option => write!(f, "Option"),
            TokenType::Some => write!(f, "Some"),
            TokenType::None => write!(f, "None"),
            TokenType::Ok => write!(f, "Ok"),
            TokenType::Err => write!(f, "Err"),
            TokenType::List => write!(f, "List"),
            TokenType::Map => write!(f, "Map"),
            TokenType::Set => write!(f, "Set"),
            TokenType::Generic => write!(f, "Generic"),
            TokenType::Box => write!(f, "Box"),
            TokenType::Ref => write!(f, "ref"),
            TokenType::Mut => write!(f, "mut"),
            TokenType::Const => write!(f, "const"),
            TokenType::Static => write!(f, "static"),
            TokenType::Extern => write!(f, "extern"),
            TokenType::Crate => write!(f, "crate"),
            TokenType::Super => write!(f, "super"),
            TokenType::Self_ => write!(f, "self"),
            TokenType::SelfType => write!(f, "Self"),
            
            // Operators
            TokenType::Plus => write!(f, "+"),
            TokenType::Minus => write!(f, "-"),
            TokenType::Star => write!(f, "*"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::And => write!(f, "&&"),
            TokenType::Or => write!(f, "||"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Pipe => write!(f, "|"),
            TokenType::Caret => write!(f, "^"),
            TokenType::Tilde => write!(f, "~"),
            TokenType::LeftShift => write!(f, "<<"),
            TokenType::RightShift => write!(f, ">>"),
            
            // Assignment operators
            TokenType::PlusEqual => write!(f, "+="),
            TokenType::MinusEqual => write!(f, "-="),
            TokenType::StarEqual => write!(f, "*="),
            TokenType::SlashEqual => write!(f, "/="),
            TokenType::PercentEqual => write!(f, "%="),
            TokenType::AndEqual => write!(f, "&="),
            TokenType::OrEqual => write!(f, "|="),
            TokenType::CaretEqual => write!(f, "^="),
            TokenType::LeftShiftEqual => write!(f, "<<="),
            TokenType::RightShiftEqual => write!(f, ">>="),
            
            // Punctuation
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Arrow => write!(f, "->"),
            TokenType::FatArrow => write!(f, "=>"),
            TokenType::At => write!(f, "@"),
            TokenType::Hash => write!(f, "#"),
            TokenType::Dollar => write!(f, "$"),
            TokenType::Question => write!(f, "?"),
            TokenType::Exclamation => write!(f, "!"),
            
            // Attributes
            TokenType::Trust => write!(f, "@trust"),
            TokenType::Secure => write!(f, "@secure"),
            TokenType::Limit => write!(f, "@limit"),
            // TokenType::Web => write!(f, "@web"),
            TokenType::Mobile => write!(f, "@mobile"),
            TokenType::Desktop => write!(f, "@desktop"),
            TokenType::Iot => write!(f, "@iot"),
            TokenType::Ai => write!(f, "@ai"),
            TokenType::Persistent => write!(f, "@persistent"),
            TokenType::Cached => write!(f, "@cached"),
            TokenType::Versioned => write!(f, "@versioned"),
            TokenType::Deprecated => write!(f, "@deprecated"),
            TokenType::CompileTarget => write!(f, "@compile_target"),
            TokenType::Chain => write!(f, "@chain"),
            TokenType::Interface => write!(f, "@interface"),
            
            // Special
            TokenType::EOF => write!(f, "EOF"),
            TokenType::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

// Legacy compatibility enums for the parser
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Try,
    Catch,
    Throw,
    Spawn,
    Agent,
    Msg,
    Event,
    Service,
    Impl,
    Struct,
    Enum,
    Type,
    Namespace,
    Use,
    Pub,
    Mod,
    Trait,
    Where,
    Match,
    Case,
    Default,
    Break,
    Continue,
    Loop,
    Await,
    Async,
    Result,
    Option,
    Some,
    None,
    Ok,
    Err,
    List,
    Map,
    Set,
    Generic,
    Box,
    Ref,
    Mut,
    Const,
    Static,
    Extern,
    Crate,
    Super,
    Self_,
    SelfType,
    With,
    Finally,
    Txn,
    Secure,
    Limit,
    Trust,
    Import,
    Export,
    As,
    Private,
    Audit,
    CompileTarget, // NEW: Compilation target keyword
    Chain, // NEW: Chain keyword
    Interface, // NEW: Interface keyword
    // Web, // Treat as identifier for namespace calls
    Mobile,
    Desktop,
    Iot,
    Ai,
    Persistent,
    Cached,
    Versioned,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    LeftShift,
    RightShift,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    AndEqual,
    OrEqual,
    CaretEqual,
    LeftShiftEqual,
    RightShiftEqual,
    Assign,
    Not,
    Colon,
    NotEqual,  // != operator
    Dot,       // . operator
}

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuation {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    DotDot,  // Range operator (..)
    Arrow,
    FatArrow,
    At,
    Hash,
    Dollar,
    Question,
    Exclamation,
    DoubleColon,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

// Token enum variants for compatibility with parser
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keyword variants
    Keyword(Keyword),
    
    // Operator variants  
    Operator(Operator),
    
    // Punctuation variants
    Punctuation(Punctuation),
    
    // Literal variants
    Literal(Literal),
    
    // Identifier variant
    Identifier(String),
    
    // Special token
    EOF,
}

// Token with position information for accurate error reporting
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

impl TokenWithPosition {
    pub fn new(token: Token, line: usize, column: usize) -> Self {
        Self { token, line, column }
    }
}

// Keep the old struct as TokenStruct for internal use
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStruct {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl TokenStruct {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        TokenStruct {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for TokenStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}, column {}", self.token_type, self.line, self.column)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Keyword(k) => write!(f, "Keyword({:?})", k),
            Token::Operator(o) => write!(f, "Operator({:?})", o),
            Token::Punctuation(p) => write!(f, "Punctuation({:?})", p),
            Token::Literal(l) => write!(f, "Literal({:?})", l),
            Token::Identifier(i) => write!(f, "Identifier({})", i),
            Token::EOF => write!(f, "EOF"),
        }
    }
}

// NEW: Compilation Target System
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompilationTarget {
    Blockchain,
    WebAssembly,
    Native,
    Mobile,
    Edge,
}

impl CompilationTarget {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "blockchain" => Some(CompilationTarget::Blockchain),
            "wasm" | "webassembly" => Some(CompilationTarget::WebAssembly),
            "native" => Some(CompilationTarget::Native),
            "mobile" => Some(CompilationTarget::Mobile),
            "edge" => Some(CompilationTarget::Edge),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            CompilationTarget::Blockchain => "blockchain".to_string(),
            CompilationTarget::WebAssembly => "wasm".to_string(),
            CompilationTarget::Native => "native".to_string(),
            CompilationTarget::Mobile => "mobile".to_string(),
            CompilationTarget::Edge => "edge".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TargetConstraint {
    pub target: CompilationTarget,
    pub allowed_operations: Vec<String>,
    pub forbidden_operations: Vec<String>,
    pub required_attributes: Vec<String>,
    pub trust_profiles: HashMap<TrustLevel, SecurityProfile>, // NEW: Trust-aware constraints
}

impl TargetConstraint {
    pub fn new(target: CompilationTarget) -> Self {
        Self {
            target,
            allowed_operations: Vec::new(),
            forbidden_operations: Vec::new(),
            required_attributes: Vec::new(),
            trust_profiles: HashMap::new(),
        }
    }
    
    pub fn with_allowed_operations(mut self, operations: Vec<String>) -> Self {
        self.allowed_operations = operations;
        self
    }
    
    pub fn with_forbidden_operations(mut self, operations: Vec<String>) -> Self {
        self.forbidden_operations = operations;
        self
    }
    
    pub fn with_required_attributes(mut self, attributes: Vec<String>) -> Self {
        self.required_attributes = attributes;
        self
    }
    
    pub fn with_trust_profiles(mut self, profiles: HashMap<TrustLevel, SecurityProfile>) -> Self {
        self.trust_profiles = profiles;
        self
    }
}

// NEW: Trust Model System
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrustLevel {
    Decentralized,  // Pure blockchain, no external deps
    Hybrid,         // Mixed blockchain + external services
    Centralized,    // Traditional centralized services
}

impl TrustLevel {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "decentralized" => Some(TrustLevel::Decentralized),
            "hybrid" => Some(TrustLevel::Hybrid),
            "centralized" => Some(TrustLevel::Centralized),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            TrustLevel::Decentralized => "decentralized".to_string(),
            TrustLevel::Hybrid => "hybrid".to_string(),
            TrustLevel::Centralized => "centralized".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityProfile {
    pub trust_level: TrustLevel,
    pub allowed_external_apis: Vec<String>,
    pub forbidden_external_apis: Vec<String>,
    pub required_security_checks: Vec<String>,
    pub audit_requirements: Vec<String>,
}

impl SecurityProfile {
    pub fn new(trust_level: TrustLevel) -> Self {
        Self {
            trust_level,
            allowed_external_apis: Vec::new(),
            forbidden_external_apis: Vec::new(),
            required_security_checks: Vec::new(),
            audit_requirements: Vec::new(),
        }
    }
    
    pub fn with_allowed_apis(mut self, apis: Vec<String>) -> Self {
        self.allowed_external_apis = apis;
        self
    }
    
    pub fn with_forbidden_apis(mut self, apis: Vec<String>) -> Self {
        self.forbidden_external_apis = apis;
        self
    }
    
    pub fn with_required_checks(mut self, checks: Vec<String>) -> Self {
        self.required_security_checks = checks;
        self
    }
    
    pub fn with_audit_requirements(mut self, requirements: Vec<String>) -> Self {
        self.audit_requirements = requirements;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub operation: String,
    pub trust_level: TrustLevel,
    pub conditions: Vec<String>,
}

impl Permission {
    pub fn new(resource: String, operation: String, trust_level: TrustLevel) -> Self {
        Self {
            resource,
            operation,
            trust_level,
            conditions: Vec::new(),
        }
    }
    
    pub fn with_conditions(mut self, conditions: Vec<String>) -> Self {
        self.conditions = conditions;
        self
    }
}

// Predefined trust profiles
pub fn get_trust_profiles() -> HashMap<TrustLevel, SecurityProfile> {
    let mut profiles = HashMap::new();
    
    // Decentralized profile
    profiles.insert(TrustLevel::Decentralized, SecurityProfile::new(TrustLevel::Decentralized)
        .with_allowed_apis(vec![
            "oracle::fetch".to_string(),
            "chain::cross_chain".to_string(),
        ])
        .with_forbidden_apis(vec![
            "web::http_request".to_string(),
            "service::external_api".to_string(),
            "database::external".to_string(),
        ])
        .with_required_checks(vec![
            "crypto::verify_signature".to_string(),
            "auth::verify_identity".to_string(),
            "audit::log_operation".to_string(),
        ])
        .with_audit_requirements(vec![
            "all_operations_logged".to_string(),
            "signature_verification".to_string(),
            "immutable_audit_trail".to_string(),
        ])
    );
    
    // Hybrid profile
    profiles.insert(TrustLevel::Hybrid, SecurityProfile::new(TrustLevel::Hybrid)
        .with_allowed_apis(vec![
            "oracle::fetch".to_string(),
            "service::external_api".to_string(),
            "web::http_request".to_string(),
        ])
        .with_forbidden_apis(vec![
            "database::external".to_string(),
            "system::file_access".to_string(),
        ])
        .with_required_checks(vec![
            "auth::verify_identity".to_string(),
            "crypto::encrypt_data".to_string(),
            "audit::log_operation".to_string(),
        ])
        .with_audit_requirements(vec![
            "external_calls_logged".to_string(),
            "data_encryption".to_string(),
            "identity_verification".to_string(),
        ])
    );
    
    // Centralized profile
    profiles.insert(TrustLevel::Centralized, SecurityProfile::new(TrustLevel::Centralized)
        .with_allowed_apis(vec![
            "service::external_api".to_string(),
            "web::http_request".to_string(),
            "database::external".to_string(),
            "system::file_access".to_string(),
        ])
        .with_forbidden_apis(vec![
            "chain::transaction".to_string(),
            "oracle::fetch".to_string(),
        ])
        .with_required_checks(vec![
            "auth::verify_identity".to_string(),
            "audit::log_operation".to_string(),
        ])
        .with_audit_requirements(vec![
            "all_operations_logged".to_string(),
            "identity_verification".to_string(),
            "access_control".to_string(),
        ])
    );
    
    profiles
}

// NEW: Cross-Chain Support System
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockchainNetwork {
    Ethereum,
    Polygon,
    Binance,
    Solana,
    Avalanche,
    Arbitrum,
    Optimism,
    Custom(String),
}

impl BlockchainNetwork {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" => Some(BlockchainNetwork::Ethereum),
            "polygon" => Some(BlockchainNetwork::Polygon),
            "binance" => Some(BlockchainNetwork::Binance),
            "solana" => Some(BlockchainNetwork::Solana),
            "avalanche" => Some(BlockchainNetwork::Avalanche),
            "arbitrum" => Some(BlockchainNetwork::Arbitrum),
            "optimism" => Some(BlockchainNetwork::Optimism),
            _ => Some(BlockchainNetwork::Custom(s.to_string())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            BlockchainNetwork::Ethereum => "ethereum".to_string(),
            BlockchainNetwork::Polygon => "polygon".to_string(),
            BlockchainNetwork::Binance => "binance".to_string(),
            BlockchainNetwork::Solana => "solana".to_string(),
            BlockchainNetwork::Avalanche => "avalanche".to_string(),
            BlockchainNetwork::Arbitrum => "arbitrum".to_string(),
            BlockchainNetwork::Optimism => "optimism".to_string(),
            BlockchainNetwork::Custom(name) => name.clone(),
        }
    }
    
    pub fn is_evm_compatible(&self) -> bool {
        matches!(self, 
            BlockchainNetwork::Ethereum |
            BlockchainNetwork::Polygon |
            BlockchainNetwork::Binance |
            BlockchainNetwork::Avalanche |
            BlockchainNetwork::Arbitrum |
            BlockchainNetwork::Optimism
        )
    }
    
    pub fn is_solana_compatible(&self) -> bool {
        matches!(self, BlockchainNetwork::Solana)
    }
}

#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub network: BlockchainNetwork,
    pub chain_id: u64,
    pub rpc_url: String,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub native_token: String,
    pub block_time: u64,
    pub max_transaction_size: usize,
    pub supported_operations: Vec<String>,
    pub forbidden_operations: Vec<String>,
}

impl ChainConfig {
    pub fn new(network: BlockchainNetwork) -> Self {
        Self {
            network,
            chain_id: 0,
            rpc_url: String::new(),
            gas_limit: 0,
            gas_price: 0,
            native_token: String::new(),
            block_time: 0,
            max_transaction_size: 0,
            supported_operations: Vec::new(),
            forbidden_operations: Vec::new(),
        }
    }
    
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }
    
    pub fn with_rpc_url(mut self, rpc_url: String) -> Self {
        self.rpc_url = rpc_url;
        self
    }
    
    pub fn with_gas_config(mut self, gas_limit: u64, gas_price: u64) -> Self {
        self.gas_limit = gas_limit;
        self.gas_price = gas_price;
        self
    }
    
    pub fn with_native_token(mut self, token: String) -> Self {
        self.native_token = token;
        self
    }
    
    pub fn with_supported_operations(mut self, operations: Vec<String>) -> Self {
        self.supported_operations = operations;
        self
    }
    
    pub fn with_forbidden_operations(mut self, operations: Vec<String>) -> Self {
        self.forbidden_operations = operations;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CrossChainOperation {
    pub source_chain: BlockchainNetwork,
    pub target_chain: BlockchainNetwork,
    pub operation_type: CrossChainOpType,
    pub data: HashMap<String, String>,
    pub bridge_config: Option<BridgeConfig>,
}

#[derive(Debug, Clone)]
pub enum CrossChainOpType {
    Transfer,
    Deploy,
    Call,
    Bridge,
    Oracle,
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub bridge_address: String,
    pub bridge_fee: u64,
    pub bridge_timeout: u64,
    pub supported_tokens: Vec<String>,
}

// Predefined chain configurations
pub fn get_chain_configs() -> HashMap<BlockchainNetwork, ChainConfig> {
    let mut configs = HashMap::new();
    
    // Ethereum configuration
    configs.insert(BlockchainNetwork::Ethereum, ChainConfig::new(BlockchainNetwork::Ethereum)
        .with_chain_id(1)
        .with_rpc_url("https://mainnet.infura.io/v3/".to_string())
        .with_gas_config(21000, 20)
        .with_native_token("ETH".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "oracle::fetch".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "avalanche::subnet".to_string(),
        ])
    );
    
    // Polygon configuration
    configs.insert(BlockchainNetwork::Polygon, ChainConfig::new(BlockchainNetwork::Polygon)
        .with_chain_id(137)
        .with_rpc_url("https://polygon-rpc.com/".to_string())
        .with_gas_config(21000, 30)
        .with_native_token("MATIC".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "ethereum::layer1".to_string(),
        ])
    );
    
    // Solana configuration
    configs.insert(BlockchainNetwork::Solana, ChainConfig::new(BlockchainNetwork::Solana)
        .with_chain_id(101)
        .with_rpc_url("https://api.mainnet-beta.solana.com".to_string())
        .with_gas_config(5000, 5000)
        .with_native_token("SOL".to_string())
        .with_supported_operations(vec![
            "solana::deploy".to_string(),
            "solana::transaction".to_string(),
            "solana::instruction".to_string(),
            "oracle::fetch".to_string(),
        ])
        .with_forbidden_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "ethereum::layer1".to_string(),
        ])
    );
    
    // Binance Smart Chain configuration
    configs.insert(BlockchainNetwork::Binance, ChainConfig::new(BlockchainNetwork::Binance)
        .with_chain_id(56)
        .with_rpc_url("https://bsc-dataseed.binance.org/".to_string())
        .with_gas_config(21000, 5)
        .with_native_token("BNB".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "avalanche::subnet".to_string(),
        ])
    );
    
    // Avalanche configuration
    configs.insert(BlockchainNetwork::Avalanche, ChainConfig::new(BlockchainNetwork::Avalanche)
        .with_chain_id(43114)
        .with_rpc_url("https://api.avax.network/ext/bc/C/rpc".to_string())
        .with_gas_config(21000, 25)
        .with_native_token("AVAX".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "ethereum::layer1".to_string(),
        ])
    );
    
    // Arbitrum configuration
    configs.insert(BlockchainNetwork::Arbitrum, ChainConfig::new(BlockchainNetwork::Arbitrum)
        .with_chain_id(42161)
        .with_rpc_url("https://arb1.arbitrum.io/rpc".to_string())
        .with_gas_config(21000, 0) // L2 gas pricing
        .with_native_token("ETH".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "avalanche::subnet".to_string(),
        ])
    );
    
    // Optimism configuration
    configs.insert(BlockchainNetwork::Optimism, ChainConfig::new(BlockchainNetwork::Optimism)
        .with_chain_id(10)
        .with_rpc_url("https://mainnet.optimism.io".to_string())
        .with_gas_config(21000, 0) // L2 gas pricing
        .with_native_token("ETH".to_string())
        .with_supported_operations(vec![
            "chain::deploy".to_string(),
            "chain::transaction".to_string(),
            "chain::call".to_string(),
            "bridge::transfer".to_string(),
        ])
        .with_forbidden_operations(vec![
            "solana::instruction".to_string(),
            "avalanche::subnet".to_string(),
        ])
    );
    
    configs
}

// NEW: Interface Generation System
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InterfaceLanguage {
    TypeScript,
    JavaScript,
    Python,
    Rust,
    Java,
    Go,
    Custom(String),
}

impl InterfaceLanguage {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "typescript" => Some(InterfaceLanguage::TypeScript),
            "javascript" => Some(InterfaceLanguage::JavaScript),
            "python" => Some(InterfaceLanguage::Python),
            "rust" => Some(InterfaceLanguage::Rust),
            "java" => Some(InterfaceLanguage::Java),
            "go" => Some(InterfaceLanguage::Go),
            _ => Some(InterfaceLanguage::Custom(s.to_string())),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            InterfaceLanguage::TypeScript => "typescript".to_string(),
            InterfaceLanguage::JavaScript => "javascript".to_string(),
            InterfaceLanguage::Python => "python".to_string(),
            InterfaceLanguage::Rust => "rust".to_string(),
            InterfaceLanguage::Java => "java".to_string(),
            InterfaceLanguage::Go => "go".to_string(),
            InterfaceLanguage::Custom(name) => name.clone(),
        }
    }
    
    pub fn file_extension(&self) -> &str {
        match self {
            InterfaceLanguage::TypeScript => ".ts",
            InterfaceLanguage::JavaScript => ".js",
            InterfaceLanguage::Python => ".py",
            InterfaceLanguage::Rust => ".rs",
            InterfaceLanguage::Java => ".java",
            InterfaceLanguage::Go => ".go",
            InterfaceLanguage::Custom(_) => ".txt",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientInterface {
    pub language: InterfaceLanguage,
    pub service_name: String,
    pub chains: Vec<BlockchainNetwork>,
    pub methods: Vec<InterfaceMethod>,
    pub events: Vec<InterfaceEvent>,
    pub bridge_operations: Vec<BridgeOperation>,
    pub deployment_config: Option<DeploymentInterface>,
}

#[derive(Debug, Clone)]
pub struct InterfaceMethod {
    pub name: String,
    pub parameters: Vec<InterfaceParameter>,
    pub return_type: Option<String>,
    pub chain_specific: bool,
    pub supported_chains: Vec<BlockchainNetwork>,
    pub async_method: bool,
}

#[derive(Debug, Clone)]
pub struct InterfaceParameter {
    pub name: String,
    pub parameter_type: String,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InterfaceEvent {
    pub name: String,
    pub parameters: Vec<InterfaceParameter>,
    pub chain_specific: bool,
    pub supported_chains: Vec<BlockchainNetwork>,
}

#[derive(Debug, Clone)]
pub struct BridgeOperation {
    pub name: String,
    pub source_chain: BlockchainNetwork,
    pub target_chain: BlockchainNetwork,
    pub parameters: Vec<InterfaceParameter>,
    pub return_type: Option<String>,
    pub bridge_config: Option<BridgeConfig>,
}

#[derive(Debug, Clone)]
pub struct DeploymentInterface {
    pub deploy_methods: Vec<InterfaceMethod>,
    pub verify_methods: Vec<InterfaceMethod>,
    pub rollback_methods: Vec<InterfaceMethod>,
    pub status_methods: Vec<InterfaceMethod>,
}

// Predefined target constraints
pub fn get_target_constraints() -> HashMap<CompilationTarget, TargetConstraint> {
    let mut constraints = HashMap::new();
    
    // Blockchain constraints
    constraints.insert(CompilationTarget::Blockchain, TargetConstraint::new(CompilationTarget::Blockchain)
        .with_allowed_operations(vec![
            "chain::transaction".to_string(),
            "chain::deploy".to_string(),
            "chain::call".to_string(),
            "oracle::fetch".to_string(),
            "crypto::sign".to_string(),
            "crypto::verify".to_string(),
            "auth::verify".to_string(),
            "log::info".to_string(),
        ])
        .with_forbidden_operations(vec![
            "web::http_request".to_string(),
            "web::websocket".to_string(),
            "desktop::window".to_string(),
            "mobile::notification".to_string(),
            "iot::sensor_read".to_string(),
        ])
        .with_required_attributes(vec![
            "@secure".to_string(),
            "@trust".to_string(),
        ])
    );
    
    // WebAssembly constraints
    constraints.insert(CompilationTarget::WebAssembly, TargetConstraint::new(CompilationTarget::WebAssembly)
        .with_allowed_operations(vec![
            "web::dom_manipulation".to_string(),
            "web::local_storage".to_string(),
            "web::fetch".to_string(),
            "web::websocket".to_string(),
            "web::event_listener".to_string(),
            "web::element_create".to_string(),
        ])
        .with_forbidden_operations(vec![
            "chain::transaction".to_string(),
            "chain::deploy".to_string(),
            "desktop::file_system".to_string(),
            "mobile::camera".to_string(),
            "iot::device_control".to_string(),
        ])
        .with_required_attributes(vec![
            "@web".to_string(),
        ])
    );
    
    // Native constraints
    constraints.insert(CompilationTarget::Native, TargetConstraint::new(CompilationTarget::Native)
        .with_allowed_operations(vec![
            "system::file_io".to_string(),
            "system::process".to_string(),
            "system::network".to_string(),
            "desktop::window".to_string(),
            "desktop::menu".to_string(),
            "database::query".to_string(),
        ])
        .with_forbidden_operations(vec![
            "chain::transaction".to_string(),
            "mobile::touch_event".to_string(),
            "iot::sensor_read".to_string(),
        ])
        .with_required_attributes(vec![
            "@native".to_string(),
        ])
    );
    
    // Mobile constraints
    constraints.insert(CompilationTarget::Mobile, TargetConstraint::new(CompilationTarget::Mobile)
        .with_allowed_operations(vec![
            "mobile::notification".to_string(),
            "mobile::camera".to_string(),
            "mobile::location".to_string(),
            "mobile::touch_event".to_string(),
            "mobile::vibration".to_string(),
            "mobile::storage".to_string(),
        ])
        .with_forbidden_operations(vec![
            "chain::transaction".to_string(),
            "desktop::window".to_string(),
            "iot::device_control".to_string(),
        ])
        .with_required_attributes(vec![
            "@mobile".to_string(),
        ])
    );
    
    // Edge constraints
    constraints.insert(CompilationTarget::Edge, TargetConstraint::new(CompilationTarget::Edge)
        .with_allowed_operations(vec![
            "iot::sensor_read".to_string(),
            "iot::device_control".to_string(),
            "iot::data_process".to_string(),
            "iot::network_send".to_string(),
            "iot::local_storage".to_string(),
        ])
        .with_forbidden_operations(vec![
            "chain::transaction".to_string(),
            "web::dom_manipulation".to_string(),
            "desktop::window".to_string(),
            "mobile::camera".to_string(),
        ])
        .with_required_attributes(vec![
            "@edge".to_string(),
        ])
    );
    
    constraints
}
