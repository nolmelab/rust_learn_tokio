use tokio::task::{ self, JoinHandle };
use tokio::time::{ sleep, Duration };
use std::rc::Rc;

#[tokio::main]
async fn main() {
    let mut handles = Vec::<Rc<JoinHandle<()>>>::new();

    for i in 0..10 {
        let handle = task::spawn(async move {
            let v = hello(i).await;
            println!("{v}");
        });
        handles.push(Rc::new(handle));
    }

    for handle in handles.iter() {
        // JoinHandle의 RawTask의 Header를 통해 파악한다. 
        // 이 메모리는 언제까지 유효한가? 어디서 RawTask를 Drop하는가?
        if !handle.is_finished() {
            sleep(Duration::from_millis(10)).await;
        }
    }
}

async fn hello(i: i32) -> String {
    format!("Hello {i}").into()
}
