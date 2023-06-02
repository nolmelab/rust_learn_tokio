
# Hello, Tokio

매우 기본적인 Tokio 애플리케이션을 작성하여 시작하겠습니다. 이 애플리케이션은 Mini-Redis 
서버에 연결하고 키 "hello"의 값을 "world"로 설정한 다음 키를 다시 읽어올 것입니다. 이 
작업은 Mini-Redis 클라이언트 라이브러리를 사용하여 수행됩니다.

# 코드

## 새로운 크레이트 생성하기

먼저 새로운 Rust 앱을 생성합니다.

```
$ cargo new my-redis
$ cd my-redis
```

## 의존성 추가하기

다음으로, Cargo.toml 파일을 열고 [dependencies] 바로 아래에 다음을 추가합니다.

```toml
tokio = { version = "1", features = ["full"] }
mini-redis = "0.4"
```

## 코드 작성하기

그런 다음, main.rs 파일을 열고 파일의 내용을 다음으로 대체합니다.

```rust
use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
// mini-redis 주소로 연결을 엽니다.
let mut client = client::connect("127.0.0.1:6379").await?;

vbnet
Copy code
// "hello" 키에 "world" 값을 설정합니다.
client.set("hello", "world".into()).await?;

// "hello" 키를 가져옵니다.
let result = client.get("hello").await?;

println!("got value from the server; result={:?}", result);

Ok(())
}
```


Mini-Redis 서버가 실행 중인지 확인해주세요. 별도의 터미널 창에서 다음 명령을 실행하세요:

```bash
$ mini-redis-server
```

만약 아직 Mini-Redis를 설치하지 않았다면 다음 명령으로 설치할 수 있습니다:

```bash
$ cargo install mini-redis
```

이제 my-redis 애플리케이션을 실행해보세요:

```bash
$ cargo run
```

출력으로 got value from the server; result=Some(b"world")를 볼 수 있어야 합니다. 이는 
성공적으로 실행되었음을 나타냅니다.

# 나눠서 보기 

이제 우리가 방금 한 작업에 대해 자세히 살펴보겠습니다. 코드는 많지 않지만 많은 일이 
벌어집니다.

```rust
let mut client = client::connect("127.0.0.1:6379").await?;
```

client::connect 함수는 mini-redis 크레이트에서 제공됩니다. 이 함수는 지정된 원격 주소와 
비동기적으로 TCP 연결을 설정합니다. 연결이 수립되면 클라이언트 핸들이 반환됩니다. 비동기 
작업이 수행되지만 우리가 작성하는 코드는 동기적으로 보입니다. 비동기 작업임을 나타내는 
유일한 표시는 .await 연산자입니다.

## 비동기 프로그래밍이란 무엇인가요?

대부분의 컴퓨터 프로그램은 작성된 순서대로 실행됩니다. 첫 번째 줄이 실행되고, 다음 줄이 
실행되는 식으로 진행됩니다. 동기적 프로그래밍에서는 프로그램이 즉시 완료되지 않는 작업을 
만나면 작업이 완료될 때까지 차단됩니다. 예를 들어, TCP 연결 설정은 네트워크를 통해 상대와 
교환하는 작업을 필요로하며 상당한 시간이 소요될 수 있습니다. 이 시간 동안 스레드는 
차단됩니다.

비동기 프로그래밍에서는 즉시 완료될 수 없는 작업이 백그라운드로 중단됩니다. 스레드는 
차단되지 않고 다른 작업을 계속할 수 있습니다. 작업이 완료되면 작업은 중단된 위치에서 계속 
처리됩니다. 이전의 예제는 하나의 작업만 있기 때문에 중단되는 동안 아무 작업도 수행되지 
않지만, 비동기 프로그램에는 일반적으로 많은 작업이 포함됩니다.

비동기 프로그래밍은 더 빠른 애플리케이션을 만들 수 있지만, 종종 더 복잡한 프로그램을 
만들게 됩니다. 프로그래머는 비동기 작업이 완료될 때 작업을 다시 시작하기 위해 필요한 모든 
상태를 추적해야 합니다. 역사적으로 이 작업은 지루하고 오류가 발생하기 쉬운 
작업이었습니다.

## 컴파일 시 그린 쓰레드 

Rust는 async/await라는 기능을 사용하여 비동기 프로그래밍을 구현합니다. 비동기 작업을 
수행하는 함수는 async 키워드로 표시됩니다. 예제에서 connect 함수는 다음과 같이 정의됩니다:

```rust
use mini_redis::Result;
use mini_redis::client::Client;
use tokio::net::ToSocketAddrs;

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
// ...
}
```

async fn 정의는 일반적인 동기 함수와 유사하지만 비동기로 작동합니다. Rust는 컴파일 시 
async fn을 비동기로 작동하는 루틴으로 변환합니다. async fn 내에서의 .await 호출은 제어를 
스레드에게 반환합니다. 스레드는 백그라운드에서 작업이 처리되는 동안 다른 작업을 수행할 수 
있습니다.

## async/await 사용하기

비동기 함수는 다른 Rust 함수와 마찬가지로 호출됩니다. 그러나 이러한 함수를 호출하더라도 
함수 본문이 실행되지는 않습니다. 대신, async fn을 호출하면 작업을 나타내는 값을 
반환합니다. 이는 개념적으로는 인수가 없는 클로저와 유사합니다. 실제로 작업을 실행하려면 
반환 값을 사용하여 .await 연산자를 사용해야 합니다.

예를 들어, 다음과 같은 프로그램을 고려해 봅시다.

```rust
async fn say_world() {
println!("world");
}

#[tokio::main]
async fn main() {
// say_world() 호출은 say_world()의 본문을 실행하지 않습니다.
let op = say_world();

perl
Copy code
// 이 println!은 먼저 실행됩니다.
println!("hello");

// `op`에 대해 `.await`를 호출하면 `say_world`가 실행됩니다.
op.await;
}
```

결과는 다음과 같습니다:

hello
world
비동기 fn의 반환 값은 Future 트레이트를 구현한 익명 타입입니다.

## 비동기 main 함수 

애플리케이션을 실행하는 데 사용되는 main 함수는 Rust 크레이트에서 일반적으로 찾을 수 있는 
main 함수와 다릅니다.

- 이는 async fn입니다.
- #[tokio::main]으로 주석이 달려 있습니다.

비동기 함수로 진입하기 위해 async fn을 사용합니다. 그러나 비동기 함수는 런타임에 의해 
실행되어야 합니다. 런타임에는 비동기 작업 스케줄러, 이벤트 I/O, 타이머 등이 포함되어 
있습니다. 런타임은 자동으로 시작되지 않으므로 main 함수에서 시작해야 합니다.

#[tokio::main] 함수는 매크로입니다. 이 매크로는 async fn main()을 동기적으로 실행하는 
fn main()으로 변환합니다. 또한 런타임 인스턴스를 초기화하고 async main 함수를 실행합니다.

예를 들어, 다음과 같은 코드:

```rust
#[tokio::main]
async fn main() {
println!("hello");
}
```

다음과 같이 변환됩니다:

```rust
fn main() {
let mut rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
println!("hello");
})
}
```

Tokio 런타임의 자세한 내용은 나중에 다루게 됩니다.

# 카고 기능

이 튜토리얼에서 Tokio를 종속성으로 사용할 때, "full" 기능 플래그가 활성화됩니다:

```toml
tokio = { version = "1", features = ["full"] }
```

Tokio는 TCP, UDP, Unix 소켓, 타이머, 동기화 유틸리티, 다중 스케줄러 유형 등 다양한 기능을 
갖고 있습니다. 모든 애플리케이션이 모든 기능을 필요로 하는 것은 아닙니다. 컴파일 시간을 
최적화하거나 최종 애플리케이션의 크기를 줄이기 위해 애플리케이션은 필요한 기능만 활성화할 
수 있습니다.


