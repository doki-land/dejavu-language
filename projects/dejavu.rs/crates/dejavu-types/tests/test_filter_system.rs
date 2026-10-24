use dejavu_types::values::{
    Context, Value,
    filter::{Filter, FilterRegistry},
};
use std::sync::Arc;

// 定义一个自定义过滤器
#[derive(Debug)]
struct CustomFilter;

impl Filter for CustomFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> dejavu_types::DejavuResult<Value> {
        match value {
            Value::String(s) => {
                let prefix = args
                    .get(0)
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("Custom: ");
                Ok(Value::String(format!("{}{}", prefix, s)))
            }
            _ => Ok(value),
        }
    }
}

#[test]
fn test_builtin_filters() {
    // 创建上下文
    let mut ctx = Context::new();

    // 测试 uppercase 过滤器
    let result = ctx.apply_filter("uppercase", Value::String("hello".to_string()), &[]).unwrap();
    assert_eq!(result, Value::String("HELLO".to_string()));

    // 测试 lowercase 过滤器
    let result = ctx.apply_filter("lowercase", Value::String("HELLO".to_string()), &[]).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    // 测试 trim 过滤器
    let result = ctx.apply_filter("trim", Value::String("  hello  ".to_string()), &[]).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    // 测试 format 过滤器
    let result = ctx
        .apply_filter(
            "format",
            Value::String("Hello, {0}! You are {1} years old.".to_string()),
            &[Value::String("World".to_string()), Value::Integer(20)],
        )
        .unwrap();
    assert_eq!(result, Value::String("Hello, World! You are 20 years old.".to_string()));
}

#[test]
fn test_custom_filter() {
    // 创建上下文
    let mut ctx = Context::new();

    // 注册自定义过滤器
    ctx.register_filter("custom", Arc::new(CustomFilter));

    // 测试自定义过滤器
    let result =
        ctx.apply_filter("custom", Value::String("test".to_string()), &[Value::String("Prefix: ".to_string())]).unwrap();
    assert_eq!(result, Value::String("Prefix: test".to_string()));
}

#[test]
fn test_filter_chain() {
    // 创建上下文
    let mut ctx = Context::new();

    // 测试过滤器链（模拟）
    let value = Value::String("  hello world  ".to_string());
    let result = ctx.apply_filter("trim", value.clone(), &[]).unwrap();
    let result = ctx.apply_filter("uppercase", result, &[]).unwrap();
    assert_eq!(result, Value::String("HELLO WORLD".to_string()));
}
