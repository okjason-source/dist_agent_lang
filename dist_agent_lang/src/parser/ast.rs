use crate::lexer::tokens::Literal;
use crate::lexer::tokens::Operator;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Let(LetStatement),
    Return(ReturnStatement),
    Block(BlockStatement),
    Function(FunctionStatement),
    Service(ServiceStatement),  // NEW: Service statement
    Spawn(SpawnStatement),
    Agent(AgentStatement),
    Message(MessageStatement),
    Event(EventStatement),
    If(IfStatement),
    Try(TryStatement),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct FunctionStatement {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub body: BlockStatement,
    pub attributes: Vec<Attribute>,
    pub is_async: bool,
}

impl FunctionStatement {
    pub fn new(name: String, parameters: Vec<Parameter>, return_type: Option<String>, body: BlockStatement) -> Self {
        Self {
            name,
            parameters,
            return_type,
            body,
            attributes: Vec::new(),
            is_async: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct SpawnStatement {
    pub agent_name: String,
    pub agent_type: Option<String>, // "ai", "system", "worker", etc.
    pub config: Option<HashMap<String, Expression>>, // Agent configuration
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct AgentStatement {
    pub name: String,
    pub agent_type: AgentType,
    pub config: HashMap<String, Expression>,
    pub capabilities: Vec<String>,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub enum AgentType {
    AI,
    System,
    Worker,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct MessageStatement {
    pub recipient: String,
    pub data: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
pub struct EventStatement {
    pub event_name: String,
    pub data: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

#[derive(Debug, Clone)]
pub struct TryStatement {
    pub try_block: BlockStatement,
    pub catch_blocks: Vec<CatchBlock>,
    pub finally_block: Option<BlockStatement>,
}

#[derive(Debug, Clone)]
pub struct CatchBlock {
    pub error_type: Option<String>,
    pub error_variable: Option<String>,
    pub body: BlockStatement,
}

// NEW: Service-related structures

#[derive(Debug, Clone)]
pub struct ServiceField {
    pub name: String,
    pub field_type: String,
    pub initial_value: Option<Expression>,
    pub visibility: FieldVisibility,
}

#[derive(Debug, Clone)]
pub enum FieldVisibility {
    Public,
    Private,
    Internal,
}

#[derive(Debug, Clone)]
pub struct EventDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

// NEW: Compilation Target System
#[derive(Debug, Clone)]
pub struct CompilationTargetInfo {
    pub target: crate::lexer::tokens::CompilationTarget,
    pub constraints: crate::lexer::tokens::TargetConstraint,
    pub validation_errors: Vec<String>,
}

// Extend ServiceStatement with compilation target
#[derive(Debug, Clone)]
pub struct ServiceStatement {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub fields: Vec<ServiceField>,
    pub methods: Vec<FunctionStatement>,
    pub events: Vec<EventDeclaration>,
    pub compilation_target: Option<CompilationTargetInfo>, // NEW
}



#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub parameters: Vec<Expression>,
    pub target: AttributeTarget,
}

#[derive(Debug, Clone)]
pub enum AttributeTarget {
    Function,
    Block,
    Variable,
    Module,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp(Box<Expression>, Operator, Box<Expression>),
    UnaryOp(Operator, Box<Expression>),
    Assignment(String, Box<Expression>),
    FunctionCall(FunctionCall),
    FieldAccess(Box<Expression>, String),  // NEW: self.field syntax
    FieldAssignment(Box<Expression>, String, Box<Expression>),  // NEW: self.field = value
    Await(Box<Expression>),
    Throw(Box<Expression>),
    ObjectLiteral(HashMap<String, Expression>),  // NEW: object literal syntax
    ArrayLiteral(Vec<Expression>),  // NEW: array literal syntax [expr1, expr2, ...]
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}

impl BlockStatement {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}
