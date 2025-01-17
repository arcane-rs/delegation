use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

#[delegate(derive(AsStr))]
enum Name {
    First(String),
}

fn main() {
    let name = Name::First("John".to_string());
    assert_eq!(name.as_str(), "John");
}
