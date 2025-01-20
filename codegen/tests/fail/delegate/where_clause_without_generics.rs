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
    Named<String>
    where
        U: Named<String> + 'static;
))]
enum Profile<U> {
    Admin(U),
    User(U),
}

fn main() {
    unreachable!()
}
