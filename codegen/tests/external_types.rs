use delegation::{__macros::Either, delegate};

#[delegate]
trait AsStr {
    fn as_str(&self) -> &str;

    #[allow(dead_code)]
    fn as_prepended_string<'s>(&self, prefix: &'s str) -> String
    // TODO: Remove once https://github.com/rust-lang/rust/issues/87803
    //       is resolved.
    where
        's: 's;
}

impl AsStr for String {
    fn as_str(&self) -> &str {
        self
    }

    fn as_prepended_string<'s>(&self, prefix: &'s str) -> String
    where
        's: 's,
    {
        format!("{prefix}{self}")
    }
}

#[delegate(derive(AsStr))]
enum EitherDef {
    Left(String),
    Right(String),
}

// TODO: Remove this once there is a way to impl it automatically with
//       guarantees of types equality.
impl<'a> From<&'a mut Either<String, String>> for &'a mut EitherDef {
    fn from(t: &'a mut Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "testing purposes")]
        unsafe {
            &mut *(t as *mut Either<String, String> as *mut EitherDef)
        }
    }
}

// TODO: Remove this once there is a way to impl it automatically with
//       guarantees of types equality.
impl<'a> From<&'a Either<String, String>> for &'a EitherDef {
    fn from(t: &'a Either<String, String>) -> Self {
        #[expect(unsafe_code, reason = "testing purposes")]
        unsafe {
            &*(t as *const Either<String, String> as *const EitherDef)
        }
    }
}

// TODO: Remove this once these impl will be provided by the macro.
impl From<Either<String, String>> for EitherDef {
    fn from(t: Either<String, String>) -> Self {
        match t {
            Either::Left(t) => EitherDef::Left(t),
            Either::Right(t) => EitherDef::Right(t),
        }
    }
}

#[delegate(derive(AsStr))]
struct EitherString(#[delegate(as = EitherDef)] Either<String, String>);

#[test]
fn derives_on_external_type() {
    let left = EitherString(Either::Left("left".to_string()));
    let right = EitherString(Either::Right("right".to_string()));
    assert_eq!(left.as_str(), "left");
    assert_eq!(right.as_str(), "right");
}
