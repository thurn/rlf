use rlf::rlf;

rlf! {
    a = "see {b}";
    b = "see {c}";
    c = "see {a}";
}

fn main() {}
