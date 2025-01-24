use delegation::delegate;

#[delegate]
trait PrependWith {
    fn prepend_with<'s>(&self, prefix: &'s str) -> String;
}

impl PrependWith for String {
    fn prepend_with<'s>(&self, prefix: &'s str) -> String {
        format!("{prefix}{self}")
    }
}

#[delegate(derive(PrependWith))]
enum EitherDef {
    Left(String),
    Right(String),
}

fn main() {}
