use std::fmt;


/// A structure to store bytes of data and the length of the string.
#[derive(Copy, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct Varchar<const N: usize> {
    length: usize,
    bytes: [u8; N],
}


impl<const N: usize> Varchar<N> {
    /// Creates Varchar from *str*.
    pub fn new(s: &str) -> Self {
        let s_bytes = String::from(s).into_bytes();
        let length = s_bytes.len();
        assert!(length <= N);
        let mut bytes = [0u8; N];
        bytes[..length].clone_from_slice(&s_bytes);
        Self { bytes, length }
    }
}


impl<const N: usize> fmt::Display for Varchar<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = String::from_utf8_lossy(
            &self.bytes[..self.length]
        ).to_string();
        write!(f, "{}", s)
    }
}


impl<const N: usize> fmt::Debug for Varchar<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Varchar<{}>(\"{}\")", N, self.to_string())
    }
}


#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_varchar() {
        let v = Varchar::<32>::new("varchar32");
        assert_eq!(v.to_string(), String::from("varchar32"));
        assert_eq!(mem::size_of::<Varchar::<32>>(), 40);
    }
}
