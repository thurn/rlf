use rlf::rlf;

rlf! {
    inner($x) = "[{$x}]";
    outer($x) = "({$x})";
    bad($y) = "{outer(inner($y))}";
}

fn main() {}
