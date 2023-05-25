# 비동기 프로그래밍이란?

비동기 프로그래밍은 마치    순차적으로 실행하는 코드 구조로 동시성을 올리는 방법입니다. Node.js, Go, Erlang 등에서 이미 많이 쓰고 있는 방법으로 Task, Future (또는 Promise), Scheduler, Executor, async / await 키워드와 컴파일러 지원으로 쉽게 사용할 수 있도록 합니다.&#x20;

비동기 프로그래밍은 동시성을 올리기위해 두 개의 분리된 처리 구조를 갖습니다. 하나는 async / await로 구조화된 Future를 포함하는 Task의 실행 구조와 대기하거나 오래 걸리는 일들을 별도로 비동기로 처리하게 하는 구조입니다. 대기하거나 오래 걸리는 작업은 OS의 비동기 처리 기능(epoll, kqueue, IOCP 등)을 사용하거나 별도의 쓰레드 풀에서 실행한 후 결과를 비동기 처리 구조에 통지하여 이어서 처리할 수 있도록 합니다.&#x20;

```rust
use mini_redis::{clients::Client, Result};

#[tokio::main]
pub async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = Client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("got value from the server; success={:?}", result.is_some());

    Ok(())
}
```

위 코드는 미니 레디스의 클라이언트 예시 코드입니다. 비동기 프로그래밍의 장점을 잘 보여줍니다.&#x20;

* 여러 async (Future)를 동시에 처리할 수 있습니다.&#x20;
  * 예시에는 한 클라이언트만 있지만 task::spawn()을 통해 여러 클라이언트를 처리하는 코드로 변환이 쉽습니다.
* 콜백 대신에 마치 순서대로 처리하는 코드처럼 실행됩니다.&#x20;
* await 부분에서 Scheduler를 통해 여러 Task를 동시에 처리할 수 있습니다.&#x20;

위 기능을 처리하는 다른 두 가지 처리 구조는 핸들러를 통해 처리하거나 콜백을 지정하거나 쓰레드를 만들어서 계속 확인하는 방법이 있습니다. 비동기 구조는 논리를 한 곳에 순차적인 코드처럼 배치할 수 있다는 것이 가장 큰 장점입니다.&#x20;

