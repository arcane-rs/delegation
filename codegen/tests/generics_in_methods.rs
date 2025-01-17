use delegation::delegate;

#[delegate]
trait PrependString {
    fn prepend_string<S: Into<String>>(&self, s: S) -> String;
}

struct FirstName;

impl PrependString for FirstName {
    fn prepend_string<S: Into<String>>(&self, s: S) -> String {
        format!("Bob's {}", s.into())
    }
}

struct LastName;

impl PrependString for LastName {
    fn prepend_string<S: Into<String>>(&self, s: S) -> String {
        format!("Smith's {}", s.into())
    }
}

#[delegate(derive(PrependString))]
enum Name {
    First(FirstName),
    Last(LastName),
}

#[test]
fn derives_with_generics() {
    let first = Name::First(FirstName);
    assert_eq!(first.prepend_string("apple"), "Bob's apple");

    let last = Name::Last(LastName);
    assert_eq!(last.prepend_string("apple"), "Smith's apple");
}
