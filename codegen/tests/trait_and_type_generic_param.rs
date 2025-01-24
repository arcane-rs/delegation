use delegation::delegate;

#[delegate]
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
        U: Named<N> + 'static;
))]
enum Case<U> {
    User(U),
}

#[test]
fn derives_with_generics() {
    let user = Case::User(User("User".to_string()));
    assert_eq!(user.name(), "User");
}
