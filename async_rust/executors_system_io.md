# Executors and System IO 

퓨처 특성에 대한 이전 섹션에서 소켓에서 비동기 읽기를 수행하는 퓨처의 예제에 대해 
설명했습니다:

```rust
pub struct SocketRead<'a> {
    socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // The socket has data -- read it into a buffer and return it.
            Poll::Ready(self.socket.read_buf())
        } else {
            // The socket does not yet have data.
            //
            // Arrange for `wake` to be called once data is available.
            // When data becomes available, `wake` will be called, and the
            // user of this `Future` will know to call `poll` again and
            // receive data.
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}
```

이 퓨처는 소켓에서 사용 가능한 데이터를 읽고, 사용 가능한 데이터가 없으면 실행자에게
양보하여 소켓을 다시 읽을 수 있게 되면 해당 작업을 깨우도록 요청합니다. 그러나 이 
예제에서는 소켓 유형이 어떻게 구현되는지 명확하지 않으며, 특히 set_readable_callback 
함수가 어떻게 작동하는지 명확하지 않습니다. 소켓을 읽을 수 있게 되면 어떻게 wake()가 
호출되도록 준비할 수 있을까요? 한 가지 옵션은 소켓이 읽을 수 있는지 여부를 지속적으로 
확인하여 적절한 경우 wake()를 호출하는 스레드를 갖는 것입니다. 그러나 이는 매우 
비효율적이며, 향후 차단된 각 IO에 대해 별도의 스레드가 필요합니다. 이렇게 하면 비동기 
코드의 효율성이 크게 떨어집니다.

실제로 이 문제는 Linux의 epoll, FreeBSD 및 Mac OS의 kqueue, Windows의 IOCP, Fuchsia의 
포트(모두 크로스 플랫폼 Rust 크레이트 mio를 통해 노출됨)와 같은 IO 인식 시스템 차단 
프리미티브와의 통합을 통해 해결할 수 있습니다. 이러한 프리미티브는 모두 스레드가 여러 
비동기 IO 이벤트를 차단하고 이벤트 중 하나가 완료되면 반환할 수 있도록 해줍니다. 실제로 
이러한 API는 일반적으로 다음과 같이 보입니다:

```rust
struct IoBlocker {
    /* ... */
}

struct Event {
    // An ID uniquely identifying the event that occurred and was listened for.
    id: usize,

    // A set of signals to wait for, or which occurred.
    signals: Signals,
}

impl IoBlocker {
    /// Create a new collection of asynchronous IO events to block on.
    fn new() -> Self { /* ... */ }

    /// Express an interest in a particular IO event.
    fn add_io_event_interest(
        &self,

        /// The object on which the event will occur
        io_object: &IoObject,

        /// A set of signals that may appear on the `io_object` for
        /// which an event should be triggered, paired with
        /// an ID to give to events that result from this interest.
        event: Event,
    ) { /* ... */ }

    /// Block until one of the events occurs.
    fn block(&self) -> Event { /* ... */ }
}

let mut io_blocker = IoBlocker::new();
io_blocker.add_io_event_interest(
    &socket_1,
    Event { id: 1, signals: READABLE },
);
io_blocker.add_io_event_interest(
    &socket_2,
    Event { id: 2, signals: READABLE | WRITABLE },
);
let event = io_blocker.block();

// prints e.g. "Socket 1 is now READABLE" if socket one became readable.
println!("Socket {:?} is now {:?}", event.id, event.signals);
```

퓨처 실행자는 이러한 프리미티브를 사용하여 특정 IO 이벤트가 발생할 때 실행되도록 콜백을 
구성할 수 있는 소켓과 같은 비동기 IO 객체를 제공할 수 있습니다. 위의 SocketRead 예제의 
경우, Socket::set_readable_callback 함수는 다음과 같은 의사 코드처럼 보일 수 있습니다:
```rust
impl Socket {
    fn set_readable_callback(&self, waker: Waker) {
        // `local_executor` is a reference to the local executor.
        // this could be provided at creation of the socket, but in practice
        // many executor implementations pass it down through thread local
        // storage for convenience.
        let local_executor = self.local_executor;

        // Unique ID for this IO object.
        let id = self.id;

        // Store the local waker in the executor's map so that it can be called
        // once the IO event arrives.
        local_executor.event_map.insert(id, waker);
        local_executor.add_io_event_interest(
            &self.socket_file_descriptor,
            Event { id, signals: READABLE },
        );
    }
}
```

이제 실행자 스레드 하나만 있으면 모든 IO 이벤트를 수신하여 적절한 웨이커에 디스패치할 수 
있으며, 웨이커는 해당 작업을 깨워 더 많은 작업을 완료한 후 다시 돌아와 더 많은 IO 이벤트를 
확인할 수 있습니다(그리고 이 사이클은 계속됩니다...).

<details>

<summary> 놀미 노트 </summary>

select, poll, epoll, kqueue을 사용해 본 경험이 없다면 IoBlocker가 생소할 수 있습니다. 
IOCP는 조금 다른 인터페이스를 갖지만 OS의 io 이벤트를 대기하는 큪라는 점에서는 같습니다. 

io_block.block()이 OS나 한 쓰레드에서 add_io_event_interest()로 등록한 이벤트를 
기다리다가 하나라도 이벤트가 발생하면 깨어나서 알리는 기능을 합니다. 

데이터베이스 처리나 파일 시스템 처리도 블럭킹(차단) 호출을 갖고 있으므로 이 쪽의 
Future 구현도 궁금합니다. 
</details>

