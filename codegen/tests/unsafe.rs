use delegation::delegate;

#[delegate]
trait AsStr {
    unsafe fn as_str(&self) -> &str;
}

impl AsStr for String {
    unsafe fn as_str(&self) -> &str {
        self
    }
}

#[delegate]
unsafe trait AsString {
    fn as_string(&self) -> &str;
}

unsafe impl AsString for String {
    fn as_string(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr; AsString))]
struct Name(String);

#[test]
fn impls_trait_with_unsafe_fn() {
    use self::AsStr as _;

    let name = Name(String::from("John"));
    assert_eq!(unsafe { name.as_str() }, "John");
}

#[test]
fn impls_unsafe_trait() {
    use self::AsString as _;

    let name = Name(String::from("John"));
    assert_eq!(name.as_string(), "John");
}
