# as-method

```rust
mod a {
    use as_method::as_method;

    #[as_method]
    pub fn foo<T: std::fmt::Debug>(x: impl std::fmt::Debug, y: T) {
        println!("{x:?}, {y:?}");
    }
}

use a::foo;

fn main() {
    1.foo(2);
}
```
