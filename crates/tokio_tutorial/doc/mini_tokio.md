# mini tokio 

미니 토키오는 mini-redis, mini-http 처럼 단순화된 예시를 통해 설명하려는 시도 중 하나이다. 토키오 튜토리얼을 여러 번 보고 있는데 볼수록 괜찮은 문서이다. 한번 보고 이해하기 어렵다는 단점이 있지만 토키오를 이 이상으로 설명하기도 애매할 수 있을 정도로 모두 설명하고 있다. 

미니 토키오를 보면서 연관된 내용을 함께 살펴서 이해하면 토키오 전체를 이해할 수 있을 것으로 기대한다. 

## Futures

```rust
use tokio::net::TcpStream;

async fn my_async_fn() {
    println!("hello from async");
    let _socket = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    println!("async TCP operation complete");
}

#[tokio::main]
async fn main() {
    let what_is_this = my_async_fn();
    // Nothing has been printed yet.
    what_is_this.await;
    // Text has been printed and socket has been
    // established and closed.
}
```

async fn은 Future를 돌려준다. 컴파일러가 Future 코드를 만들어 돌려준다. 
await를 해야 poll()이 실행된다. 

```rust
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
}
```

Future는 trait이다. poll() 하나만 있다. 이를 통해 계산(Computation)을 수행한다. 

### Future 구현

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<&'static str>
    {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Ignore this line for now.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    let when = Instant::now() + Duration::from_millis(10);
    let future = Delay { when };

    let out = future.await;
    assert_eq!(out, "done");
}
```

Poll::Ready()로 값을 돌려주고, Poll::Pending으로 대기해야 함을 알리고, 
cx.waker()를 통해 준비되면 알려서 다시 poll()이 호출 되도록 한다. 매우 간단하다. 

### async fn 을 Future로 

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

enum MainFuture {
    // Initialized, never polled
    State0,
    // Waiting on `Delay`, i.e. the `future.await` line.
    State1(Delay),
    // The future has completed.
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<()>
    {
        use MainFuture::*;

        loop {
            match *self {
                State0 => {
                    let when = Instant::now() +
                        Duration::from_millis(10);
                    let future = Delay { when };
                    *self = State1(future);
                }
                State1(ref mut my_future) => {
                    match Pin::new(my_future).poll(cx) {
                        Poll::Ready(out) => {
                            assert_eq!(out, "done");
                            *self = Terminated;
                            return Poll::Ready(());
                        }
                        Poll::Pending => {
                            return Poll::Pending;
                        }
                    }
                }
                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}
```

컴파일러가 생성하는 Future의 상상도이다. 아마도 컴파일러 작성자들은 알고 있을 것이다. 매크로 확장처럼 키워드에 대응하는 코드를 만들고 파싱과 이후 단계를 진행할 것이다. 컴파일 단계의 정확히 어느 지점에서 코드 생성을 수행할 지는 컴파일러 작성자의 권한이자 의무이다. 

위 생성된 코드에서 보면 Future는 매우 간단한 FSM으로 구현되는 걸 알 수 있다. 내부 상태에 따라 Pending을 반복하고 Ready로 끝난다. 

## Executors (실행기)

가장 외부의 Future를 누군가 실행해야 한다. 이 역할을 Executor가 한다. 

### mini-tokio 

```rust
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use futures::task;

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

struct MiniTokio {
    tasks: VecDeque<Task>,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }
    
    /// Spawn a future onto the mini-tokio instance.
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future));
    }
    
    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        
        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}
```

MiniTokio::run()이 실행을 하고 내부에 Task를 갖는다. Task는 Pin된 Future의 Box이다. 

위 run() 함수는 Pending일 경우 다시 Task 큐에 넣고 다음에 다시 실행한다. Ready를 리턴하면 Task를 더 이상 실행하지 않는다. 

spawn()은 더 단순하게 큐에 Task를 Pin해서 넣는다. 

## Wakers

Context에서 Task와 연결된 Waker를 waker() 함수로 얻을 수 있다. waker().wake() 함수 호출로 Executor에 깨워야 할 Task를 알려준다. 

### mini-tokio 개선 

```rust
use crossbeam::channel;
use std::sync::Arc;

struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}
```

Waker 구현을 위해 crossbeam의 channel을 사용한다. Send + Sync이기 때문에 이를 사용한다. 

```rust
use std::sync::{Arc, Mutex};

struct Task {
    // The `Mutex` is to make `Task` implement `Sync`. Only
    // one thread accesses `future` at any given time. The
    // `Mutex` is not required for correctness. Real Tokio
    // does not use a mutex here, but real Tokio has
    // more lines of code than can fit in a single tutorial
    // page.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        self.executor.send(self.clone());
    }
}
```

schedule도 단순하게 채널로 보낸다. 

```rust
use futures::task::{self, ArcWake};
use std::sync::Arc;
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}
```

futures crate에 있는 ArcWake 트레이트를 구현하여 처리한다. 

```rust
    fn poll(self: Arc<Self>) {
        // Create a waker from the `Task` instance. This
        // uses the `ArcWake` impl from above.
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        // No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();

        // Poll the future
        let _ = future.as_mut().poll(&mut cx);
    }
```

Task::poll() 함수 구현이다. 자기 자신이 ArcWake라서 이를 Context에 waker로 제공한다. 
poll() 할때마다 생성해서 전달한다. 

```rust
    // Spawns a new task with the given future.
    //
    // Initializes a new Task harness containing the given future and pushes it
    // onto `sender`. The receiver half of the channel will get the task and
    // execute it.
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
```

Task::spawn()을 사용해서 MiniTokio가 spawn() 한다. 

## 전체 코드 읽기 

CURRENT를 thread_local!로 생성하고 with()로 지정하여 MiniTokio의 sender를 
얻을 수 있게 한다. 

다른 부분은 설명에 나온 코드 그대로 구현되었다. 주석도 매우 자세하다. 

## await는 어떻게 동작하나?

await! 매크로 처럼 동작하고 상태기계를 갖는 Future로 만든다. 이 Future를 런타임에서 실행하고 완료되면 함수내 다음 코드로 진행한다. 

이 부분의 코드 생성은 컴파일러만 가능하다. 

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

async fn my_async_function() -> u32 {
    let result = await!(fetch_data());
    println!("Data fetched: {}", result);
    42
}

fn fetch_data() -> impl Future<Output = u32> {
    // Simulating an asynchronous operation
    async {
        // Simulating waiting for some external data
        await!(async_std::task::sleep(std::time::Duration::from_secs(5)));

        100
    }
}

fn main() {
    async_std::task::block_on(async {
        let result = await!(my_async_function());
        println!("Async function returned: {}", result);
    });
}
```

```rust
fn my_async_function(
    __task: &mut std::task::Context<'_>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = u32> + '_>> {
    struct State {
        future: Pin<Box<dyn std::future::Future<Output = u32> + '_>>,
        completed: bool,
    }

    impl std::future::Future for State {
        type Output = u32;

        fn poll(
            self: Pin<&mut Self>,
            __task: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            if self.completed {
                // The future has already completed, return the result
                std::task::Poll::Ready(0)
            } else {
                let result = std::future::Future::poll(
                    Pin::new(&mut self.get_mut().future),
                    __task,
                );

                match result {
                    std::task::Poll::Ready(value) => {
                        self.get_mut().completed = true;
                        std::task::Poll::Ready(value)
                    }
                    std::task::Poll::Pending => std::task::Poll::Pending,
                }
            }
        }
    }

    let mut state = State {
        future: Box::pin(fetch_data()),
        completed: false,
    };

    std::pin::Pin::new(&mut state)
}
```

Future를 호출하여 생성하고 await를 호출하는 부분이 통합되어 코드로 만들어진다. 















