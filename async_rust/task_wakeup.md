# Task wakeups with Waker

Future가 처음 폴링될 때 완료되지 않는 경우가 종종 있습니다. 이런 일이 발생하면 퓨처는 더 많은 진전을 이룰 준비가 되면 다시 폴링해야 합니다. 이는 웨이커 유형(Waker type)에서
수행됩니다.

퓨처가 폴링될 때마다 "task"의 일부로 폴링됩니다. 작업은 실행자에게 제출된 최상위 수준의
퓨처입니다.

Waker는 실행자에게 연결된 태스크를 깨우라고 알리는 데 사용할 수 있는 wake() 메서드를
제공합니다. wake()가 호출되면 실행자는 Waker와 연결된 태스크가 진행 준비가 되었음을 알 수
있으며, 해당 퓨처를 다시 폴링해야 합니다.

웨이커는 또한 clone()을 구현하여 복사하여 저장할 수 있습니다.

Waker를 사용해 간단한 타이머 Future를 구현해 보겠습니다.

## Applied: Build a Timer

이 예제에서는 타이머가 생성되면 새 스레드를 시작하고 필요한 시간 동안 쉰 후  
타이머 Future에 신호를 보내겠습니다.

먼저 cargo new --lib timer_future로 새 프로젝트를 시작하고 필요한 임포트를
src/lib.rs에 추가합니다:

```rust
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};
```

Future type 자체를 정의하는 것부터 시작하겠습니다. Future는 타이머가 경과했고 Future가
완료되어야 한다는 것을 스레드가 전달할 수 있는 방법이 필요합니다. 스레드와 퓨처 간에
통신하기 위해 공유 Arc<Mutex<..>> 값을 사용하겠습니다.

```rust
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

/// Shared state between the future and the waiting thread
struct SharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}
```

이제 실제로 Future를 작성해 보겠습니다!

```rust
impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `TimerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `TimerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

꽤 간단하죠? 스레드에서 shared_state.completed = true를 설정했다면 완료된 것입니다! 그렇지
않으면 현재 태스크에 대한 Waker를 복제하고 이를 shared_state.waker에 전달하여 스레드가 
태스크를 다시 깨울 수 있도록 합니다.

중요한 것은 퓨처가 다른 Waker를 사용하는 다른 작업으로 이동했을 수 있으므로 퓨처가 폴링될 
때마다 Waker를 업데이트해야 한다는 점입니다. 이는 폴링된 후 작업 간에 퓨처가 전달될 때
발생합니다.

마지막으로 타이머를 실제로 구성하고 스레드를 시작하는 API가 필요합니다:
```rust
impl TimerFuture {
    /// Create a new `TimerFuture` which will complete after the provided
    /// timeout.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Spawn the new thread
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut shared_state = thread_shared_state.lock().unwrap();
            // Signal that the timer has completed and wake up the last
            // task on which the future was polled, if one exists.
            shared_state.completed = true;
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}
```
우와! 간단한 타이머 Future를 만드는 데 필요한 것은 이것뿐입니다. 이제 Future를 실행할
실행자(Executor)만 있다면...








