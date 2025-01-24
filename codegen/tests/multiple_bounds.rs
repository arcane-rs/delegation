use delegation::delegate;

#[delegate(for(
    for<U> Case2<U>
    where
        U: Named<N> + 'static,
        U: Versioned,

))]
trait Named<N> {
    fn name(&self) -> N;
}

#[delegate(for(
    for<U> Case2<U>
    where
        U: Versioned + 'static,
        U: Default,
    for<U> Case3<U>
    where
        U: Versioned + 'static,
        U: Default,
))]
trait Versioned {
    fn version(&self) -> usize;
}

#[derive(Default)]
struct User(String);
impl Named<String> for User {
    fn name(&self) -> String {
        self.0.clone()
    }
}
impl Versioned for User {
    fn version(&self) -> usize {
        self.0.len().into()
    }
}

#[delegate(derive(
    for<N> Named<N>
    where
        U: Named<N> + 'static,
        U: Versioned,
    Versioned
    where
        U: Versioned + 'static,
        U: Default,
))]
enum Case1<U> {
    User(U),
}

#[delegate]
struct Case2<U>(U);

#[delegate(derive(
   for<N> Named<N>
   where
       U: Named<N> + 'static,
       U: Versioned,
))]
enum Case3<U> {
    Case1(Case1<U>),
    #[allow(dead_code)]
    Case2(Case2<U>),
}

#[test]
fn derives_with_generics() {
    let user1 = Case1::User(User("User".to_string()));
    assert_eq!(user1.name(), "User");
    assert_eq!(user1.version(), 4);

    let user2 = Case2(User("User2".to_string()));
    assert_eq!(user2.name(), "User2");
    assert_eq!(user2.version(), 5);

    let user3 = Case3::Case1(Case1::User(User("Charlie".to_string())));
    assert_eq!(user3.name(), "Charlie");
    assert_eq!(user3.version(), 7);
    let user4 = Case3::Case2(Case2(User("Tom".to_string())));
    assert_eq!(user4.name(), "Tom");
    assert_eq!(user4.version(), 3);
}
