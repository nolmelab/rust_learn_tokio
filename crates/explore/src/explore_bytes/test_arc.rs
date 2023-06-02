
#[cfg(test)]
mod test {
    use std::sync::Arc;

    #[test]
    fn test_arc() {
        // 아래는 Arc::new에서 Sized가 아니라고 에러를 준다.
        // Arc<[u8]>을 만들고 싶다. 쩝
        let av = Arc::from(b"Hello"[..]);
    }
}