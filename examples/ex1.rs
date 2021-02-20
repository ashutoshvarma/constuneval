extern crate constuneval;

use constuneval::{to_string, UnevalCow};
use std::fmt;

#[derive(Debug)]
pub struct FftDomain<F>
where
    // F: 'static,
    [F]: 'static + ToOwned + fmt::Debug,
    <[F] as std::borrow::ToOwned>::Owned: fmt::Debug,
{
    pub some_table: UnevalCow<'static, [UnevalCow<'static, [F]>]>,
}

fn main() {
    let fft_temp = FftDomain {
        some_table: UnevalCow::Owned(vec![
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
            UnevalCow::Owned(vec![1, 2, 3, 4, 5]),
        ]),
    };
    println!(
        "{}",
        to_string("FFT_TABLE", &fft_temp, Some("FftDomain<'static, i32>"))
    );
}
