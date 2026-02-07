use rlf::rlf;

rlf! {
    cards($n) = :match($n) { 1: "a card", other: "{$n} cards" };
}

fn main() {}
