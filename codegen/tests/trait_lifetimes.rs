use std::fmt::Display;

use delegation::delegate;

#[delegate]
trait User<'s> {
    fn name(&'s self) -> &'s str;

    fn into_prepended_with_name<S: Display + ?Sized>(self, s: &'s S) -> String;
}

struct UserOleg;

impl<'s> User<'s> for UserOleg {
    fn name(&'s self) -> &'s str {
        "Oleg"
    }

    fn into_prepended_with_name<S: Display + ?Sized>(self, s: &'s S) -> String {
        format!("{}'s {s}", self.name())
    }
}

struct UserBoris;

impl<'s> User<'s> for UserBoris {
    fn name(&'s self) -> &'s str {
        "Boris"
    }

    fn into_prepended_with_name<S: Display + ?Sized>(self, s: &'s S) -> String {
        format!("{}'s {s}", self.name())
    }
}

#[delegate(derive(for<'s> User<'s>))]
enum Users {
    Oleg(UserOleg),
    Boris(UserBoris),
}

#[test]
fn derives_with_lifetimes() {
    let oleg = Users::Oleg(UserOleg);
    assert_eq!(oleg.name(), "Oleg");
    assert_eq!(oleg.into_prepended_with_name("apple"), "Oleg's apple");

    let boris = Users::Boris(UserBoris);
    assert_eq!(boris.name(), "Boris");
    assert_eq!(boris.into_prepended_with_name("apple"), "Boris's apple");
}
