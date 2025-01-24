use delegation::delegate;

#[delegate]
enum Name {
    First { first: String, last: String },
}

fn main() {}
