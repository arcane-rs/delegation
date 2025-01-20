use delegation_codegen::delegate;

#[delegate(as = AsRef)]
pub trait AsRefDef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

#[delegate(derive(AsRefDef<str>))]
pub enum Name {
    First(String),
}

fn main() {
    unreachable!()
}
