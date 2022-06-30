use dejavu_macros::Template;
use dejavu_types::values::{Context, Value};

#[derive(Template)]
#[template(path = "templates/hello.dejavu")]
struct HelloTemplate;

#[test]
fn test_template_rendering() {
    let mut ctx = Context::new();
    ctx.set_var("name".to_string(), Value::String("World".to_string()));

    let items =
        vec![Value::String("Item 1".to_string()), Value::String("Item 2".to_string()), Value::String("Item 3".to_string())];
    ctx.set_var("items".to_string(), Value::Array(items));
    ctx.set_var("show_extra".to_string(), Value::Bool(true));

    let template = HelloTemplate;
    let result = template.render(&ctx).unwrap();

    assert!(result.contains("Hello, World!"));
    assert!(result.contains("- Item 1"));
    assert!(result.contains("- Item 2"));
    assert!(result.contains("- Item 3"));
    assert!(result.contains("This is extra content!"));
}

#[test]
fn test_template_without_extra() {
    let mut ctx = Context::new();
    ctx.set_var("name".to_string(), Value::String("Test".to_string()));
    ctx.set_var("items".to_string(), Value::Array(vec![]));
    ctx.set_var("show_extra".to_string(), Value::Bool(false));

    let template = HelloTemplate;
    let result = template.render(&ctx).unwrap();

    assert!(result.contains("Hello, Test!"));
    assert!(!result.contains("This is extra content!"));
}
