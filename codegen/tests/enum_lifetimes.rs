#![expect(elided_named_lifetimes, reason = "testing purposes")]

use delegation::delegate;

#[delegate]
trait Named<'a> {
    fn reborrowed_name(&self) -> &str;

    fn passed_name(&'a self) -> &'a str;

    fn passed_elided_name(&'a self) -> &str;

    fn bounded_name<'b>(&'b self) -> &'b str
    where
        Self: 'b;

    fn bounded_elided_name<'b>(&'b self) -> &str
    where
        Self: 'b;
}

struct User {
    name: String,
}

impl<'a> Named<'a> for User {
    fn reborrowed_name(&self) -> &str {
        &self.name
    }

    fn passed_name(&'a self) -> &'a str {
        &self.name
    }

    fn passed_elided_name(&'a self) -> &str {
        &self.name
    }

    fn bounded_name<'b>(&'b self) -> &'b str
    where
        Self: 'b,
    {
        &self.name
    }

    fn bounded_elided_name<'b>(&'b self) -> &str
    where
        Self: 'b,
    {
        &self.name
    }
}

impl<'a> Named<'a> for &'a User {
    fn reborrowed_name(&self) -> &str {
        &self.name
    }

    fn passed_name(&'a self) -> &'a str {
        &self.name
    }

    fn passed_elided_name(&'a self) -> &str {
        &self.name
    }

    fn bounded_name<'b>(&'b self) -> &'b str
    where
        Self: 'b,
    {
        &self.name
    }

    fn bounded_elided_name<'b>(&'b self) -> &str
    where
        Self: 'b,
    {
        &self.name
    }
}

#[expect(dead_code, reason = "testing purposes")]
#[delegate(derive(for<'a> Named<'a>))]
enum CowUser<'a> {
    Borrowed(&'a User),
    Owned(User),
}

#[test]
fn newtype_derives_trait() {
    let user = User { name: "John".into() };
    let user = CowUser::Borrowed(&user);
    assert_eq!(user.reborrowed_name(), "John");
    assert_eq!(user.passed_name(), "John");
    assert_eq!(user.passed_elided_name(), "John");
    assert_eq!(user.bounded_name(), "John");
    assert_eq!(user.bounded_elided_name(), "John");
}
