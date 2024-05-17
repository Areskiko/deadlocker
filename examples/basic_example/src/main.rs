use deadlocker::Locker;
use std::sync::{Arc, Mutex};

type Foo = Vec<usize>;
type Bar = usize;
type Baz = u8;

#[derive(Locker)]
pub struct MyStruct {
    #[result]
    pub foo: Arc<Mutex<Foo>>,
    #[result]
    pub bar: Arc<Mutex<Bar>>,
    #[result]
    pub baz: Arc<Mutex<Baz>>,
}

pub fn main() {
    let mut my_struct = MyStruct {
        foo: Arc::new(Mutex::new(Vec::new())),
        bar: Arc::new(Mutex::new(0)),
        baz: Arc::new(Mutex::new(0)),
    };

    {
        let mut lock = my_struct
            .locker()
            .baz()
            .foo()
            .lock()
            .expect("Mutex was poisoned");
        lock.foo.push(1);
        **lock.baz = 1;
    }

    {
        let lock = my_struct
            .locker()
            .bar()
            .foo()
            .baz()
            .lock()
            .expect("Mutex was poisoned");

        println!("Foo: {:?}", **lock.foo);
        println!("Bar: {:?}", **lock.bar);
        println!("Baz: {:?}", **lock.baz);
    }
}
