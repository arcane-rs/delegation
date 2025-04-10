use delegation_codegen::delegate;

#[delegate]
pub trait AsRefDef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate(derive(AsRef<str> as AsRefDef))]
pub enum Name {
    First(String),
}

fn main() {}
