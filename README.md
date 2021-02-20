# constuneval

Try to serializes your data/tables to const rust code using `Debug` trait.

## Why?
This crate was inspired by the this
[Github issue](https://github.com/not-yet-awesome-rust/not-yet-awesome-rust/issues/93).

## Usage
This crate can be used form your build script. It will try to serialize data/tables you provide to any file you specify. After that you can use [include!](https://doc.rust-lang.org/stable/std/macro.include.html)
to embed the generated code into your crate.

For full documentation see - https://docs.rs/constuneval

## Credit
[uneval](https://github.com/Cerberuser/uneval) and [@burdges](https://github.com/burdges)

## License
MIT