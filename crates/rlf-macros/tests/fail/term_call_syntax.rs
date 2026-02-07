use rlf::rlf;

rlf! {
    card = { one: "card", other: "cards" };
    bad($n) = "{card($n)}";
}

fn main() {}
