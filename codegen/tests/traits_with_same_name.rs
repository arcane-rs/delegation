use delegation::delegate;

mod first_name {
    use super::*;

    #[delegate]
    pub(super) trait AsStr {
        fn as_str(&self) -> &str;
    }

    impl AsStr for String {
        fn as_str(&self) -> &str {
            self.as_str()
        }
    }
}

mod last_name {
    use super::*;

    #[delegate]
    pub(super) trait AsStr {
        fn as_string(&self) -> &str;
    }

    impl AsStr for String {
        fn as_string(&self) -> &str {
            self.as_str()
        }
    }
}

#[delegate(derive(first_name::AsStr; last_name::AsStr))]
struct Name(String);

#[test]
fn impls_both_traits() {
    use self::{first_name::AsStr as _, last_name::AsStr as _};

    let name = Name("John".into());
    assert_eq!(name.as_str(), "John");
    assert_eq!(name.as_string(), "John");
}
