use rlf::rlf;

rlf! {
    // Four-node cycle: a -> b -> c -> d -> a
    a = "see {b}";
    b = "see {c}";
    c = "see {d}";
    d = "see {a}";
}

fn main() {}
