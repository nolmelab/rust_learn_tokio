#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    #[test]
    fn test_bytes_mut_basic_usage() {
        let mut buf = BytesMut::with_capacity(64);

        buf.put_u8(b'h');
        buf.put_u8(b'e');
        buf.put(&b"llo"[..]);

        assert_eq!(&buf[..], b"hello");

        // Freeze the buffer so that it can be shared
        let a = buf.freeze();

        // This does not allocate, instead `b` points to the same memory.
        let b = a.clone();

        assert_eq!(&a[..], b"hello");
        assert_eq!(&b[..], b"hello");
    }

    #[test]
    fn test_with_capacity() {
      let mut bytes = BytesMut::with_capacity(1024);
      assert!(bytes.len() == 1024);

      bytes.put(&b"hello"[..]);

      // Bytes, BytesMut, Buf, BufMut는 slice 친화적이다
      let vs = vec![1_u8, 2, 3];
      bytes.put(&vs[0..]);
    }

    #[test]
    fn test_freeze() {
      let b = BytesMut::with_capacity(128);
      let buf = b.freeze();
      let buf2 = buf.clone();

      assert_eq!(buf, buf2);
    }

    #[test]
    fn test_split_off() {
      let mut buf = BytesMut::with_capacity(128);
      buf.put(&b"hello world"[..]);

      // split_off()에서 두 개의 BytesMut는 원래 메모리를 공유한다. 
      // 위치 차이로 서로 접근하지 않도록 한다.  
      let last = buf.split_off(5);
      let tail = &last[..];
      let head = &buf[..];
      assert_eq!(tail, b" world");
      assert_eq!(head, b"hello");
    }

    #[test]
    fn test_split_to() {
      // 내가 앞을 갖고, 뒤를 돌려준다.
      // 위 함수의 반복이다. 
    }
}
