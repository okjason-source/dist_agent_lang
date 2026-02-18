// Python FFI bindings using PyO3
// Provides high-performance direct function calls from Python

#[cfg(feature = "python-ffi")]
use crate::runtime::engine::Runtime;
#[cfg(feature = "python-ffi")]
use crate::runtime::values::Value;
#[cfg(feature = "python-ffi")]
use pyo3::prelude::*;

/// Python FFI module for dist_agent_lang
#[cfg(feature = "python-ffi")]
#[pymodule]
fn dist_agent_lang(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> PyResult<()> {
    m.add_class::<DistAgentLangRuntime>()?;
    m.add_function(wrap_pyfunction!(hash_data, m)?)?;
    m.add_function(wrap_pyfunction!(sign_data, m)?)?;
    m.add_function(wrap_pyfunction!(verify_signature, m)?)?;
    Ok(())
}

/// Python runtime wrapper
#[cfg(feature = "python-ffi")]
#[pyclass(unsendable)]
pub struct DistAgentLangRuntime {
    runtime: Runtime,
}

#[cfg(feature = "python-ffi")]
#[pymethods]
impl DistAgentLangRuntime {
    #[new]
    fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    /// Call a registered function by name
    fn call_function(&mut self, name: String, args: Vec<PyObject>) -> PyResult<PyObject> {
        let dal_args: Vec<Value> = args
            .into_iter()
            .map(python_to_dal_value)
            .collect::<Result<Vec<Value>, _>>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        let result = self
            .runtime
            .call_function(&name, &dal_args)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

        Ok(dal_to_python_value(result))
    }

    /// Execute dist_agent_lang source code
    fn execute(&mut self, source: String) -> PyResult<PyObject> {
        let program = crate::parse_source(&source)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PySyntaxError, _>(format!("{}", e)))?;

        let result = self
            .runtime
            .execute(&program)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

        Ok(dal_to_python_value(result))
    }
}

/// Hash data using dist_agent_lang crypto
#[cfg(feature = "python-ffi")]
#[pyfunction]
fn hash_data(data: Vec<u8>, algorithm: Option<String>) -> PyResult<String> {
    let algo = algorithm.unwrap_or_else(|| "SHA256".to_string());
    let result = crate::stdlib::crypto::hash_bytes(&data, &algo)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
    Ok(result)
}

/// Sign data using dist_agent_lang crypto (key as hex/base64 string)
#[cfg(feature = "python-ffi")]
#[pyfunction]
fn sign_data(data: Vec<u8>, private_key: String) -> PyResult<String> {
    let signature = crate::stdlib::crypto_signatures::sign(&data, &private_key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
    Ok(signature)
}

/// Verify signature using dist_agent_lang crypto (signature and key as hex/base64 strings)
#[cfg(feature = "python-ffi")]
#[pyfunction]
fn verify_signature(data: Vec<u8>, signature: String, public_key: String) -> PyResult<bool> {
    let valid = crate::stdlib::crypto_signatures::verify(&data, &signature, &public_key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
    Ok(valid)
}

#[cfg(feature = "python-ffi")]
fn python_to_dal_value(py_value: PyObject) -> Result<Value, String> {
    Python::with_gil(|py| {
        let value = py_value.bind(py);
        if value.is_instance_of::<pyo3::types::PyBool>() {
            let b: bool = value.extract().map_err(|e| format!("{}", e))?;
            Ok(Value::Bool(b))
        } else if value.is_instance_of::<pyo3::types::PyInt>() {
            let i: i64 = value.extract().map_err(|e| format!("{}", e))?;
            Ok(Value::Int(i))
        } else if value.is_instance_of::<pyo3::types::PyFloat>() {
            let f: f64 = value.extract().map_err(|e| format!("{}", e))?;
            Ok(Value::Float(f))
        } else if value.is_instance_of::<pyo3::types::PyString>() {
            let s: String = value.extract().map_err(|e| format!("{}", e))?;
            Ok(Value::String(s))
        } else if value.is_none() {
            Ok(Value::Null)
        } else if value.is_instance_of::<pyo3::types::PyList>() {
            let list: Vec<PyObject> = value.extract().map_err(|e| format!("{}", e))?;
            let values: Result<Vec<Value>, String> =
                list.into_iter().map(python_to_dal_value).collect();
            Ok(Value::Array(values?))
        } else if value.is_instance_of::<pyo3::types::PyDict>() {
            let dict: std::collections::HashMap<String, PyObject> =
                value.extract().map_err(|e| format!("{}", e))?;
            let mut value_map = std::collections::HashMap::new();
            for (k, v) in dict {
                value_map.insert(k, python_to_dal_value(v)?);
            }
            Ok(Value::Map(value_map))
        } else {
            Err("Unsupported Python type".to_string())
        }
    })
}

#[cfg(feature = "python-ffi")]
#[allow(deprecated)] // IntoPy still works; IntoPyObject migration can follow
fn dal_to_python_value(value: Value) -> PyObject {
    Python::with_gil(|py| match value {
        Value::Int(i) => i.into_py(py),
        Value::Float(f) => f.into_py(py),
        Value::String(s) | Value::Closure(s) => s.into_py(py),
        Value::Bool(b) => b.into_py(py),
        Value::Null => py.None(),
        Value::Array(items) | Value::List(items) => {
            let py_list = pyo3::types::PyList::empty(py);
            for v in items {
                py_list.append(dal_to_python_value(v)).unwrap();
            }
            py_list.unbind()
        }
        Value::Map(map) => {
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, v) in map {
                py_dict.set_item(k, dal_to_python_value(v)).unwrap();
            }
            py_dict.unbind()
        }
        Value::Struct(_, fields) => {
            let py_dict = pyo3::types::PyDict::new(py);
            for (k, v) in fields {
                py_dict.set_item(k, dal_to_python_value(v)).unwrap();
            }
            py_dict.unbind()
        }
        Value::Set(set) => {
            let py_list = pyo3::types::PyList::empty(py);
            for v in set {
                py_list.append(v).unwrap();
            }
            py_list.unbind()
        }
        Value::Result(ok, _err) => dal_to_python_value(*ok),
        Value::Option(opt) => match opt {
            Some(v) => dal_to_python_value(*v),
            None => py.None(),
        },
    })
}

// Stub implementations when Python FFI is not enabled
#[cfg(not(feature = "python-ffi"))]
pub struct DistAgentLangRuntime;

#[cfg(not(feature = "python-ffi"))]
impl DistAgentLangRuntime {
    pub fn new() -> Self {
        Self
    }
}
