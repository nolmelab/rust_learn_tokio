# Applied : Build an Executor

Rust의 퓨처는 게으르기 때문에 적극적으로 완료를 유도하지 않으면 아무 작업도 하지 않습니다.
퓨처를 완료까지 구동하는 한 가지 방법은 비동기 함수 내부에서 .await하는 것이지만, 이는
최상위 비동기 함수에서 반환된 퓨처를 누가 실행할 것인가라는 문제를 한 단계 더 끌어올릴
뿐입니다. 정답은 퓨처 실행자가 필요하다는 것입니다.

퓨처 실행자는 일련의 최상위 퓨처를 가져와서 퓨처가 진전을 이룰 수 있을 때마다 poll을
호출하여 완료될 때까지 실행합니다. 일반적으로 실행자는 시작을 위해 퓨처를 한 번 폴링합니다.
퓨처가 wake()를 호출하여 진행 준비가 되었음을 나타내면 퓨처를 대기열에 다시 배치하고 poll을
다시 호출하여 퓨처가 완료될 때까지 반복합니다.

이 섹션에서는 많은 수의 최상위 퓨처를 동시에 실행하여 완료할 수 있는 간단한 실행기를 직접
작성해 보겠습니다.

이 예제에서는 웨이커를 쉽게 구성할 수 있는 ArcWake 특성의 Future 크레이트에 의존합니다. 
Cargo.toml을 편집하여 새 종속성을 추가합니다:
```toml
[package]
name = "timer_future"
version = "0.1.0"
authors = ["XYZ Author"]
edition = "2021"

[dependencies]
futures = "0.3"
```

다음으로 src/main.rs의 상단에 다음과 같은 임포트가 필요합니다:
```rust
use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};
use std::{
    future::Future,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::Context,
    time::Duration,
};
// The timer we wrote in the previous section:
use timer_future::TimerFuture;
```

실행자는 채널에서 실행할 작업을 전송하는 방식으로 작동합니다. 실행자는 채널에서 이벤트를 
가져와 실행합니다. 태스크가 더 많은 작업을 수행할 준비가 되면(깨어남), 채널에 다시 배치하여
다시 폴링되도록 스스로 예약할 수 있습니다.

이 설계에서는 실행자 자체는 작업 채널의 수신 측만 있으면 됩니다. 사용자는 새로운 퓨처를
생성할 수 있도록 송신단을 얻게 됩니다. 태스크 자체는 스스로 일정을 변경할 수 있는
퓨처이므로 태스크가 스스로 큐에 대기하는 데 사용할 수 있는 발신자와 짝을 이루는 퓨처로
저장할 것입니다.

```rust
/// Task executor that receives tasks off of a channel and runs them.
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone)]
struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need to use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: SyncSender<Arc<Task>>,
}

fn new_executor_and_spawner() -> (Executor, Spawner) {
    // Maximum number of tasks to allow queueing in the channel at once.
    // This is just to make `sync_channel` happy, and wouldn't be present in
    // a real executor.
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}
```

또한 새로운 퓨처를 쉽게 생성할 수 있도록 Spawner에 메서드를 추가해 보겠습니다. 이 메서드는
Future 타잎을 가져와서 Box에 넣고 그 안에 실행기 큐에 넣을 수 있는 새로운 Arc<Task>를
생성합니다.

퓨처를 폴링하려면 Waker를 생성해야 합니다. 작업 웨이크업 섹션에서 설명한 것처럼 Waker는
waker()가 호출되면 다시 폴링할 작업을 예약할 책임이 있습니다. Waker는 실행자(Executor)에게
어떤 작업이 준비되었는지 정확히 알려주므로, 실행자(Executor)는 진행 준비가 완료된 퓨처만
폴링할 수 있습니다. 새 Waker를 생성하는 가장 쉬운 방법은 ArcWake 특성을 구현한 다음
waker_ref 또는 .into_waker() 함수를 사용하여 Arc<impl ArcWake>를 Waker로 바꾸는 것입니다.
Task에 ArcWake를 구현하여 Waker로 전환하고 깨울 수 있도록 해보겠습니다:

Arc<Task>에서 Waker가 생성되고, wake()를 호출하면 Arc의 복사본이 태스크 채널로 전송됩니다.
그러면 실행자가 태스크를 선택해 폴링해야 합니다. 이를 구현해 봅시다:

```rust
impl Executor {
    fn run(&self) {
        while let Ok(task) = self.ready_queue.recv() {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if future.as_mut().poll(context).is_pending() {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}
```

축하합니다! 이제 작동하는 퓨처 실행자가 생겼습니다. 이 실행기를 사용해 앞서 작성한 
TimerFuture와 같은 사용자 정의 퓨처와 async/.await 코드를 실행할 수도 있습니다:

```rust
fn main() {
    let (executor, spawner) = new_executor_and_spawner();

    // Spawn a task to print before and after waiting on a timer.
    spawner.spawn(async {
        println!("howdy!");
        // Wait for our timer future to complete after two seconds.
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    // Drop the spawner so that our executor knows it is finished and won't
    // receive more incoming tasks to run.
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    executor.run();
}
```

<details>

<summary> 놀미 노트 </summary>

메인 쓰레드에서 실행하는 Task와 Channel 기반으로 Future들을 실행하는 
간단한 실행기와 ArcWake 기반의  Waker 구현을 보여줍니다. 

tokio와 같이 실작업에 사용하는 라이브러리는 훨씬 더 강력하고 
섬세하게 구현한 스케줄러와 실행기를 포함합니다. 

개념 상으로 정리하기에는 좋은 예시입니다. 
</details>