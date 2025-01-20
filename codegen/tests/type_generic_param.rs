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

#[delegate(derive(Named<String>))]
enum Case1<U>
where
    U: Named<String> + 'static,
{
    User(U),
}

#[delegate(derive(
    for<U> Named<String>
    where
        U: Named<String> + 'static;
))]
enum Case2<U> {
    Admin(U),
}

#[test]
fn derives_with_generics() {
    let user = Case1::User(User("User".to_string()));
    assert_eq!(user.name(), "User");

    let admin = Case2::Admin(User("Admin".to_string()));
    assert_eq!(admin.name(), "Admin");
}
