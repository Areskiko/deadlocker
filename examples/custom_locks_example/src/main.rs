use deadlocker::Locker;
use std::sync::Arc;

type Foo = Vec<usize>;
type Bar = usize;
type Baz = u8;

pub struct CustomLock<T> {
    content: std::cell::RefCell<T>,
}

struct CustomLockGuard<'a, T> {
    content: std::cell::RefMut<'a, T>,
}

impl<'a, T> CustomLock<T> {
    fn new(item: T) -> Self {
        CustomLock {
            content: std::cell::RefCell::new(item),
        }
    }

    fn custom_lock_method(&'a self) -> CustomLockGuard<'a, T> {
        CustomLockGuard {
            content: self.content.borrow_mut(),
        }
    }
}

impl<'a, T> std::ops::Deref for CustomLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<'a, T> std::ops::DerefMut for CustomLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

#[derive(Locker)]
pub struct MyStruct {
    #[inner_type = "Foo"]
    #[lock_method = "custom_lock_method()"]
    pub foo: Arc<CustomLock<Foo>>,

    #[outer_type = "Arc<CustomLock<(.*)>>"]
    #[lock_method = "custom_lock_method()"]
    pub bar: Arc<CustomLock<Option<Vec<Bar>>>>,

    #[inner_type = "Baz"]
    #[lock_method = "custom_lock_method()"]
    pub baz: Arc<CustomLock<Baz>>,
}

pub fn main() {
    let mut my_struct = MyStruct {
        foo: Arc::new(CustomLock::new(Vec::new())),
        bar: Arc::new(CustomLock::new(Some(Vec::new()))),
        baz: Arc::new(CustomLock::new(0)),
    };

    {
        let mut lock = my_struct.locker().baz().foo().lock();
        lock.foo.push(1);
        **lock.baz = 1;
    }

    {
        let lock = my_struct.locker().bar().foo().baz().lock();

        println!("Foo: {:?}", **lock.foo);
        println!("Bar: {:?}", **lock.bar);
        println!("Baz: {:?}", **lock.baz);
    }
}
