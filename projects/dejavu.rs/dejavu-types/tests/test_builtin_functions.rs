use dejavu_types::values::{Context, Value};

#[test]
fn test_range_function() {
    let context = Context::new();
    let result = context.call_function("range", &[Value::Integer(0), Value::Integer(5)]).unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[1], Value::Integer(1));
            assert_eq!(arr[2], Value::Integer(2));
            assert_eq!(arr[3], Value::Integer(3));
            assert_eq!(arr[4], Value::Integer(4));
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_len_function() {
    let context = Context::new();
    let result = context.call_function("len", &[Value::String("hello".to_string())]).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_join_function() {
    let context = Context::new();
    let array =
        Value::Array(vec![Value::String("a".to_string()), Value::String("b".to_string()), Value::String("c".to_string())]);
    let result = context.call_function("join", &[array, Value::String(",".to_string())]).unwrap();
    assert_eq!(result, Value::String("a,b,c".to_string()));
}

#[test]
fn test_abs_function() {
    let context = Context::new();
    let result = context.call_function("abs", &[Value::Integer(-5)]).unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_max_function() {
    let context = Context::new();
    let result = context.call_function("max", &[Value::Integer(1), Value::Integer(3), Value::Integer(2)]).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_min_function() {
    let context = Context::new();
    let result = context.call_function("min", &[Value::Integer(1), Value::Integer(3), Value::Integer(2)]).unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_sum_function() {
    let context = Context::new();
    let result = context.call_function("sum", &[Value::Integer(1), Value::Integer(2), Value::Integer(3)]).unwrap();
    assert_eq!(result, Value::Decimal(6.0));
}

#[test]
fn test_split_function() {
    let context = Context::new();
    let result = context.call_function("split", &[Value::String("a,b,c".to_string()), Value::String(",".to_string())]).unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::String("a".to_string()));
            assert_eq!(arr[1], Value::String("b".to_string()));
            assert_eq!(arr[2], Value::String("c".to_string()));
        }
        _ => panic!("Expected array result"),
    }
}

#[test]
fn test_str_function() {
    let context = Context::new();
    let result = context.call_function("str", &[Value::Integer(42)]).unwrap();
    assert_eq!(result, Value::String("42".to_string()));
}

#[test]
fn test_int_function() {
    let context = Context::new();
    let result = context.call_function("int", &[Value::String("42".to_string())]).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_float_function() {
    let context = Context::new();
    let result = context.call_function("float", &[Value::String("3.14".to_string())]).unwrap();
    assert_eq!(result, Value::Decimal(3.14));
}

#[test]
fn test_bool_function() {
    let context = Context::new();
    let result = context.call_function("bool", &[Value::Integer(1)]).unwrap();
    assert_eq!(result, Value::Bool(true));
}
