#[derive(Debug)]
pub struct WriteBytes<T>(pub T);

impl<T> core::slice::SlicePattern for WriteBytes<&[T]> {
    type Item = T;

    fn as_slice(&self) -> &[Self::Item] {
        self.0
    }
}

impl<T> core::fmt::Write for WriteBytes<T>
where
    T: IntoIterator + core::iter::Extend<u8> + core::iter::FromIterator<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.extend(s.as_bytes().iter().copied());
        Ok(())
    }
}

pub fn append_prefix(prefix: &str, data: &[u8]) -> alloc::vec::Vec<u8> {
    let slice_collection: [&[u8]; 2] = [prefix[0..].as_bytes(), data];
    let output = slice_collection.join("".as_bytes());
    assert_eq!(output.len(), prefix.len() + data.len());

    output
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests {
    use super::*;
    use core::{fmt::Write, str::from_utf8};
    use std::dbg;

    #[derive(Debug, PartialEq)]
    struct FormattedString<T>(T);

    #[test]
    fn confirm_prefix() {
        let data = "Tx data".as_bytes();

        // Append text `Echo: ` before the received data.
        let prefix = "Echo: ";
        let mut writer = WriteBytes::<alloc::vec::Vec<u8>>(alloc::vec::Vec::new());
        let _ = core::write!(
            writer,
            "{}",
            alloc::string::String::from_utf8(append_prefix(prefix, &data).clone()).unwrap()
        );

        let res = alloc::string::String::from_utf8(writer.0).unwrap();
        assert_eq!(FormattedString("Echo: Tx data"), FormattedString(&*res));
    }
}
