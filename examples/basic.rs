use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
enum Name {
    First(FirstName),
    Last { name: LastName },
}

#[delegate(derive(AsStr))]
struct FirstName(String);

#[delegate(derive(AsStr))]
struct LastName {
    name: String,
}

fn main() {
    let john = Name::First(FirstName("John".into()));
    assert_eq!(john.as_str(), "John");

    let smith = Name::Last {
        name: LastName {
            name: "Smith".into(),
        },
    };
    assert_eq!(smith.as_str(), "Smith");
}
