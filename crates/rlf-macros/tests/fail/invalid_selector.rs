use rlf::rlf;

rlf! {
    card = { one: "card", other: "cards" };
    bad = "{card:nonexistent}";
}

fn main() {}
