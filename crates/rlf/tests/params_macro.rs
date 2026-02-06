use rlf::{Phrase, Value, params};

#[test]
fn empty_params() {
    let p = params! {};
    assert!(p.is_empty());
}

#[test]
fn single_integer_param() {
    let p = params! { "n" => 42 };
    assert_eq!(p.len(), 1);
    assert_eq!(p["n"].as_number(), Some(42));
}

#[test]
fn single_string_param() {
    let p = params! { "name" => "Alice" };
    assert_eq!(p.len(), 1);
    assert_eq!(p["name"].as_string(), Some("Alice"));
}

#[test]
fn multiple_params() {
    let p = params! {
        "count" => 3,
        "name" => "Bob",
        "score" => 9.5_f64
    };
    assert_eq!(p.len(), 3);
    assert_eq!(p["count"].as_number(), Some(3));
    assert_eq!(p["name"].as_string(), Some("Bob"));
    assert_eq!(p["score"].as_float(), Some(9.5));
}

#[test]
fn trailing_comma() {
    let p = params! {
        "a" => 1,
        "b" => 2,
    };
    assert_eq!(p.len(), 2);
    assert_eq!(p["a"].as_number(), Some(1));
    assert_eq!(p["b"].as_number(), Some(2));
}

#[test]
fn various_integer_types() {
    let p = params! {
        "i32" => 10_i32,
        "i64" => 20_i64,
        "u32" => 30_u32,
        "u64" => 40_u64,
        "usize" => 50_usize
    };
    assert_eq!(p.len(), 5);
    assert_eq!(p["i32"].as_number(), Some(10));
    assert_eq!(p["i64"].as_number(), Some(20));
    assert_eq!(p["u32"].as_number(), Some(30));
    assert_eq!(p["u64"].as_number(), Some(40));
    assert_eq!(p["usize"].as_number(), Some(50));
}

#[test]
fn float_types() {
    let p = params! {
        "f32" => 1.5_f32,
        "f64" => 2.5_f64
    };
    assert_eq!(p.len(), 2);
    assert_eq!(p["f32"].as_float(), Some(1.5));
    assert_eq!(p["f64"].as_float(), Some(2.5));
}

#[test]
fn owned_string_value() {
    let name = String::from("Charlie");
    let p = params! { "name" => name };
    assert_eq!(p["name"].as_string(), Some("Charlie"));
}

#[test]
fn phrase_value() {
    let phrase = Phrase::builder().text("sword".to_string()).build();
    let p = params! { "weapon" => phrase };
    assert!(p["weapon"].as_phrase().is_some());
    assert_eq!(p["weapon"].to_string(), "sword");
}

#[test]
fn value_directly() {
    let v = Value::Number(99);
    let p = params! { "x" => v };
    assert_eq!(p["x"].as_number(), Some(99));
}

#[test]
fn expression_keys() {
    let key = "dynamic_key";
    let p = params! { key => 7 };
    assert_eq!(p["dynamic_key"].as_number(), Some(7));
}

#[test]
fn expression_values() {
    let count = 2 + 3;
    let p = params! { "total" => count };
    assert_eq!(p["total"].as_number(), Some(5));
}
