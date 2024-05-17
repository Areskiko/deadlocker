# Deadlocker

Dealocker is a crate that aims to eliminate deadlocks in a builder-pattern
inspired manner

---

The main feature of the crate is the derive macro which will do a number of
things:

1. Create one struct for each combination of locks that may be held
2. Create a struct for holding references to the locks in the original struct
3. Allow the locker struct to chain methods to specify desired locks in any
   order
4. Implement a final lock method on the locker struct which will lock the
   desired locks in a deterministic order, eliminating deadlocks caused by
   out-of-order acquisition of locks

## Example

```rust
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
```

## Attributes

Each field may be annotated with a number of attributes to modify the behaviour
of the derive macro. The effects are noted below, and examples are provided in
the [examples directory](examples)

### outer_type
Indicates what is to be the outer type, i.e. what is the locking part, as
opposed to the [inner_type](#inner_type). This is the complement to [inner_type](#inner_type), it
only makes sense to specify one of them, and [inner_type](#inner_type) takes precedence.

It is specified using a regex with a single capture group where the
[inner_type](#inner_type) is supposed to be. Specifying the outer type is mostly useful
when the inner type is long and complex.

```rust
#[outer_type = "Arc<Mutex<(.*)>>"]
```

Note that this is by default set to the above, meaning that if your field
conforms to the pattern you needn't specify anything. See the
[basic example](examples/basic_example) for more.

### inner_type

Indicates what is the be the inner type, i.e. what is being guarded by the lock.
This is the complement to [outer_type](#outer_type), it only makes sense to specify one of
them, and this takes precedence over [outer_type](#outer_type)

```rust
#[inner_type = "usize"]
```

### is_async

Marks the lock as async, this causes the final `lock` method in a chain
containing an `is_async` marked field to become asynchronous as well. This
does not affect other locks in the struct, so the final `lock` method needn't be
asynchronous just because some of the locks are.

```rust
#[is_async]
```

### lock_method

Indicates how to get at the [inner_type](#inner_type) from the lock. For
`std::sync::Mutex` this is `lock()`, and for `tokio::sync::Mutex` it is
`lock().await`. Note the omission of the leading period, as well as the trailing
semicolon.

```rust
#[lock_method = "lock()"]
```

For synchronous locks this defaults to `lock()`, and to `lock().await` for
asynchronous ones, meaning most people won't have to specify this.


### result

Marks the lock as yielding a result over its guard, rather than the guard
directly. This is true for `std::sync::Mutex` but not for `tokio::sync::Mutex`.
This causes the final `lock` method in a chain containing a `result` marked
field to return a result, with the normally returned struct embedded in its `Ok`
variant.

```rust
#[result]
```

### include

Indicates that this field should be included in the locker struct. The presence
of a single field marked as such implies that no field without such a mark should
be included. Useful if you have a large state struct where only a small portion
are locks. This overrides any [exclude](#exclude) attribute.

```rust
#[include]
```

### exclude

Indicates that this field should not be included in the locker struct. Useful if
you have a large state struct where most of the fields are locked.

```rust
#[exclude]
```
