use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

#[delegate(derive(AsStr))]
struct FirstName(String);

fn main() {
    unreachable!()
}
