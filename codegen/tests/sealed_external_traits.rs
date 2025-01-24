use delegation::delegate;

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;

    #[allow(dead_code)]
    fn as_prepended_string<'s>(&self, prefix: &'s str) -> String
    // TODO: Remove once https://github.com/rust-lang/rust/issues/87803
    //       is resolved.
    where
        's: 's;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }

    fn as_prepended_string<'s>(&self, prefix: &'s str) -> String
    where
        's: 's,
    {
        format!("{prefix}{self}")
    }
}

mod sealed {
    use delegation::delegate;

    use super::AsStr;

    #[delegate(as = AsRef)]
    pub trait AsRefDef<T: ?Sized> {
        fn as_ref(&self) -> &T;
    }

    #[delegate(as = AsStr)]
    pub trait AsStrDef {
        fn as_str(&self) -> &str;

        fn as_prepended_string<'s>(&self, prefix: &'s str) -> String
        // TODO: Remove once https://github.com/rust-lang/rust/issues/87803
        //       is resolved.
        where
            's: 's;
    }
}

#[delegate(derive(
    AsRef<str> as sealed::AsRefDef,
    AsStr as sealed::AsStrDef,
))]
enum Name {
    First(String),
}

#[test]
fn derives_external_trait() {
    let name = Name::First("John".to_string());
    assert_eq!(<Name as AsRef<str>>::as_ref(&name), "John");
}

#[test]
fn derives_local_trait_as_external() {
    let name = Name::First("John".to_string());
    assert_eq!(name.as_str(), "John");
}
