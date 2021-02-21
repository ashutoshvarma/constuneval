extern crate constuneval;

use constuneval::UnevalCow;
use std::borrow::ToOwned;
// use std::ffi::{CStr, OsStr};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

macro_rules! test_from_cow {
    ($value:ident => $($ty:ty),+) => {$(
        let borrowed = <$ty>::from(UnevalCow::Borrowed($value));
        let owned = <$ty>::from(UnevalCow::Owned($value.to_owned()));
        assert_eq!($value, &*borrowed);
        assert_eq!($value, &*owned);
    )+};
    ($value:ident : & $ty:ty) => {
        test_from_cow!($value => Box<$ty>, Rc<$ty>, Arc<$ty>);
    }
}

#[test]
fn test_from_cow_slice() {
    let slice: &[i32] = &[1, 2, 3];
    test_from_cow!(slice: &[i32]);
}

#[test]
fn test_from_cow_str() {
    let string = "hello";
    test_from_cow!(string: &str);
}

// #[test]
// fn test_from_cow_c_str() {
//     let string = CStr::from_bytes_with_nul(b"hello\0").unwrap();
//     test_from_cow!(string: &CStr);
// }

// #[test]
// fn test_from_cow_os_str() {
//     let string = OsStr::new("hello");
//     test_from_cow!(string: &OsStr);
// }

#[test]
fn test_from_cow_path() {
    let path = Path::new("hello");
    test_from_cow!(path: &Path);
}

// #[test]
// fn cow_const() {
//     // test that the methods of `Cow` are usable in a const context

//     const COW: Cow<'_, str> = Cow::Borrowed("moo");

//     const IS_BORROWED: bool = COW.is_borrowed();
//     assert!(IS_BORROWED);

//     const IS_OWNED: bool = COW.is_owned();
//     assert!(!IS_OWNED);
// }

#[test]
fn test_debug_primitive() {
    // int
    assert_eq!(
        format!("{:?}", UnevalCow::<u64>::Borrowed(&1)),
        "UnevalCow::Borrowed( &1 )"
    );
    assert_eq!(
        format!("{:?}", UnevalCow::<u64>::Owned(1)),
        "UnevalCow::Borrowed( &1 )"
    );
    // float
    assert_eq!(
        format!("{:?}", UnevalCow::<f64>::Borrowed(&3.7_f64)),
        "UnevalCow::Borrowed( &3.7 )"
    );
    assert_eq!(
        format!("{:?}", UnevalCow::<f64>::Owned(3.7_f64)),
        "UnevalCow::Borrowed( &3.7 )"
    );

    // str
    assert_eq!(
        format!("{:?}", UnevalCow::<str>::Borrowed("Hello")),
        "UnevalCow::Borrowed( \"Hello\" )"
    );

    assert_eq!(
        format!("{:?}", UnevalCow::<str>::Owned("Hello".to_string())),
        "UnevalCow::Borrowed( \"Hello\" )"
    );
}

#[test]
fn test_debug_slice() {
    // [T]
    assert_eq!(
        format!("{:?}", UnevalCow::<[u64]>::Borrowed(&[1, 2, 3])),
        "UnevalCow::Borrowed( &[1, 2, 3] )"
    );

    assert_eq!(
        format!("{:?}", UnevalCow::<[u64]>::Owned(vec![1, 2, 3])),
        "UnevalCow::Borrowed( &[1, 2, 3] )"
    );

    // Box<[T]>
    let box_u64: Box<[u64]> = Box::new([1, 2, 3]);
    assert_eq!(
        format!("{:?}", UnevalCow::<Box<[u64]>>::Borrowed(&box_u64)),
        "UnevalCow::Borrowed( &[1, 2, 3] )"
    );
    assert_eq!(
        format!("{:?}", UnevalCow::<Box<[u64]>>::Owned(box_u64)),
        "UnevalCow::Borrowed( &[1, 2, 3] )"
    );
}
