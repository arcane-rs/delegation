use delegation::delegate;

trait Name {}

impl Name for String {}

#[delegate]
trait Named<N>
where
    N: Name,
{
    fn name(&self) -> N;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ver<const V: u8>;

#[delegate]
trait Versioned<const V: u8> {
    fn version(&self) -> String;

    fn version_num(&self) -> Ver<{ V }>;
}

#[delegate(derive(
    Named<String>;
    for<const V: u8> Versioned<{ V }>;
))]
enum Users {
    Oleg(UserOleg),
    Boris { user: UserBoris },
}

struct UserOleg(String);

impl Named<String> for UserOleg {
    fn name(&self) -> String {
        self.0.clone()
    }
}

impl<const V: u8> Versioned<V> for UserOleg {
    fn version(&self) -> String {
        format!("UserOleg v{V}")
    }

    fn version_num(&self) -> Ver<V> {
        Ver::<V>
    }
}

struct UserBoris {
    name: String,
}

impl Named<String> for UserBoris {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<const V: u8> Versioned<V> for UserBoris {
    fn version(&self) -> String {
        format!("UserBoris v{V}")
    }

    fn version_num(&self) -> Ver<V> {
        Ver::<V>
    }
}

#[test]
fn derives_with_generics() {
    let oleg = Users::Oleg(UserOleg("Oleg".to_string()));
    assert_eq!(oleg.name(), "Oleg");

    let boris = Users::Boris { user: UserBoris { name: "Boris".to_string() } };
    assert_eq!(boris.name(), "Boris");
}

#[test]
fn derives_with_const_generics() {
    let oleg = Users::Oleg(UserOleg("Oleg".to_string()));
    assert_eq!(<Users as Versioned<2>>::version(&oleg), "UserOleg v2");
    assert_eq!(oleg.version_num(), Ver::<2>);

    let boris = Users::Boris { user: UserBoris { name: "Boris".to_string() } };
    assert_eq!(<Users as Versioned<2>>::version(&boris), "UserBoris v2");
    assert_eq!(boris.version_num(), Ver::<2>);
}
