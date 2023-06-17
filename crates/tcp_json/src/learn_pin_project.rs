#[cfg(test)]
mod test {
    use std::pin::Pin;
    use pin_project_lite::pin_project;

    pin_project! {
        struct Struct<T, U> {
            #[pin]
            pinned: T,
            unpinned: U
        }
    } 

    impl<T, U> Struct<T, U> {
        pub fn method(self: Pin<&mut Self>, v : U) {
            let this = self.project();
            let _ = this.pinned;
        }
    }

    #[test]
    fn test_pin_project() {
        // Box is a kind of a pointer
        let mut s = Box::pin(Struct { pinned: 3, unpinned : 5 });

        // pin은 여전히 까다롭다. 
        let ps = s.as_mut();
        ps.method(32);

        // stack에서 pin 만들기
    }

}