use delegation::delegate;

#[delegate(for(Name; FirstName; LastName))]
trait AsString {
    fn into_string(self) -> String;
    fn as_str(&self) -> &str;
    fn as_mut_str(&mut self) -> &mut String;
}

impl AsString for String {
    fn into_string(self) -> String {
        self
    }

    fn as_str(&self) -> &str {
        self
    }

    fn as_mut_str(&mut self) -> &mut String {
        self
    }
}

#[delegate]
enum Name {
    First(FirstName),
    Last { name: LastName },
}

#[delegate]
struct FirstName(String);

#[delegate]
struct LastName {
    name: String,
}

#[test]
fn impls_trait_for_newtype() {
    let mut first_name = FirstName("John".into());
    *first_name.as_mut_str() = "Jane".into();
    assert_eq!(first_name.as_str(), "Jane");
    assert_eq!(first_name.into_string(), "Jane");

    let mut last_name = LastName { name: "Doe".into() };
    *last_name.as_mut_str() = "Smith".into();
    assert_eq!(last_name.as_str(), "Smith");
    assert_eq!(last_name.into_string(), "Smith");
}

#[test]
fn impls_trait_for_enum() {
    let mut first_name = Name::First(FirstName("John".to_string()));
    *first_name.as_mut_str() = "Jane".to_string();
    assert_eq!(first_name.as_str(), "Jane");
    assert_eq!(first_name.into_string(), "Jane");

    let mut last_name = Name::Last {
        name: LastName { name: "Doe".into() },
    };
    *last_name.as_mut_str() = "Smith".into();
    assert_eq!(last_name.as_str(), "Smith");
    assert_eq!(last_name.into_string(), "Smith");
}
