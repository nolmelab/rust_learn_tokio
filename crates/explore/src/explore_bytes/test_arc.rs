
#[cfg(test)]
mod test {
    use std::sync::Arc;

    #[test]
    fn test_arc_1() {
        let foo = Arc::new(vec![1, 2, 3]);
        let foo2 = foo.clone();

        assert_eq!(*foo, *foo2);

        println!("{:?}", *foo);
    }
}