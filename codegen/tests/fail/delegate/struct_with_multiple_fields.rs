use delegation::delegate;

#[delegate]
struct FirstName(String, String);

#[delegate]
struct Name {
    first: String,
    last: String,
}

fn main() {
    unreachable!()
}
