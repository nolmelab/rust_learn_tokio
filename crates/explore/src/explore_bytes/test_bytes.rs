
#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut, Buf, BufMut};

    #[test]
    fn test_bytes_from_static() {
        let b = Bytes::from_static(b"hello");
        assert_eq!(&b[..], b"hello");
    }
}