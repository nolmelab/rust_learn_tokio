#[cfg(test)]
mod test {
    use std::sync::Arc;

    #[test]
    fn test_arc_usage() {
        let foo = Arc::new(vec![1, 2, 3]);
        let foo2 = foo.clone();

        assert_eq!(*foo, *foo2);
        println!("{:?}", *foo);
    }

    #[test]
    fn test_arc_deref() {
        let arc = Arc::new(vec![1, 2]);
        let _arc2 = arc.clone();

        // Deref는 [], .에서 동작한다. 아래는 &Vec<i32>에 대한 호출
        let _v = arc[0];
        let ov = arc.get(0);
        let v = ov.unwrap_or(&3);
        assert_eq!(*v, 1);
    }

    #[cfg(disabled)]
    #[test]
    fn test_arc_uninit_slic() {
        // unsafe feature들을 사용한다.
        let mut values = Arc::<[u32]>::new_uninit_slice(3);

        // Deferred initialization:
        let data = Arc::get_mut(&mut values).unwrap();
        data[0].write(1);
        data[1].write(2);
        data[2].write(3);

        let values = unsafe { values.assume_init() };

        assert_eq!(*values, [1, 2, 3])
    }

    #[test]
    fn test_arc_slice() {
        let original = &[1, 2, 3];
        let shared: Arc<[i32]> = Arc::from(&original[1..=2]);
        println!("{:?}", shared);
    }


}
