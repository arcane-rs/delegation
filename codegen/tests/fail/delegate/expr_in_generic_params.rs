use delegation::delegate;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ver<const V: u8>;

#[delegate]
trait Versioned<const V: u8> {
    fn version(&self) -> Ver<{ V }>;

    fn next_version(&self) -> Ver<{ V + 1 }>;
}

struct Created;

impl<const V: u8> Versioned<V> for Created {
    fn version(&self) -> Ver<V> {
        Ver::<V>
    }

    fn next_version(&self) -> Ver<{ V + 1 }> {
        Ver::<{ V + 1 }>
    }
}

#[delegate(derive(
    for<const V: u8> Versioned<{ V + 1 }>,
))]
enum Events {
    Create(Created),
}

fn main() {}
