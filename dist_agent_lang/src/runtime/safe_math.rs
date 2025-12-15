/// Safe Mathematics Operations for DAL Runtime
/// Provides overflow/underflow protection for all arithmetic operations

use crate::runtime::values::Value;
use crate::runtime::functions::RuntimeError;

/// Safe arithmetic operations with overflow/underflow protection
pub struct SafeMath;

impl SafeMath {
    /// Safe addition with overflow checking
    pub fn add(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                a.checked_add(*b)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerOverflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                let result = a + b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let result = (*a as f64) + b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let result = a + (*b as f64);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("addition".to_string()))
        }
    }

    /// Safe subtraction with underflow checking
    pub fn subtract(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                a.checked_sub(*b)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerUnderflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                let result = a - b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerUnderflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let result = (*a as f64) - b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerUnderflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let result = a - (*b as f64);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerUnderflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("subtraction".to_string()))
        }
    }

    /// Safe multiplication with overflow checking
    pub fn multiply(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                a.checked_mul(*b)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerOverflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                let result = a * b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let result = (*a as f64) * b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let result = a * (*b as f64);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("multiplication".to_string()))
        }
    }

    /// Safe division with overflow and division by zero checking
    pub fn divide(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                a.checked_div(*b)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerOverflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                let result = a / b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                let result = (*a as f64) / b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                let result = a / (*b as f64);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("division".to_string()))
        }
    }

    /// Safe modulo operation
    pub fn modulo(left: &Value, right: &Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                a.checked_rem(*b)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerOverflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                let result = a % b;
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("modulo".to_string()))
        }
    }

    /// Safe exponentiation
    pub fn power(base: &Value, exp: &Value) -> Result<Value, RuntimeError> {
        match (base, exp) {
            (Value::Int(a), Value::Int(b)) => {
                if *b < 0 {
                    return Err(RuntimeError::General("Negative exponent not supported for integers".to_string()));
                }
                if *b > 32 {
                    return Err(RuntimeError::IntegerOverflow);
                }
                a.checked_pow(*b as u32)
                    .map(Value::Int)
                    .ok_or(RuntimeError::IntegerOverflow)
            }
            (Value::Float(a), Value::Float(b)) => {
                let result = a.powf(*b);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                let result = (*a as f64).powf(*b);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                let result = a.powi(*b as i32);
                if result.is_infinite() || result.is_nan() {
                    Err(RuntimeError::IntegerOverflow)
                } else {
                    Ok(Value::Float(result))
                }
            }
            _ => Err(RuntimeError::TypeMismatch("exponentiation".to_string()))
        }
    }

    /// Check if a value is within safe integer range
    pub fn is_safe_integer(value: &Value) -> bool {
        match value {
            Value::Int(n) => *n >= i64::MIN / 2 && *n <= i64::MAX / 2,
            Value::Float(f) => *f >= (i64::MIN / 2) as f64 && *f <= (i64::MAX / 2) as f64,
            _ => false
        }
    }

    /// Convert to safe integer with bounds checking
    pub fn to_safe_integer(value: &Value) -> Result<i64, RuntimeError> {
        match value {
            Value::Int(n) => {
                if Self::is_safe_integer(value) {
                    Ok(*n)
                } else {
                    Err(RuntimeError::IntegerOverflow)
                }
            }
            Value::Float(f) => {
                if f.is_finite() && Self::is_safe_integer(value) {
                    Ok(*f as i64)
                } else {
                    Err(RuntimeError::IntegerOverflow)
                }
            }
            _ => Err(RuntimeError::TypeMismatch("integer conversion".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_addition() {
        let a = Value::Int(100);
        let b = Value::Int(200);
        let result = SafeMath::add(&a, &b).unwrap();
        assert_eq!(result, Value::Int(300));

        // Test overflow
        let max_val = Value::Int(i64::MAX);
        let one = Value::Int(1);
        let overflow_result = SafeMath::add(&max_val, &one);
        assert!(overflow_result.is_err());
    }

    #[test]
    fn test_safe_subtraction() {
        let a = Value::Int(300);
        let b = Value::Int(100);
        let result = SafeMath::subtract(&a, &b).unwrap();
        assert_eq!(result, Value::Int(200));

        // Test underflow
        let min_val = Value::Int(i64::MIN);
        let one = Value::Int(1);
        let underflow_result = SafeMath::subtract(&min_val, &one);
        assert!(underflow_result.is_err());
    }

    #[test]
    fn test_safe_division() {
        let a = Value::Int(100);
        let b = Value::Int(10);
        let result = SafeMath::divide(&a, &b).unwrap();
        assert_eq!(result, Value::Int(10));

        // Test division by zero
        let zero = Value::Int(0);
        let div_by_zero = SafeMath::divide(&a, &zero);
        assert!(div_by_zero.is_err());
    }

    #[test]
    fn test_mixed_type_arithmetic() {
        let int_val = Value::Int(10);
        let float_val = Value::Float(5.5);
        
        let result = SafeMath::add(&int_val, &float_val).unwrap();
        assert_eq!(result, Value::Float(15.5));
    }
}
