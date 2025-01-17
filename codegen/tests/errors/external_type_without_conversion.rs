use delegation::{__macros::Either, delegate};

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }
}

#[delegate(derive(AsStr))]
enum EitherDef {
    Left(String),
    Right(String),
}

#[delegate(derive(AsStr))]
struct EitherString(#[delegate(as = EitherDef)] Either<String, String>);

fn main() {
    unreachable!()
}
