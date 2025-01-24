use delegation::delegate;

#[delegate(for(
    for<U> Case2<U>
    where
        U: Named<N> + 'static;
))]
trait Named<N> {
    fn name(&self) -> N;
}

struct User(String);
impl Named<String> for User {
    fn name(&self) -> String {
        self.0.clone()
    }
}

#[delegate(derive(
    for<N> Named<N>
    where
        U: Named<N> + 'static,
))]
enum Case1<U> {
    User(U),
}

#[delegate]
struct Case2<U>(U);

#[delegate(derive(
   Named<String>
   where
       U: Named<String> + 'static,
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

    let user2 = Case2(User("User".to_string()));
    assert_eq!(user2.name(), "User");

    let user3 = Case3::Case1(Case1::User(User("Charlie".to_string())));
    assert_eq!(user3.name(), "Charlie");
}
