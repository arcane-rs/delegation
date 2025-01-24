use delegation::delegate;

#[delegate(as = AsRef)]
trait AsRefDef<T: ?Sized> {
    fn as_ref(&mut self) -> &mut T;
}

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(as = AsStr)]
trait AsStrDef {
    fn as_str(&self) -> &str;
}

#[delegate(derive(
    AsRef<str> as AsRefDef,
    AsStr as AsStrDef,
))]
enum Name {
    First(String),
}

fn main() {}
