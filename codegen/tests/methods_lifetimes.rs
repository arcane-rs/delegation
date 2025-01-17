use delegation::delegate;

#[delegate]
trait User<'name> {
    fn name(&'name self) -> &'name str;

    fn prepended_with_name(&'_ self, s: &'_ str) -> String;

    fn prepend_with_name<'s>(&'s self, s: &'s mut String) -> &'s mut String;

    fn prepend_name_to<'s>(&self, s: &'s mut String)
    // TODO: Remove once https://github.com/rust-lang/rust/issues/87803
    //       is resolved.
    where
        's: 's;

    fn into_prepended_with_name<'a>(self, s: &'a str) -> String
    // TODO: Remove once https://github.com/rust-lang/rust/issues/87803
    //       is resolved.
    where
        'a: 'a;
}

struct UserOleg;

impl<'name> User<'name> for UserOleg {
    fn name(&'name self) -> &'name str {
        "Oleg"
    }

    fn prepended_with_name(&'_ self, s: &'_ str) -> String {
        format!("{}'s {s}", self.name())
    }

    fn prepend_with_name<'s>(&self, s: &'s mut String) -> &'s mut String {
        s.insert_str(0, &format!("{}'s ", self.name()));
        s
    }

    fn prepend_name_to<'s>(&self, s: &'s mut String)
    where
        's: 's,
    {
        s.insert_str(0, &format!("{}'s ", self.name()));
    }

    fn into_prepended_with_name<'a>(self, s: &'a str) -> String
    where
        'a: 'a,
    {
        format!("{}'s {s}", self.name())
    }
}

struct UserBoris;

impl<'name> User<'name> for UserBoris {
    fn name(&'name self) -> &'name str {
        "Boris"
    }

    fn prepended_with_name(&'_ self, s: &'_ str) -> String {
        format!("{}'s {s}", self.name())
    }

    fn prepend_with_name<'s>(&self, s: &'s mut String) -> &'s mut String {
        s.insert_str(0, &format!("{}'s ", self.name()));
        s
    }

    fn prepend_name_to<'s>(&self, s: &'s mut String)
    where
        's: 's,
    {
        s.insert_str(0, &format!("{}'s ", self.name()));
    }

    fn into_prepended_with_name<'a>(self, s: &'a str) -> String
    where
        'a: 'a,
    {
        format!("{}'s {s}", self.name())
    }
}

#[delegate(derive(for<'name> User<'name>))]
enum Users {
    Oleg(UserOleg),
    Boris(UserBoris),
}

#[test]
fn derives_with_lifetimes() {
    let oleg = Users::Oleg(UserOleg);
    assert_eq!(oleg.name(), "Oleg");
    assert_eq!(oleg.prepended_with_name("apple"), "Oleg's apple");
    assert_eq!(
        oleg.prepend_with_name(&mut String::from("orange")),
        "Oleg's orange",
    );

    let mut orange = String::from("orange");
    oleg.prepend_name_to(&mut orange);
    assert_eq!(orange, "Oleg's orange");
    assert_eq!(oleg.into_prepended_with_name("apple"), "Oleg's apple");

    let boris = Users::Boris(UserBoris);
    assert_eq!(boris.name(), "Boris");
    assert_eq!(boris.prepended_with_name("apple"), "Boris's apple");
    assert_eq!(
        boris.prepend_with_name(&mut String::from("orange")),
        "Boris's orange",
    );

    let mut orange = String::from("orange");
    boris.prepend_name_to(&mut orange);
    assert_eq!(orange, "Boris's orange");
    assert_eq!(boris.into_prepended_with_name("apple"), "Boris's apple");
}
