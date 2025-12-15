use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Basic types
    Int,
    Float,
    String,
    Bool,
    Null,
    
    // Generic types
    Result(Box<Type>, Box<Type>), // Ok(T), Err(E)
    Option(Box<Type>),            // Some(T), None
    List(Box<Type>),              // [T] - Dynamic arrays
    Map(Box<Type>, Box<Type>),    // map<K, V>
    Set(Box<Type>),              // set<T>
    
    // Structured types
    Struct(String, HashMap<String, Type>),
    Array(Box<Type>),
    
    // Function types
    Function(Vec<Type>, Box<Type>), // parameters, return type
    
    // Special types
    Any,
    Void,
    Error,
    
    // Generic type parameters
    Generic(String), // T, E, K, V, etc.
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
    
    pub fn is_boolean(&self) -> bool {
        matches!(self, Type::Bool)
    }
    
    pub fn is_string(&self) -> bool {
        matches!(self, Type::String)
    }
    
    pub fn is_null(&self) -> bool {
        matches!(self, Type::Null)
    }
    
    pub fn is_generic(&self) -> bool {
        matches!(self, Type::Generic(_))
    }
    
    pub fn is_result(&self) -> bool {
        matches!(self, Type::Result(_, _))
    }
    
    pub fn is_option(&self) -> bool {
        matches!(self, Type::Option(_))
    }
    
    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_))
    }
    
    pub fn is_map(&self) -> bool {
        matches!(self, Type::Map(_, _))
    }
    
    pub fn is_set(&self) -> bool {
        matches!(self, Type::Set(_))
    }
    
    pub fn get_generic_name(&self) -> Option<&str> {
        match self {
            Type::Generic(name) => Some(name),
            _ => None,
        }
    }
    
    pub fn substitute_generic(&self, generic_name: &str, concrete_type: &Type) -> Type {
        match self {
            Type::Generic(name) if name == generic_name => concrete_type.clone(),
            Type::Result(ok_type, err_type) => Type::Result(
                Box::new(ok_type.substitute_generic(generic_name, concrete_type)),
                Box::new(err_type.substitute_generic(generic_name, concrete_type))
            ),
            Type::Option(inner_type) => Type::Option(
                Box::new(inner_type.substitute_generic(generic_name, concrete_type))
            ),
            Type::List(inner_type) => Type::List(
                Box::new(inner_type.substitute_generic(generic_name, concrete_type))
            ),
            Type::Map(key_type, value_type) => Type::Map(
                Box::new(key_type.substitute_generic(generic_name, concrete_type)),
                Box::new(value_type.substitute_generic(generic_name, concrete_type))
            ),
            Type::Set(inner_type) => Type::Set(
                Box::new(inner_type.substitute_generic(generic_name, concrete_type))
            ),
            _ => self.clone(),
        }
    }
    
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Null, _) | (_, Type::Null) => true,
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,
            (Type::Result(ok1, err1), Type::Result(ok2, err2)) => {
                ok1.is_compatible_with(ok2) && err1.is_compatible_with(err2)
            },
            (Type::Option(inner1), Type::Option(inner2)) => {
                inner1.is_compatible_with(inner2)
            },
            (Type::List(inner1), Type::List(inner2)) => {
                inner1.is_compatible_with(inner2)
            },
            (Type::Map(key1, val1), Type::Map(key2, val2)) => {
                key1.is_compatible_with(key2) && val1.is_compatible_with(val2)
            },
            (Type::Set(inner1), Type::Set(inner2)) => {
                inner1.is_compatible_with(inner2)
            },
            (Type::Generic(_), _) | (_, Type::Generic(_)) => true, // Generic types are compatible
            _ => self == other,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Result(ok_type, err_type) => write!(f, "Result<{}, {}>", ok_type, err_type),
            Type::Option(inner_type) => write!(f, "Option<{}>", inner_type),
            Type::List(inner_type) => write!(f, "[{}]", inner_type),
            Type::Map(key_type, value_type) => write!(f, "map<{}, {}>", key_type, value_type),
            Type::Set(inner_type) => write!(f, "set<{}>", inner_type),
            Type::Struct(name, fields) => {
                write!(f, "struct {} {{", name)?;
                for (field_name, field_type) in fields {
                    write!(f, " {}: {},", field_name, field_type)?;
                }
                write!(f, " }}")
            },
            Type::Array(inner_type) => write!(f, "array<{}>", inner_type),
            Type::Function(params, return_type) => {
                write!(f, "fn(")?;
                for (i, param_type) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param_type)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Any => write!(f, "any"),
            Type::Void => write!(f, "void"),
            Type::Error => write!(f, "error"),
            Type::Generic(name) => write!(f, "{}", name),
        }
    }
}

// Type constructors for convenience
impl Type {
    pub fn result(ok_type: Type, err_type: Type) -> Type {
        Type::Result(Box::new(ok_type), Box::new(err_type))
    }
    
    pub fn option(inner_type: Type) -> Type {
        Type::Option(Box::new(inner_type))
    }
    
    pub fn list(inner_type: Type) -> Type {
        Type::List(Box::new(inner_type))
    }
    
    pub fn map(key_type: Type, value_type: Type) -> Type {
        Type::Map(Box::new(key_type), Box::new(value_type))
    }
    
    pub fn set(inner_type: Type) -> Type {
        Type::Set(Box::new(inner_type))
    }
    
    pub fn generic(name: &str) -> Type {
        Type::Generic(name.to_string())
    }
}

// Type checker trait
pub trait TypeCheck {
    fn type_check(&self) -> Result<Type, TypeError>;
}

#[derive(Debug, thiserror::Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch {
        expected: Type,
        got: Type,
    },
    
    #[error("Undefined field '{field}' in type {type_name}")]
    UndefinedField {
        field: String,
        type_name: String,
    },
    
    #[error("Invalid function call: expected {expected} parameters, got {got}")]
    ParameterCountMismatch {
        expected: usize,
        got: usize,
    },
    
    #[error("Invalid array access: index must be int, got {got}")]
    InvalidArrayIndex {
        got: Type,
    },
    
    #[error("Circular type definition detected")]
    CircularType,
}

// Type environment for tracking types during compilation
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    variables: HashMap<String, Type>,
    functions: HashMap<String, Type>,
    structs: HashMap<String, HashMap<String, Type>>,
    parent: Option<Box<TypeEnvironment>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn with_parent(parent: TypeEnvironment) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define_variable(&mut self, name: String, type_: Type) {
        self.variables.insert(name, type_);
    }
    
    pub fn define_function(&mut self, name: String, type_: Type) {
        self.functions.insert(name, type_);
    }
    
    pub fn define_struct(&mut self, name: String, fields: HashMap<String, Type>) {
        self.structs.insert(name, fields);
    }
    
    pub fn get_variable_type(&self, name: &str) -> Option<&Type> {
        self.variables.get(name).or_else(|| {
            self.parent.as_ref().and_then(|parent| parent.get_variable_type(name))
        })
    }
    
    pub fn get_function_type(&self, name: &str) -> Option<&Type> {
        self.functions.get(name).or_else(|| {
            self.parent.as_ref().and_then(|parent| parent.get_function_type(name))
        })
    }
    
    pub fn get_struct_fields(&self, name: &str) -> Option<&HashMap<String, Type>> {
        self.structs.get(name).or_else(|| {
            self.parent.as_ref().and_then(|parent| parent.get_struct_fields(name))
        })
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
