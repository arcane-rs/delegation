use delegation_codegen::delegate;

#[delegate]
pub trait AsRefDef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate]
pub trait AsRefDef2<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate(derive(for<> AsRefDef<str> as AsRefDef2))]
pub enum Name {
    First(String),
}

fn main() {
    unreachable!()
}
