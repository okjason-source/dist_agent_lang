// Python FFI bindings using PyO3
// Provides high-performance direct function calls from Python

#[cfg(feature = "python-ffi")]
use pyo3::prelude::*;
#[cfg(feature = "python-ffi")]
use crate::runtime::values::Value;
#[cfg(feature = "python-ffi")]
use crate::runtime::engine::Runtime;

/// Python FFI module for dist_agent_lang
#[cfg(feature = "python-ffi")]
#[pymodule]
fn dist_agent_lang(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DistAgentLangRuntime>()?;
    m.add_function(wrap_pyfunction!(hash_data, m)?)?;
    m.add_function(wrap_pyfunction!(sign_data, m)?)?;
    m.add_function(wrap_pyfunction!(verify_signature, m)?)?;
    Ok(())
}

/// Python runtime wrapper
#[cfg(feature = "python-ffi")]
#[pyclass]
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

    /// Call a service function
    fn call_function(
        &mut self,
        service_name: String,
        function_name: String,
        args: Vec<PyValue>,
    ) -> PyResult<PyValue> {
        // Convert Python values to dist_agent_lang values
        let dal_args: Vec<Value> = args
            .into_iter()
            .map(|v| python_to_dal_value(v))
            .collect::<Result<Vec<Value>, _>>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        // Execute function
        let result = self
            .runtime
            .execute_function(&service_name, &function_name, &dal_args)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

        // Convert result back to Python
        Ok(dal_to_python_value(result))
    }

    /// Execute dist_agent_lang source code
    fn execute(&mut self, source: String) -> PyResult<PyValue> {
        let result = self
            .runtime
            .execute_source(&source)
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

/// Sign data using dist_agent_lang crypto
#[cfg(feature = "python-ffi")]
#[pyfunction]
fn sign_data(data: Vec<u8>, private_key: Vec<u8>) -> PyResult<Vec<u8>> {
    let signature = crate::stdlib::crypto_signatures::sign(&data, &private_key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
    Ok(signature)
}

/// Verify signature using dist_agent_lang crypto
#[cfg(feature = "python-ffi")]
#[pyfunction]
fn verify_signature(data: Vec<u8>, signature: Vec<u8>, public_key: Vec<u8>) -> PyResult<bool> {
    let valid = crate::stdlib::crypto_signatures::verify(&data, &signature, &public_key)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
    Ok(valid)
}

// Type aliases for Python values
#[cfg(feature = "python-ffi")]
type PyValue = PyObject;

#[cfg(feature = "python-ffi")]
fn python_to_dal_value(py_value: PyValue) -> Result<Value, String> {
    Python::with_gil(|py| {
        if py_value.is_instance_of::<pyo3::types::PyInt>(py)? {
            let i: i64 = py_value.extract(py)?;
            Ok(Value::Int(i))
        } else if py_value.is_instance_of::<pyo3::types::PyFloat>(py)? {
            let f: f64 = py_value.extract(py)?;
            Ok(Value::Float(f))
        } else if py_value.is_instance_of::<pyo3::types::PyString>(py)? {
            let s: String = py_value.extract(py)?;
            Ok(Value::String(s))
        } else if py_value.is_instance_of::<pyo3::types::PyBool>(py)? {
            let b: bool = py_value.extract(py)?;
            Ok(Value::Bool(b))
        } else if py_value.is_none(py) {
            Ok(Value::Null)
        } else if py_value.is_instance_of::<pyo3::types::PyList>(py)? {
            let list: Vec<PyObject> = py_value.extract(py)?;
            let values: Result<Vec<Value>, String> =
                list.into_iter().map(|v| python_to_dal_value(v)).collect();
            Ok(Value::Array(values?))
        } else if py_value.is_instance_of::<pyo3::types::PyDict>(py)? {
            let dict: std::collections::HashMap<String, PyObject> = py_value.extract(py)?;
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
fn dal_to_python_value(value: Value) -> PyValue {
    Python::with_gil(|py| {
        match value {
            Value::Int(i) => i.into_py(py),
            Value::Float(f) => f.into_py(py),
            Value::String(s) => s.into_py(py),
            Value::Bool(b) => b.into_py(py),
            Value::Null => py.None(),
            Value::Array(arr) => {
                let py_list = pyo3::types::PyList::empty(py);
                for v in arr {
                    py_list.append(dal_to_python_value(v)).unwrap();
                }
                py_list.into()
            }
            Value::Map(map) => {
                let py_dict = pyo3::types::PyDict::new(py);
                for (k, v) in map {
                    py_dict.set_item(k, dal_to_python_value(v)).unwrap();
                }
                py_dict.into()
            }
        }
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
