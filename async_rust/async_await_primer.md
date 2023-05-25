# async / await primer

async/.await는 동기 코드처럼 보이는 비동기 함수를 작성하기 위한 Rust의 내장 도구입니다. 
async는 코드 블록을 Future라는 트레이트를 구현하는 상태 머신으로 변환합니다. 동기식 
메서드에서 차단(blocking) 함수를 호출하면 전체 스레드가 대기하는 반면, 대기하는 퓨처는 
스레드를 제어하여 다른 퓨처를 실행할 수 있도록 합니다.

Cargo.toml 파일에 몇 가지 종속성을 추가해 보겠습니다:
```toml
[dependencies]
futures = "0.3"
```

비동기 함수를 만들려면 async fn 구문을 사용하면 됩니다:
```rust
async fn do_something() { /* ... */ }
```

async fn이 반환하는 값은 Future입니다. 어떤 일이 일어나려면 실행자에서 Future를 실행해야
합니다.

```rust
// `block_on` blocks the current thread until the provided future has run to
// completion. Other executors provide more complex behavior, like scheduling
// multiple futures onto the same thread.
use futures::executor::block_on;

async fn hello_world() {
    println!("hello, world!");
}

fn main() {
    let future = hello_world(); // Nothing is printed
    block_on(future); // `future` is run and "hello, world!" is printed
}
```

비동기 함수 내에서 .await을 사용하여 다른 비동기 함수의 출력과 같이 Future 특성을 구현하는 
다른 유형이 완료될 때까지 기다릴 수 있습니다. block_on과 달리 .await는 현재 스레드를 
차단하지 않고 퓨처가 완료될 때까지 비동기적으로 기다리므로 퓨처가 현재 진행되지 않는 경우 
다른 작업을 실행할 수 있습니다.

예를 들어 learn_song, sing_song, dance라는 세 개의 비동기 fn이 있다고 가정해 보겠습니다:
```rust
async fn learn_song() -> Song { /* ... */ }
async fn sing_song(song: Song) { /* ... */ }
async fn dance() { /* ... */ }
```

학습, 노래, 춤을 각각 대기하는 것도 한 가지 방법이 될 수 있습니다:
```rust
fn main() {
    let song = block_on(learn_song());
    block_on(sing_song(song));
    block_on(dance());
}
```

하지만 이렇게 하면 한 번에 한 가지만 할 수 있기 때문에 최고의 퍼포먼스를 보여줄 수 
없습니다! 분명 노래를 부르기 전에 노래를 배워야 하지만, 노래를 배우고 부르는 동시에 춤을 
추는 것도 가능합니다. 이를 위해 동시에 실행할 수 있는 두 개의 개별 async fn을 만들 수 
있습니다:
```rust
async fn learn_and_sing() {
    // Wait until the song has been learned before singing it.
    // We use `.await` here rather than `block_on` to prevent blocking the
    // thread, which makes it possible to `dance` at the same time.
    let song = learn_song().await;
    sing_song(song).await;
}

async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    // `join!` is like `.await` but can wait for multiple futures concurrently.
    // If we're temporarily blocked in the `learn_and_sing` future, the `dance`
    // future will take over the current thread. If `dance` becomes blocked,
    // `learn_and_sing` can take back over. If both futures are blocked, then
    // `async_main` is blocked and will yield to the executor.
    futures::join!(f1, f2);
}

fn main() {
    block_on(async_main());
}
```

이 예제에서는 노래를 부르기 전에 노래를 배워야 하지만, 춤을 추면서 동시에 학습과 노래를 
할 수도 있습니다. learn_and_sing에서 learn_song().await 대신 block_on(learn_song())을 
사용했다면, learn_song이 실행되는 동안 스레드는 다른 작업을 수행할 수 없게 됩니다. 
이렇게 하면 동시에 춤을 추는 것이 불가능해집니다. .await을 통해 learn_song이 차단된 경우 
다른 작업이 현재 스레드를 이어받을 수 있도록 합니다. 이렇게 하면 동일한 스레드에서 여러 
개의 퓨처를 동시에 실행하여 완료할 수 있습니다.

<details>

<summary> 놀미 노트 </summary>

위와 같이 설명하면 Future가 실행단위로 오해할 수 있습니다. 실제 런타임 구현에서는 
Task를 Scheduelr가 Executor(Worker)의 실행 큐에 배분하여 실행하도록 합니다. 

tokio::spawn()같은 함수가 Future를 받아 일련의 비동기 실행 흐름을 만들어 냅니다. 
block_on()은 하나의 Future를 실행 할 수 있는 능력만 있는 것으로 보입니다.
</details>
 
## block_on() 분석 

```rust
/// Run a future to completion on the current thread.
///
/// This function will block the caller until the given future has completed.
///
/// Use a [`LocalPool`](LocalPool) if you need finer-grained control over
/// spawned tasks.
pub fn block_on<F: Future>(f: F) -> F::Output {
    pin_mut!(f);
    run_executor(|cx| f.as_mut().poll(cx))
}
```
pin하고 run_executor로 future를 실행합니다. 

```rust
// Set up and run a basic single-threaded spawner loop, invoking `f` on each
// turn.
fn run_executor<T, F: FnMut(&mut Context<'_>) -> Poll<T>>(mut f: F) -> T {
    let _enter = enter().expect(
        "cannot execute `LocalPool` executor from within \
         another executor",
    );

    CURRENT_THREAD_NOTIFY.with(|thread_notify| {
        let waker = waker_ref(thread_notify);
        let mut cx = Context::from_waker(&waker);
        loop {
            if let Poll::Ready(t) = f(&mut cx) {
                return t;
            }

            // Wait for a wakeup.
            while !thread_notify.unparked.swap(false, Ordering::Acquire) {
                // No wakeup occurred. It may occur now, right before parking,
                // but in that case the token made available by `unpark()`
                // is guaranteed to still be available and `park()` is a no-op.
                thread::park();
            }
        }
    })
}
```

Future의 poll() 기능을 사용하여 FnMut() -> Poll<T>를 받도록 합니다. 
unparked가 아닌 동안 대기하다가 현재 쓰레드가 unparked 되면 thread::park()를 
호출합니다. thread의 park / unpark는 std에 정의된 걸 알 수 있습니다. 
park / unpark는 쓰레드를 멈추고 깨우는 기능입니다. 구체적인 동작은 이해할 내용에서 
더 살펴봅니다. 아직 한번에 다 이해는 못 하고 있습니다. 

f(&mut cx) 호출이 Pending이라면 unpark될 때까지 기다렸다가 park()를 합니다. 
그러면 언제 다시 깨우게 되나요? 이 부분이 Future 실행의 핵심입니다. 

자체 실행기 구현에 대한 내용이 추가로 있으므로 앞으로 나아갑니다. 

# 이해할 내용

## thread::park / unpark

모든 스레드에는 thread::park 함수와 thread::Thread::unpark 메서드를 통해 몇 가지 기본적인 
저수준 차단 기능이 있습니다. park는 현재 스레드를 차단한 다음 차단된 스레드의 핸들에서 
unpark 메서드를 호출하여 다른 스레드에서 다시 시작할 수 있습니다.

개념적으로 각 스레드 핸들에는 처음에는 존재하지 않는 연관 토큰이 있습니다:

thread::park 함수는 해당 스레드 핸들에 토큰을 사용할 수 있을 때까지 현재 스레드를 차단하며, 
이 시점에서 토큰을 원자적으로 소비합니다. 또한 토큰을 소비하지 않고 가짜로 반환할 수도 
있습니다. thread::park_timeout은 동일한 작업을 수행하지만 스레드를 차단할 최대 시간을 
지정할 수 있습니다.

스레드에서 unpark 메서드는 토큰을 원자 단위로 사용할 수 없는 경우 토큰을 사용할 수 있게 
만듭니다. 토큰이 처음에 없으므로 unpark에 이어 park를 사용하면 두 번째 호출이 즉시 
반환됩니다.

다시 말해, 각 스레드는 공원과 공원 해제를 통해 잠그고 해제할 수 있는 스핀록처럼 작동합니다.

차단이 해제되었다고 해서 이 스레드를 차단한 사람과 동기화되었다는 의미는 아니며, 가짜일 
수도 있다는 점에 유의하세요. 예를 들어, park 및 unpark가 모두 아무 작업 없이 즉시 
반환되도록 하는 것은 유효하지만 비효율적인 구현이 될 수 있습니다.

API는 일반적으로 현재 스레드에 대한 핸들을 가져와 다른 스레드가 찾을 수 있도록 공유 데이터
 구조에 해당 핸들을 배치한 다음 루프에 파킹하는 방식으로 사용됩니다. 원하는 조건이 
 충족되면 다른 스레드가 해당 핸들에 대해 unpark를 호출합니다.

이 설계의 동기는 두 가지입니다:

- 새로운 동기화 프리미티브를 빌드할 때 뮤텍스와 condvar를 할당할 필요가 없고, 스레드가 
이미 기본적인 블로킹/신호 기능을 제공하기 때문입니다.

- 많은 플랫폼에서 매우 효율적으로 구현할 수 있습니다.

정리하면 park()는 현재 쓰레드를 멈추는 기능이고 unpark()는 다시 실행하는 기능입니다. 

어떻게 동작하나요? 



