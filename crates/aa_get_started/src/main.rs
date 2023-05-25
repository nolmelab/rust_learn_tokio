use futures::executor::block_on;

async fn hello() {
    println!("immediate");
}

fn main() {
    let future = hello();
    block_on(future);
}
