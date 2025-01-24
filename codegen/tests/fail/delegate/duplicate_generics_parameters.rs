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
    for<T> Named<T>
    where
        T: Named<T> + 'static,
))]
struct Wrapper<T>(T);

fn main() {}
