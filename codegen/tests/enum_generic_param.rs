use delegation::delegate;

struct UserTypes;

const USER_OLEG: u8 = 1;

const USER_BORIS: u8 = 2;

trait UserType<const V: u8> {
    type User: User;
}

type TypedUser<const V: u8> = <UserTypes as UserType<V>>::User;

impl UserType<{ USER_OLEG }> for UserTypes {
    type User = UserOleg;
}

impl UserType<{ USER_BORIS }> for UserTypes {
    type User = UserBoris;
}

#[delegate(for(
    for<L> EitherUser<L, UserBoris>
    where
        L: User + 'static,
    GenericUser<{ USER_OLEG }, { USER_BORIS }>,
))]
trait User {
    fn name(&self) -> &str;
}

#[delegate]
enum EitherUser<L, R> {
    Left(L),
    Right { user: R },
}

#[delegate]
enum GenericUser<const U1: u8, const U2: u8>
where
    UserTypes: UserType<{ U1 }> + UserType<{ U2 }>,
{
    Left(TypedUser<{ U1 }>),
    Right { user: TypedUser<{ U2 }> },
}

struct UserOleg;

impl User for UserOleg {
    fn name(&self) -> &str {
        "Oleg"
    }
}

struct UserBoris;

impl User for UserBoris {
    fn name(&self) -> &str {
        "Boris"
    }
}

#[test]
fn derives_with_generics() {
    let oleg = EitherUser::<UserOleg, UserBoris>::Left(UserOleg);
    assert_eq!(oleg.name(), "Oleg");

    let boris = EitherUser::<UserOleg, UserBoris>::Right { user: UserBoris };
    assert_eq!(boris.name(), "Boris");
}

#[test]
fn derives_with_const_generics() {
    let oleg = GenericUser::<{ USER_OLEG }, { USER_BORIS }>::Left(UserOleg);
    assert_eq!(oleg.name(), "Oleg");

    let boris =
        GenericUser::<{ USER_OLEG }, { USER_BORIS }>::Right { user: UserBoris };
    assert_eq!(boris.name(), "Boris");
}
