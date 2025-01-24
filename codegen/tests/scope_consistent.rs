#[derive(Clone, Copy)]
struct MyStruct;

mod a {
    #[delegation::delegate]
    pub(super) trait ToMyStruct<A>
    where
        A: Into<super::MyStruct>,
    {
        fn to_my_struct(&self) -> super::MyStruct;

        fn type_to_my_struct<T>(&self, t: T) -> super::MyStruct
        where
            T: Into<super::MyStruct>;
    }

    #[delegation::delegate]
    pub(super) trait ToGeneric<'a, A> {
        fn to_generic(&'a self) -> A;

        fn to_generic_ref<'g>(&'a self) -> &'g A
        where
            'a: 'g;
    }
}

mod b {
    #[delegation::delegate(derive(
        super::a::ToMyStruct<Inner>
        where
            Inner: Copy + Into<super::MyStruct> + 'static;
        for<'a> super::a::ToGeneric<'a, Inner>
        where
            Inner: Copy + 'static,
    ))]
    pub(super) struct Wrapper<Inner>(pub(super) B<Inner>);

    pub(super) struct B<Inner>(pub(super) Inner);

    impl<Inner> super::a::ToMyStruct<Inner> for B<Inner>
    where
        Inner: Copy + Into<super::MyStruct>,
    {
        fn to_my_struct(&self) -> super::MyStruct {
            self.0.into()
        }

        fn type_to_my_struct<T>(&self, t: T) -> super::MyStruct
        where
            T: Into<super::MyStruct>,
        {
            t.into()
        }
    }

    impl<'a, Inner> super::a::ToGeneric<'a, Inner> for B<Inner>
    where
        Inner: Copy,
    {
        fn to_generic(&self) -> Inner {
            self.0
        }

        fn to_generic_ref<'g>(&'a self) -> &'g Inner
        where
            Inner: 'g,
            'a: 'g,
        {
            &self.0
        }
    }
}

#[test]
fn keeps_correct_trait_scope() {
    use a::ToMyStruct as _;

    let a = b::Wrapper(b::B(MyStruct));
    a.to_my_struct();
    a.type_to_my_struct(MyStruct);
    <b::Wrapper<MyStruct> as a::ToGeneric<'_, MyStruct>>::to_generic_ref(&a);
    <b::Wrapper<MyStruct> as a::ToGeneric<'_, MyStruct>>::to_generic(&a);
}
