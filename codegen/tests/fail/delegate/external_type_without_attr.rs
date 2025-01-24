use delegation::{private::Either, delegate};

#[delegate(for(for<T: AsStr> Either<T, T>))]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

fn main() {
    unreachable!()
}
