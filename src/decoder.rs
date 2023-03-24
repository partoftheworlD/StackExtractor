use std::{borrow::ToOwned, slice::from_raw_parts, str::from_utf8};
use crate::extractor::Extractor;

pub trait Decoder {
    fn from_lpstr(string: *mut u8) -> String;
}

impl<T: Extractor> Decoder for T {
    // https://github.com/Traverse-Research/hassle-rs/blob/ddcdfc6032657b2b8d75d1bc55719d186ad7e55e/src/utils.rs#L34
    fn from_lpstr(string: *mut u8) -> String {
        let len = (0..)
            .take_while(|&i| unsafe { *string.offset(i) } != 0)
            .count();
        let slice: &[u8] = unsafe { from_raw_parts(string.cast(), len) };
        from_utf8(slice).map(ToOwned::to_owned).unwrap()
    }
}