// Test universal transforms (@cap, @upper, @lower)
use rlf::{rlf, Locale};

rlf! {
    hello = "hello";
    cap_test = "{@cap hello}";
    upper_test = "{@upper hello}";
    lower_test = "{@lower hello}";
    multi_transform = "{@upper @cap hello}";
}

fn main() {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);

    let _ = hello(&locale);
    let _ = cap_test(&locale);
    let _ = upper_test(&locale);
    let _ = lower_test(&locale);
    let _ = multi_transform(&locale);
}
