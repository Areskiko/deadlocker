use deadlocker::Locker;
use std::sync::Arc;
use tokio::sync::Mutex;

type Foo = Vec<usize>;
type Bar = usize;
type Baz = u8;

#[derive(Locker)]
struct MyStruct {
    #[async_lock]
    foo: Arc<Mutex<Foo>>,
    #[async_lock]
    bar: Arc<Mutex<Bar>>,
    #[async_lock]
    baz: Arc<Mutex<Baz>>,
}

#[tokio::main]
async fn main() {
    let mut my_struct = MyStruct {
        foo: Arc::new(Mutex::new(Vec::new())),
        bar: Arc::new(Mutex::new(0)),
        baz: Arc::new(Mutex::new(0)),
    };

    {
        let mut lock = my_struct.locker().baz().foo().lock().await;
        lock.foo.push(1);
        **lock.baz = 1;
    }

    {
        let lock = my_struct.locker().bar().foo().baz().lock().await;

        println!("Foo: {:?}", **lock.foo);
        println!("Bar: {:?}", **lock.bar);
        println!("Baz: {:?}", **lock.baz);
    }
}
