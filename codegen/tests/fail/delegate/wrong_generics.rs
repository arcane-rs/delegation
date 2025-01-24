use delegation::delegate;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ver<const V: u8>;

#[delegate]
trait Versioned<const V: u8> {
    fn version(&self) -> Ver<{ V }>;
}

struct Created;

impl<const V: u8> Versioned<V> for Created {
    fn version(&self) -> Ver<V> {
        Ver::<V>
    }
}

#[delegate(derive(
    for<'a> Versioned<'a>,
))]
enum Events {
    Create(Created),
}

fn main() {}
