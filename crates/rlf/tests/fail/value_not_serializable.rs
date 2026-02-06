use rlf::Value;

fn assert_serialize<T: serde::Serialize>() {}

fn main() {
    assert_serialize::<Value>();
}
