use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

// Global object ID counter for unique object identification
static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for objects in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u64);

impl ObjectId {
    pub fn new() -> Self {
        ObjectId(OBJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Object metadata for memory management and optimization
#[derive(Debug)]
pub struct ObjectMetadata {
    pub id: ObjectId,
    pub size_bytes: usize,
    pub reference_count: AtomicUsize, // Thread-safe reference counting
    pub last_accessed: Mutex<std::time::Instant>, // Mutable field needs Mutex for Arc safety
    pub is_immutable: bool,
    pub creation_time: std::time::Instant,
}

// Manual Clone implementation since AtomicUsize doesn't implement Clone
impl Clone for ObjectMetadata {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            size_bytes: self.size_bytes,
            reference_count: AtomicUsize::new(self.reference_count.load(Ordering::Acquire)),
            last_accessed: Mutex::new(*self.last_accessed.lock().unwrap()),
            is_immutable: self.is_immutable,
            creation_time: self.creation_time,
        }
    }
}

/// Reference-counted object wrapper for efficient memory management
#[derive(Debug, Clone)]
pub struct ObjectRef {
    pub id: ObjectId,
    pub metadata: Arc<ObjectMetadata>,
}

impl ObjectRef {
    pub fn new(size_bytes: usize, is_immutable: bool) -> Self {
        let id = ObjectId::new();
        let metadata = Arc::new(ObjectMetadata {
            id,
            size_bytes,
            reference_count: AtomicUsize::new(1),
            last_accessed: Mutex::new(std::time::Instant::now()),
            is_immutable,
            creation_time: std::time::Instant::now(),
        });

        ObjectRef { id, metadata }
    }

    /// Increment the reference count atomically.
    /// Call when creating a new reference to this object.
    pub fn increment_ref(&self) {
        self.metadata.reference_count.fetch_add(1, Ordering::AcqRel);
    }

    /// Decrement the reference count atomically.
    /// Call when dropping a reference to this object.
    /// Returns the previous count (before decrement).
    pub fn decrement_ref(&self) -> usize {
        self.metadata.reference_count.fetch_sub(1, Ordering::AcqRel)
    }

    /// Get the current reference count.
    pub fn ref_count(&self) -> usize {
        self.metadata.reference_count.load(Ordering::Acquire)
    }

    /// Update last accessed time (for GC heuristics).
    pub fn touch(&self) {
        if let Ok(mut last_accessed) = self.metadata.last_accessed.lock() {
            *last_accessed = std::time::Instant::now();
        }
    }
}

/// Stored object with reference count for GC (only collect when ref_count == 0).
#[derive(Debug)]
struct StoredObject(ObjectData, usize);

/// Central object registry for memory management and object identity
#[derive(Debug)]
pub struct ObjectRegistry {
    objects: HashMap<ObjectId, StoredObject>,
    total_memory_usage: usize,
    #[allow(dead_code)]
    max_memory_threshold: usize,
}

#[derive(Debug, Clone)]
pub enum ObjectData {
    Struct(String, HashMap<String, Value>), // struct_name, fields
    Map(HashMap<String, Value>),
    Array(Vec<Value>),
    List(Vec<Value>),
    Set(HashSet<String>),
}

impl ObjectRegistry {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            total_memory_usage: 0,
            max_memory_threshold: 100 * 1024 * 1024, // 100MB default
        }
    }

    pub fn create_object(&mut self, data: ObjectData, is_immutable: bool) -> ObjectRef {
        let size_bytes = self.calculate_size(&data);
        let object_ref = ObjectRef::new(size_bytes, is_immutable);

        self.objects.insert(object_ref.id, StoredObject(data, 1));
        self.total_memory_usage += size_bytes;

        object_ref
    }

    pub fn get_object(&self, id: &ObjectId) -> Option<&ObjectData> {
        self.objects.get(id).map(|s| &s.0)
    }

    pub fn get_object_mut(&mut self, id: &ObjectId) -> Option<&mut ObjectData> {
        self.objects.get_mut(id).map(|s| &mut s.0)
    }

    /// Increment reference count for an object; call when retaining a reference.
    pub fn register_ref(&mut self, id: &ObjectId) -> bool {
        if let Some(stored) = self.objects.get_mut(id) {
            stored.1 = stored.1.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Decrement reference count; returns true if count reached 0.
    pub fn unregister_ref(&mut self, id: &ObjectId) -> bool {
        if let Some(stored) = self.objects.get_mut(id) {
            stored.1 = stored.1.saturating_sub(1);
            stored.1 == 0
        } else {
            false
        }
    }

    pub fn get_ref_count(&self, id: &ObjectId) -> Option<usize> {
        self.objects.get(id).map(|s| s.1)
    }

    pub fn update_struct_field(
        &mut self,
        object_id: &ObjectId,
        field_name: &str,
        value: Value,
    ) -> Result<(), String> {
        if let Some(StoredObject(ObjectData::Struct(_, fields), _)) =
            self.objects.get_mut(object_id)
        {
            fields.insert(field_name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Object {} is not a struct", object_id.as_u64()))
        }
    }

    pub fn update_map_field(
        &mut self,
        object_id: &ObjectId,
        key: &str,
        value: Value,
    ) -> Result<(), String> {
        if let Some(StoredObject(ObjectData::Map(map), _)) = self.objects.get_mut(object_id) {
            map.insert(key.to_string(), value);
            Ok(())
        } else {
            Err(format!("Object {} is not a map", object_id.as_u64()))
        }
    }

    pub fn update_array_element(
        &mut self,
        object_id: &ObjectId,
        index: usize,
        value: Value,
    ) -> Result<(), String> {
        match self.objects.get_mut(object_id) {
            Some(StoredObject(ObjectData::Array(arr), _)) if index < arr.len() => {
                arr[index] = value;
                Ok(())
            }
            Some(StoredObject(ObjectData::List(list), _)) if index < list.len() => {
                list[index] = value;
                Ok(())
            }
            _ => Err(format!(
                "Cannot update element at index {} in object {}",
                index,
                object_id.as_u64()
            )),
        }
    }

    pub fn remove_object(&mut self, id: &ObjectId) -> bool {
        if let Some(StoredObject(removed, _)) = self.objects.remove(id) {
            let size = self.calculate_size(&removed);
            self.total_memory_usage = self.total_memory_usage.saturating_sub(size);
            true
        } else {
            false
        }
    }

    /// Remove only objects with reference count 0 (reachability: no live references).
    pub fn garbage_collect(&mut self) -> usize {
        let to_remove: Vec<ObjectId> = self
            .objects
            .iter()
            .filter(|(_, StoredObject(_, ref_count))| *ref_count == 0)
            .map(|(id, _)| *id)
            .collect();
        let removed_count = to_remove.len();
        for id in to_remove {
            self.remove_object(&id);
        }
        removed_count
    }

    pub fn get_memory_usage(&self) -> usize {
        self.total_memory_usage
    }

    pub fn get_object_count(&self) -> usize {
        self.objects.len()
    }

    fn calculate_size(&self, data: &ObjectData) -> usize {
        match data {
            ObjectData::Struct(_, fields) => {
                // Base struct overhead + field storage
                64 + fields.len() * 32 + fields.values().map(|v| self.value_size(v)).sum::<usize>()
            }
            ObjectData::Map(map) => {
                64 + map.len() * 32 + map.values().map(|v| self.value_size(v)).sum::<usize>()
            }
            ObjectData::Array(arr) | ObjectData::List(arr) => {
                64 + arr.len() * 16 + arr.iter().map(|v| self.value_size(v)).sum::<usize>()
            }
            ObjectData::Set(set) => 64 + set.len() * 24,
        }
    }

    fn value_size(&self, value: &Value) -> usize {
        match value {
            Value::Int(_) => 8,
            Value::Float(_) => 8,
            Value::Bool(_) => 1,
            Value::String(s) => 24 + s.len(), // String overhead + content
            Value::Null => 0,
            // For complex types, estimate their memory usage
            Value::Struct(_, fields) => {
                std::mem::size_of::<String>() + // struct name
                fields.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
                // fields
            }
            Value::Array(items) => items.len() * std::mem::size_of::<Value>(),
            Value::List(items) => items.len() * std::mem::size_of::<Value>(),
            Value::Map(entries) => {
                entries.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
            }
            Value::Set(items) => items.len() * std::mem::size_of::<String>(),
            Value::Result(_, _) | Value::Option(_) => 2 * std::mem::size_of::<Value>(),
            Value::Closure(id) => 24 + id.len(),
        }
    }
}

/// Global object registry instance
static OBJECT_REGISTRY: OnceLock<Mutex<ObjectRegistry>> = OnceLock::new();

/// Initialize the global object registry
pub fn init_object_registry() {
    OBJECT_REGISTRY.get_or_init(|| Mutex::new(ObjectRegistry::new()));
}

/// Get a lock on the global object registry
pub fn get_object_registry() -> std::sync::MutexGuard<'static, ObjectRegistry> {
    OBJECT_REGISTRY
        .get_or_init(|| Mutex::new(ObjectRegistry::new()))
        .lock()
        .expect("Object registry lock poisoned")
}

/// Create a new object in the registry
pub fn create_struct_object(struct_name: String, fields: HashMap<String, Value>) -> ObjectRef {
    let mut registry = get_object_registry();
    let data = ObjectData::Struct(struct_name, fields);
    registry.create_object(data, false) // Structs are mutable by default
}

/// Create a new map object in the registry
pub fn create_map_object(map: HashMap<String, Value>) -> ObjectRef {
    let mut registry = get_object_registry();
    let data = ObjectData::Map(map);
    registry.create_object(data, false) // Maps are mutable by default
}

/// Create a new array object in the registry
pub fn create_array_object(array: Vec<Value>) -> ObjectRef {
    let mut registry = get_object_registry();
    let data = ObjectData::Array(array);
    registry.create_object(data, false) // Arrays are mutable by default
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,

    // Generic types
    Result(Box<Value>, Box<Value>), // Ok(T), Err(E)
    Option(Option<Box<Value>>),     // Some(T), None
    List(Vec<Value>),               // [T] - Dynamic arrays
    Map(HashMap<String, Value>),    // map<K, V>
    Set(HashSet<String>),           // set<T>

    // Structured types
    Struct(String, HashMap<String, Value>), // struct_name, fields
    Array(Vec<Value>),                      // Array type

    /// Arrow/closure value; id refers to engine's closure_registry (param, body, captured_scope).
    Closure(String),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::Null => "null",
            Value::Result(_, _) => "result",
            Value::Option(_) => "option",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Set(_) => "set",
            Value::Struct(_, _) => "struct",
            Value::Array(_) => "array",
            Value::Closure(_) => "closure",
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_result(&self) -> bool {
        matches!(self, Value::Result(_, _))
    }

    pub fn is_option(&self) -> bool {
        matches!(self, Value::Option(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Value::Struct(_, _))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_closure(&self) -> bool {
        matches!(self, Value::Closure(_))
    }

    // Result methods
    pub fn is_ok(&self) -> bool {
        match self {
            Value::Result(_, _) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            Value::Result(_, _) => true,
            _ => false,
        }
    }

    pub fn unwrap_ok(&self) -> Option<&Value> {
        match self {
            Value::Result(ok_val, _) => Some(ok_val),
            _ => None,
        }
    }

    pub fn unwrap_err(&self) -> Option<&Value> {
        match self {
            Value::Result(_, err_val) => Some(err_val),
            _ => None,
        }
    }

    // Option methods
    pub fn is_some(&self) -> bool {
        match self {
            Value::Option(Some(_)) => true,
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Value::Option(None) => true,
            _ => false,
        }
    }

    pub fn unwrap_option(&self) -> Option<&Value> {
        match self {
            Value::Option(Some(val)) => Some(val),
            _ => None,
        }
    }

    // List methods
    pub fn list_length(&self) -> Option<usize> {
        match self {
            Value::List(list) => Some(list.len()),
            _ => None,
        }
    }

    pub fn list_get(&self, index: usize) -> Option<&Value> {
        match self {
            Value::List(list) => list.get(index),
            _ => None,
        }
    }

    pub fn list_push(&mut self, value: Value) -> bool {
        match self {
            Value::List(list) => {
                list.push(value);
                true
            }
            _ => false,
        }
    }

    // Map methods
    pub fn map_get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Map(map) => map.get(key),
            _ => None,
        }
    }

    pub fn map_set(&mut self, key: String, value: Value) -> bool {
        match self {
            Value::Map(map) => {
                map.insert(key, value);
                true
            }
            _ => false,
        }
    }

    pub fn map_keys(&self) -> Option<Vec<&String>> {
        match self {
            Value::Map(map) => Some(map.keys().collect()),
            _ => None,
        }
    }

    // Set methods
    pub fn set_add(&mut self, value: String) -> bool {
        match self {
            Value::Set(set) => {
                set.insert(value);
                true
            }
            _ => false,
        }
    }

    pub fn set_contains(&self, value: &str) -> bool {
        match self {
            Value::Set(set) => set.contains(value),
            _ => false,
        }
    }

    // Struct methods
    pub fn struct_get_field(&self, field_name: &str) -> Option<&Value> {
        match self {
            Value::Struct(_, fields) => fields.get(field_name),
            _ => None,
        }
    }

    pub fn struct_set_field(&mut self, field_name: String, value: Value) -> bool {
        match self {
            Value::Struct(_, fields) => {
                fields.insert(field_name, value);
                true
            }
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Result(ok_val, err_val) => {
                if self.is_ok() {
                    write!(f, "Ok({})", ok_val)
                } else {
                    write!(f, "Err({})", err_val)
                }
            }
            Value::Option(opt_val) => match opt_val {
                Some(val) => write!(f, "Some({})", val),
                None => write!(f, "None"),
            },
            Value::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::Set(set) => {
                write!(f, "{{")?;
                for (i, item) in set.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\"", item)?;
                }
                write!(f, "}}")
            }
            Value::Struct(name, fields) => {
                write!(f, "{} {{", name)?;
                for (i, (field_name, field_value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field_name, field_value)?;
                }
                write!(f, "}}")
            }
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Closure(id) => write!(f, "<closure {}>", id),
        }
    }
}

// From implementations for convenience
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value as f64)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(value)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(value: HashMap<String, Value>) -> Self {
        Value::Map(value)
    }
}

// Constructor methods for convenience
impl Value {
    pub fn ok(value: Value) -> Value {
        Value::Result(Box::new(value), Box::new(Value::Null))
    }

    pub fn err(error: Value) -> Value {
        Value::Result(Box::new(Value::Null), Box::new(error))
    }

    pub fn some(value: Value) -> Value {
        Value::Option(Some(Box::new(value)))
    }

    pub fn none() -> Value {
        Value::Option(None)
    }

    pub fn list(values: Vec<Value>) -> Value {
        Value::List(values)
    }

    pub fn map(entries: HashMap<String, Value>) -> Value {
        Value::Map(entries)
    }

    pub fn set(values: Vec<String>) -> Value {
        Value::Set(values.into_iter().collect())
    }

    pub fn struct_value(name: &str, fields: HashMap<String, Value>) -> Value {
        Value::Struct(name.to_string(), fields)
    }

    pub fn array(values: Vec<Value>) -> Value {
        Value::Array(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_ref_atomic_reference_counting() {
        let obj_ref = ObjectRef::new(1024, false);

        // Initial ref count should be 1
        assert_eq!(obj_ref.ref_count(), 1);

        // Increment ref count
        obj_ref.increment_ref();
        assert_eq!(obj_ref.ref_count(), 2);

        obj_ref.increment_ref();
        assert_eq!(obj_ref.ref_count(), 3);

        // Decrement ref count
        let prev = obj_ref.decrement_ref();
        assert_eq!(prev, 3); // Previous value before decrement
        assert_eq!(obj_ref.ref_count(), 2);

        obj_ref.decrement_ref();
        assert_eq!(obj_ref.ref_count(), 1);
    }

    #[test]
    fn test_object_ref_concurrent_access() {
        use std::thread;

        let obj_ref = Arc::new(ObjectRef::new(512, true));
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing the ref count 100 times
        for _ in 0..10 {
            let obj_ref_clone = Arc::clone(&obj_ref);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    obj_ref_clone.increment_ref();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have: 1 (initial) + 10 * 100 = 1001
        assert_eq!(obj_ref.ref_count(), 1001);
    }

    #[test]
    fn test_object_ref_touch_updates_last_accessed() {
        let obj_ref = ObjectRef::new(256, false);

        // Get initial time
        let initial_time = *obj_ref.metadata.last_accessed.lock().unwrap();

        // Sleep briefly
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Touch the object
        obj_ref.touch();

        // Last accessed should be updated
        let updated_time = *obj_ref.metadata.last_accessed.lock().unwrap();
        assert!(updated_time > initial_time);
    }

    #[test]
    fn test_object_metadata_clone() {
        let metadata = ObjectMetadata {
            id: ObjectId::new(),
            size_bytes: 2048,
            reference_count: AtomicUsize::new(5),
            last_accessed: Mutex::new(std::time::Instant::now()),
            is_immutable: true,
            creation_time: std::time::Instant::now(),
        };

        let cloned = metadata.clone();

        // Verify all fields are cloned correctly
        assert_eq!(cloned.id, metadata.id);
        assert_eq!(cloned.size_bytes, metadata.size_bytes);
        assert_eq!(cloned.reference_count.load(Ordering::Acquire), 5);
        assert_eq!(cloned.is_immutable, metadata.is_immutable);
    }
}
