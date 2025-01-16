use delegation::delegate;

#[delegate]
trait AsString: Sized {
    fn into_string(self) -> String {
        "default impl".into()
    }
    fn as_str(&self) -> &str {
        "default impl"
    }
    fn as_mut_str(&mut self) -> String {
        "default impl".into()
    }
}

impl AsString for String {
    fn into_string(self) -> String {
        self
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }

    fn as_mut_str(&mut self) -> String {
        self.clone()
    }
}

impl AsString for () {}

#[delegate(derive(AsString))]
enum Name {
    First(FirstName),
    Last { name: LastName },
}

#[delegate(derive(AsString))]
struct FirstName(String);

#[delegate(derive(AsString))]
struct LastName {
    name: (),
}

#[test]
fn newtype_derives_trait() {
    let mut first_name = FirstName("John".to_string());
    assert_eq!(first_name.as_str(), "John");
    assert_eq!(first_name.as_mut_str(), "John");
    assert_eq!(first_name.into_string(), "John");

    let mut last_name = LastName { name: () };
    assert_eq!(last_name.as_str(), "default impl");
    assert_eq!(last_name.as_mut_str(), "default impl");
    assert_eq!(last_name.into_string(), "default impl");
}

#[test]
fn enum_derives_trait() {
    let mut first_name = Name::First(FirstName("John".to_string()));
    assert_eq!(first_name.as_str(), "John");
    assert_eq!(first_name.as_mut_str(), "John");
    assert_eq!(first_name.into_string(), "John");

    let mut last_name = Name::Last {
        name: LastName { name: () },
    };
    assert_eq!(last_name.as_str(), "default impl");
    assert_eq!(last_name.as_mut_str(), "default impl");
    assert_eq!(last_name.into_string(), "default impl");
}
