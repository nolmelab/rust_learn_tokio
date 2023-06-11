# Spawning

저희는 방향을 전환하여 Redis 서버 작업에 대해 작업을 시작하겠습니다.

먼저, 이전 섹션에서 클라이언트 SET/GET 코드를 예제 파일로 이동시킵니다. 이렇게 하면 서버에 
대해 실행할 수 있습니다.

```bash
$ mkdir -p examples
$ mv src/main.rs examples/hello-redis.rs
```

그런 다음, 새로운 비어 있는 src/main.rs를 생성하고 계속 진행합니다.

## 소켓 수락

Redis 서버에서 가장 먼저 해야 할 일은 들어오는 TCP 소켓을 수락하는 것입니다. 이 작업은 
tokio::net::TcpListener를 6379 포트에 바인딩하여 수행됩니다.

Tokio의 많은 타입들은 Rust 표준 라이브러리의 동기 버전과 동일한 이름을 가지고 있습니다. 이 
경우에는 Tokio가 async fn을 사용하여 동일한 API를 제공합니다.

그런 다음 소켓은 루프 내에서 수락됩니다. 각 소켓은 처리된 후에 닫힙니다. 우선은 명령을 
읽고 stdout에 출력한 다음 에러로 응답하는 방식입니다.

```rust
use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};

#[tokio::main]
async fn main() {
    // 주소에 리스너를 바인딩합니다.
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        // 두 번째 아이템에는 새로운 연결의 IP와 포트가 포함됩니다.
        let (socket, _) = listener.accept().await.unwrap();
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    // `Connection`은 바이트 스트림 대신 레디스 **프레임**을 읽고 쓸 수 있게 해줍니다.
    // `Connection` 타입은 mini-redis에 의해 정의됩니다.
    let mut connection = Connection::new(socket);

    if let Some(frame) = connection.read_frame().await.unwrap() {
        println!("GOT: {:?}", frame);

        // 에러로 응답합니다.
        let response = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response).await.unwrap();
    }
}
```


이제 소켓 수락 루프를 실행해 보겠습니다.

```bash
$ cargo run
```

별도의 터미널 창에서 hello-redis 예제를 실행합니다(이전 섹션의 SET/GET 명령):

```bash
$ cargo run --example hello-redis
```

출력은 다음과 같아야 합니다:

```bash
Error: "unimplemented"
```

서버 터미널에서의 출력은 다음과 같습니다:

```bash
GOT: Array([Bulk(b"set"), Bulk(b"hello"), Bulk(b"world")])
```

# 동시성 

서버에는 오직 오류로 응답하는 문제 외에도 약간의 문제가 있습니다. 현재는 수신된 요청을 한 
번에 하나씩 처리합니다. 소켓이 수락되면 서버는 응답이 소켓에 완전히 쓰여질 때까지 수락 
루프 블록 안에 남아 있습니다.

우리의 Redis 서버는 여러 동시 요청을 처리하도록 하려고 합니다. 이를 위해 일부 동시성을 
추가해야 합니다.

동시성과 병렬성은 동일한 개념이 아닙니다. 두 가지 작업을 번갈아 가며 처리한다면, 동시에 
작업을 수행하지만 병렬적으로는 작업하지 않습니다. 병렬 작업을 위해서는 두 명의 사람이 
필요합니다.

Tokio를 사용하는 장점 중 하나는 비동기 코드를 사용하면 많은 작업을 병렬적으로 처리하지 
않고도 동시에 많은 작업을 수행할 수 있다는 점입니다. 사실, Tokio는 단일 스레드에서 많은 
작업을 동시에 실행할 수 있습니다!

연결을 동시에 처리하기 위해 들어오는 각 연결에 대해 새로운 작업이 생성됩니다. 연결은 해당 
작업에서 처리됩니다.

수락 루프는 다음과 같이 변경됩니다:

```rust
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // 각 수신 소켓에 대해 새 작업이 생성됩니다. 소켓은
        // 새 작업으로 이동되어 해당 작업에서 처리됩니다.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}
```

## Task

Tokio의 작업(Task)은 비동기적인 그린 스레드입니다. async 블록을 tokio::spawn에 전달하여 
생성됩니다. tokio::spawn 함수는 생성된 작업과 상호작용하기 위해 사용할 수 있는 
JoinHandle을 반환합니다. async 블록은 반환값을 가질 수 있습니다. 호출자는 JoinHandle에 
.await을 사용하여 반환값을 얻을 수 있습니다.

예를 들어:

```rust
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        // 비동기 작업 수행
        "반환 값"
    });

    // 다른 작업 수행

    let out = handle.await.unwrap();
    println!("받았다: {}", out);
}
```

JoinHandle에서의 대기는 Result를 반환합니다. 작업이 실행 중에 오류가 발생하면 JoinHandle은 
Err를 반환합니다. 이는 작업이 패닉을 일으키거나 런타임이 강제로 취소될 때 발생합니다.

작업은 스케줄러에 의해 관리되는 실행 단위입니다. 작업을 생성하면 Tokio 스케줄러에 
제출되며, 스케줄러는 작업이 수행할 작업이 있는 경우 해당 작업이 실행되도록 보장합니다. 
생성된 작업은 생성된 스레드와 동일한 스레드에서 실행될 수도 있고, 다른 런타임 스레드에서 
실행될 수도 있습니다. 또한 작업은 생성된 후에 스레드 간에 이동될 수도 있습니다.


Tokio에서의 작업은 매우 가벼운 특징을 가지고 있습니다. 내부적으로는 단일 할당과 64바이트의 
메모리만을 필요로 합니다. 애플리케이션은 수천 개에서 수백만 개의 작업을 자유롭게 생성할 수
있습니다.

## 'static 바운드('static bound)

Tokio 런타임에서 작업을 생성할 때, 해당 작업의 타입은 수명(lifetime)이 'static이어야 
합니다. 이는 생성된 작업이 작업 외부에서 소유한 데이터에 대한 참조를 포함해서는 안 된다는 
것을 의미합니다.

보통 'static은 항상 "영원히 지속된다"는 의미로 오해되지만, 이는 사실이 아닙니다. 값이 
'static이라고 해서 반드시 메모리 누수가 있는 것은 아닙니다. Common Rust Lifetime 
Misconceptions에서 자세히 알아볼 수 있습니다.

예를 들어, 다음 코드는 컴파일되지 않습니다:

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let v = vec![1, 2, 3];

    task::spawn(async {
        println!("Here's a vec: {:?}", v);
    });
}
```

위 코드를 컴파일하려고 하면 다음과 같은 오류가 발생합니다:

```rust
error[E0373]: async block may outlive the current function, but
              it borrows `v`, which is owned by the current function
 --> src/main.rs:7:23
  |
7 |       task::spawn(async {
  |  _______________________^
8 | |         println!("Here's a vec: {:?}", v);
  | |                                        - `v` is borrowed here
9 | |     });
  | |_____^ may outlive borrowed value `v`
  |
note: function requires argument type to outlive `'static`
 --> src/main.rs:7:17
  |
7 |       task::spawn(async {
  |  _________________^
8 | |         println!("Here's a vec: {:?}", v);
9 | |     });
  | |_____^
help: to force the async block to take ownership of `v` (and any other
      referenced variables), use the `move` keyword
  |
7 |     task::spawn(async move {
8 |         println!("Here's a vec: {:?}", v);
9 |     });
  |
  ```

이러한 에러가 발생하는 이유는 async 블록이 현재 함수보다 오래 지속될 수 있으며, 이 블록이 
현재 함수에서 소유한 v를 대여(borrow)하기 때문입니다. 이 문제를 해결하기 위해 move 
키워드를 사용하여 async 블록이 v를 소유하도록 변경할 수 있습니다.

이는 기본적으로 변수가 async 블록으로 이동되지 않기 때문에 발생합니다. v 벡터는 여전히 
main 함수에 의해 소유됩니다. println! 라인은 v를 대여합니다. Rust 컴파일러는 친절하게도 
이를 설명해주며 문제를 해결하기 위한 제안까지 제시합니다! 7번째 라인을 
task::spawn(async move {로 변경하면 컴파일러에게 v를 생성된 작업으로 이동하도록 
지시합니다. 이제 작업은 모든 데이터를 소유하므로 'static으로 만들어집니다.

하나의 데이터가 동시에 여러 작업에서 액세스해야 하는 경우 Arc와 같은 동기화 원시 타입을 
사용하여 공유해야 합니다.

오류 메시지에서는 인수 유형이 'static 수명을 초과한다고 언급합니다. 이 용어는 혼동스러울 
수 있습니다. 왜냐하면 'static 수명은 프로그램의 끝까지 지속되기 때문에 수명을 초과한다면 
메모리 누수가 발생하지 않을까요? 그 해석은 값이 아닌 유형이 'static 수명을 초과해야 한다는 
것이며, 값은 유형이 더 이상 유효하지 않을 때 파괴될 수 있습니다.

값이 'static인 경우 단지 해당 값이 영원히 유지되는 것이 부적절하지 않다는 것을 의미합니다. 
이것은 중요합니다. 왜냐하면 컴파일러는 새롭게 생성된 작업이 얼마 동안 유지될지 추론할 수 
없기 때문입니다. 작업이 필요한 만큼 실행될 수 있도록 작업이 영원히 존속할 수 있도록 해야 
합니다.

이전에 언급한 기사에서는 "'static에 바운드됨"이라는 용어를 사용하여 T: 'static을 
의미하는데, 이는 인수의 유형이 'static 수명 동안 유효해야 한다는 것을 의미합니다. 
이는 값 자체가 'static 수명으로 주석이 달려 있다는 것과 다릅니다. 여기서 값 자체가 
'static 수명을 갖습니다.

값이 'static인 경우는 'static lifetime 동안 유효하다는 의미이고, T : 'static으로 타잎이 
'static이라는 뜻은 'static lifetime 동안 유효할 수 있다는 뜻으로 drop 되지 않는다는 뜻이 
아니다. 

## Send 제약 (bound)

tokio::spawn에 의해 생성된 작업은 Send를 구현해야 합니다. 이는 작업이 .await에서 일시 
중단되는 동안 Tokio 런타임이 작업을 스레드 간에 이동할 수 있게 합니다.

작업은 모든 .await 호출을 거칠 때 유지되는 데이터가 모두 Send여야 합니다. 이는 약간 
까다로울 수 있습니다. .await가 호출되면 작업은 스케줄러에게 양보합니다. 작업이 다음에 
실행될 때는 마지막으로 양보한 지점부터 다시 시작됩니다. 이를 가능하게 하려면 .await 이후에
사용되는 모든 상태는 작업에 의해 저장되어야 합니다. 이 상태가 Send인 경우, 즉 스레드 간에 
이동될 수 있는 경우 작업 자체도 스레드 간에 이동될 수 있습니다. 반대로 상태가 Send가 아닌 
경우, 작업 자체도 Send가 아닙니다.

예를 들어, 다음 코드는 작동합니다:

```rust
use tokio::task::yield_now;
use std::rc::Rc;

#[tokio::main]
async fn main() {
tokio::spawn(async {
  // 스코프는 .await 이전에 rc가 해제되도록 합니다.
  {
    let rc = Rc::new("hello");
    println!("{}", rc);
  }

    // `rc`는 더 이상 사용되지 않습니다. 작업이 스케줄러에게 양보될 때
    // **지속되지 않습니다**.
    yield_now().await;
});
}
```

하지만 다음 코드는 작동하지 않습니다:

```rust
use tokio::task::yield_now;
use std::rc::Rc;

#[tokio::main]
async fn main() {
tokio::spawn(async {
    let rc = Rc::new("hello");

    // `.await` 이후에 `rc`가 사용됩니다. 작업의 상태로
    // **지속되어야 합니다**.
    yield_now().await;

    println!("{}", rc);
});
}
```

해당 코드를 컴파일하려고 하면 다음과 같은 오류가 발생합니다:

```rust
error: future cannot be sent between threads safely
--> src/main.rs:6:5
|
6 | tokio::spawn(async {
| ^^^^^^^^^^^^ future created by async block is not Send
|
::: [..]spawn.rs:127:21
|
127 | T: Future + Send + 'static,
| ---- required by this bound in
| tokio::task::spawn::spawn
|
= help: within impl std::future::Future, the trait
| std::marker::Send is not implemented for
| std::rc::Rc<&str>
note: future is not Send as this value is used across an await
--> src/main.rs:10:9
|
7 | let rc = Rc::new("hello");
| -- has type std::rc::Rc<&str> which is not Send
...
10 | yield_now().await;
| ^^^^^^^^^^^^^^^^^ await occurs here, with rc maybe
| used later
11 | println!("{}", rc);
12 | });
| - rc is later dropped here
```

다음 장에서 이 오류의 특수한 경우에 대해 자세히 설명하겠습니다.

# 값 저장하기 

아래는 들어오는 명령을 처리하기 위해 process 함수를 구현하는 예시입니다. 우리는 HashMap을 
사용하여 값을 저장할 것입니다. SET 명령은 HashMap에 삽입하고 GET 명령은 값을 로드할 
것입니다. 또한, 한 연결당 하나 이상의 명령을 수락하기 위해 루프를 사용할 것입니다.

```rust
use tokio::net::TcpStream;
use mini_redis::{Connection, Frame};

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // HashMap을 사용하여 데이터를 저장합니다.
    let mut db = HashMap::new();

    // Connection은 소켓에서 프레임을 파싱하는 데 사용됩니다.
    let mut connection = Connection::new(socket);

    // `read_frame`을 사용하여 연결에서 명령을 받습니다.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // 값은 `Vec<u8>`으로 저장됩니다.
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // `Frame::Bulk`은 데이터가 `Bytes` 유형이어야 합니다.
                    // 이 유형은 튜토리얼의 후반부에서 다룰 것입니다.
                    // 현재는 `&Vec<u8>`을 `Bytes`로 변환하기 위해 `into()`를 사용합니다.
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // 클라이언트에 응답을 작성합니다.
        connection.write_frame(&response).await.unwrap();
    }
}
```

위의 코드를 사용하여 들어오는 명령을 처리하고 응답을 보내는 기능을 구현할 수 있습니다.

## task::spawn() 내부 

task/spawn.rs 파일 

- scheduler::spawn(task, id) 호출 
  - multi_thread::handle::spawn() 호출
  
multi_thread::Handle 
```rust
pub(crate) struct Handle {
    /// Task spawner
    pub(super) shared: worker::Shared,

    /// Resource driver handles
    pub(crate) driver: driver::Handle,

    /// Blocking pool spawner
    pub(crate) blocking_spawner: blocking::Spawner,

    /// Current random number generator seed
    pub(crate) seed_generator: RngSeedGenerator,
}
```

worker::Shared가 중요하다. 

owned.bind() 함수로 task를 추가한다. 

task::new_task()에서 RawTask, Task, Notified, JoinHandle을 만든다. 

RawTask는 NonNull<Header>를 갖고 NonNull은 Clone, Copy이다. 따라서, RawTask도 그런 것으로 
보인다. 여러 곳에서 보관한다. 

NonNull이 Copy이면 어떻게 복제되는가? 메모리는 그대로 공유하는 형태로 복제한다. 
포인터 위치만 유지하면서 복사한다는 뜻이다. 따라서, 실제 메모리는 전체가 공유한다. 

복잡하지만 핵심 흐름을 몇 가지 잡으면 괜찮을 듯 하다. 

- runtime 
  - Scheduler
  - Task / RawTask 
- driver 
- clock 

챗지피티의 설명이다.

- 이벤트 루프 초기화: Tokio에서 Runtime 인스턴스를 생성하면 이벤트 루프가 초기화되고 실행기(executor) 및 리액터(reactor)와 같은 필요한 구성 요소가 설정됩니다.

- 태스크 생성: 비동기 코드를 실행하려면 tokio::spawn을 사용하여 이벤트 루프에 태스크를 생성합니다. 이는 새로운 태스크를 생성하고 실행을 위해 이벤트 루프에 등록합니다.

- 태스크 폴링: 이벤트 루프는 태스크의 상태를 지속적으로 폴링하여 진행 가능 여부를 결정합니다. 각 태스크에 대해 poll 메서드를 호출합니다.

- Future 실행: 이벤트 루프가 태스크를 폴링할 때, 태스크에 연결된 퓨처의 poll 메서드를 호출합니다. 여기서 퓨처의 로직이 실행됩니다.

- Future 진행: 퓨처의 poll 메서드는 퓨처가 아직 완료되지 않았을 경우 Poll::Pending을 반환합니다. 결과가 있는 경우 Poll::Ready(result)을 반환합니다.

- 제어 양보: 퓨처가 Poll::Pending을 반환하면 이는 제어를 이벤트 루프에 다시 양보해야 함을 나타냅니다. 이벤트 루프는 퓨처가 준비되었을 때 향후 폴링을 위해 퓨처를 스케줄링합니다.

- 논블로킹 I/O: 퓨처가 I/O 작업을 수행할 때에는 일반적으로 tokio::net이나 tokio::fs와 같은 Tokio의 비동기 I/O 기능을 사용합니다. 이러한 I/O 작업은 논블로킹이며, 리액터를 통해 운영 체제와 상호 작용합니다.

- 리액터 상호 작용: 퓨처가 I/O 작업을 시작하면 해당 작업을 리액터에게 전달하여 운영 체제와의 상호 작용을 관리합니다. 리액터는 비동기 시스템 호출을 사용하고, 작업이 준비되면 이벤트 루프에게 알립니다.

- 이벤트 알림: I/O 작업이 완료되거나 다른 이벤트(예: 타이머 만료)가 발생하면 리액터가 이벤트 루프에게 알립니다. 이는 연결된 태스크가 폴링 가능한 상태임을 나타냅니다.

- 태스크 재개: 태스크가 이벤트 루프로부터 알림을 받으면 이전에 양보한 지점에서 실행을 재개합니다. 이벤트 루프는 태스크의 퓨처의 poll 메서드를 다시 호출합니다.

- Future 완료: 퓨처가 폴링 중에 Poll::Ready(result)를 반환하면 이는 퓨처가 결과와 함께 완료되었음을 나타냅니다. 태스크가 완료되었으며, 이벤트 루프는 다음 태스크로 진행할 수 있습니다.

- 태스크 정리: 태스크가 완료되면 이벤트 루프는 필요한 정리 작업을 수행합니다. 예를 들어 태스크와 관련된 리소스를 해제합니다.

- 이러한 플로우는 모든 태스크가 완료되거나 명시적으로 취소를 요청할 때까지 반복됩니다.

참고로, 이는 단순화된 개요이며, Tokio의 내부 구현에는 추가적인 최적화와 세부 사항이 포함됩니다. 그러나 이를 통해 이벤트 루프가 Tokio에서 퓨처의 실행을 어떻게 구동하는지에 대한 대략적인 이해를 얻을 수 있습니다.



