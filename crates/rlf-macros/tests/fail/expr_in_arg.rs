use rlf::rlf;

rlf! {
    card = { one: "card", other: "cards" };
    f($x) = "{$x}";
    bad($y) = "{f(card:one)}";
}

fn main() {}
