use std::fmt;


/// A structure to store bytes of data and the length.
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash, Ord)]
pub struct Bytes<const N: usize> {
    length: usize,
    bytes: [u8; N],
}


impl<const N: usize> Bytes<N> {
    /// Creates Bytes from [u8] slice.
    pub fn new(b: &[u8]) -> Self {
        let length = b.len();
        assert!(length <= N);
        let mut bytes = [0u8; N];
        bytes[..length].clone_from_slice(&b);
        Self { bytes, length }
    }
}


impl<const N: usize> fmt::Display for Bytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.bytes)
    }
}


impl<const N: usize> fmt::Debug for Bytes<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bytes<{}>{:?}", N, self.bytes)
    }
}


#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_bytes() {
        let b = Bytes::<32>::new(b"bytes32");
        assert_eq!(b.to_string(), String::from("[98, 121, 116, 101, 115, 51, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]"));
        assert_eq!(mem::size_of::<Bytes::<32>>(), 40);
    }
}
