# Hello Tokio

```rust
use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("got value from the server; result={:?}", result);

    Ok(())
}
```

Cargo.toml의 [dependency]에 다음을 등록합니다. 
```toml
[dependency]
tokio = { version = "1", features = ["full"] }
mini-redis = "0.4"
```

위 코드를 main.rs에 복사하고 빌드합니다. 

# async fn은 future

tokio는 러스트의 비동기 기능인 async / await와 함께 동작합니다. 
다른 비동기 관련 std 라이브러리도 함꼐 사용합니다. 

tokio가 주로 구현한 부분은 스케줄러, 실행기 부분이고 스케줄러가 
핵심입니다. 

async fn으로 된 부분은 Future로 컴파일러가 변환합니다. 
Future는 트레이트이며 poll() 함수를 갖고 있습니다. 

Future는 task 안에서 실행되고 tokio가 스케줄링 하는 단위는 
task입니다. task가 그린 스레드라고 불리는 스케줄러의 실행 
단위입니다. 

Future는 Ready로 실행 결과 값을 전달하거나 Pending으로 대기하고 
Waker를 통해 준비가 되면 스케줄러에 알려서 Task를 통해 
다시 poll()이 불리도록 합니다. 이 때 완료되게 됩니다. 

Future는 처음에 불릴 때 대부분 비동기 작업을 시작하고 Pending을 돌려준 후 
다른 비동기 호출이나 별도의 쓰레드 풀을 사용하여 작업을 완료한 후에 
Waker를 통해 다시  poll()이 불리도록 하고, 이 때는 Ready를 돌려줘서 
await가 끝나도록 합니다. 

Future는 Task로만 실행되고, 한번 Task 안에서 실행되면 여러 Future를 
통해 이어서 실행이 가능합니다. 

- [ ] Waker를 통해 통지하는 구조를 확인하세요
- [ ] tokio::fs, tokio::io를 분석하여 Future를 만드는 법을 확실히 연습하세요.


# client::connect() 흐름 

- `TcpStream` 
  - `pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream>` 
    - `async fn connect_addr(addr: SocketAddr) -> io::Result<TcpStream>`
    - tcp module 
      - `pub(crate) fn connect(socket: &net::TcpStream, addr: SocketAddr) -> io::Result<()>`
      - 소켓의 연결을 시작한다. 
    - `pub(crate) async fn connect_mio(sys: mio::net::TcpStream) -> io::Result<TcpStream>`

TcpStream::connect_mio()는 자세히 봐야 합니다.

```rust
pub(crate) async fn connect_mio(sys: mio::net::TcpStream) -> io::Result<TcpStream> {
  let stream = TcpStream::new(sys)?;

  // Once we've connected, wait for the stream to be writable as
  // that's when the actual connection has been initiated. Once we're
  // writable we check for `take_socket_error` to see if the connect
  // actually hit an error or not.
  //
  // If all that succeeded then we ship everything on up.
  poll_fn(|cx| stream.io.registration().poll_write_ready(cx)).await?;

  if let Some(e) = stream.io.take_error()? {
    return Err(e);
  }

  Ok(stream)
}

위 코드에서 핵심은 poll_fn()과 그 안의 Fn인 stream.io.registration().poll_write_ready(cx)입니다. 

```rust
pub struct PollFn<F> {
    f: F,
}

/// Creates a new future wrapping around a function returning [`Poll`].
pub fn poll_fn<T, F>(f: F) -> PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    PollFn { f }
}
```

poll_fn은 Poll<T>를 돌려주는 FnMut를 보관합니다. 
이는 Poll<T>읜 T는 poll_write_ready()가 돌려주는 값입니다. 

```rust
pub(crate) fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<io::Result<ReadyEvent>> 
```

위에서 보듯이 io::Result<ReadyEvent>입니다. 

```rust
impl<T, F> Future for PollFn<F>
where
    F: FnMut(&mut Context<'_>) -> Poll<T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        // Safety: We never construct a `Pin<&mut F>` anywhere, so accessing `f`
        // mutably in an unpinned way is sound.
        //
        // This use of unsafe cannot be replaced with the pin-project macro
        // because:
        //  * If we put `#[pin]` on the field, then it gives us a `Pin<&mut F>`,
        //    which we can't use to call the closure.
        //  * If we don't put `#[pin]` on the field, then it makes `PollFn` be
        //    unconditionally `Unpin`, which we also don't want.
        let me = unsafe { Pin::into_inner_unchecked(self) };
        (me.f)(cx)
    }
}
```

PollFn<F>는 Future입니다. 위에서 me의 타잎은 &mut PollFn<F>입니다. 

- [ ] 위 코드의 주석에 설명한 Pin 내용을 이해하지 못 합니다. 

```rust
    /// Polls for events on the I/O resource's `direction` readiness stream.
    ///
    /// If called with a task context, notify the task when a new event is
    /// received.
    fn poll_ready(
        &self,
        cx: &mut Context<'_>,
        direction: Direction,
    ) -> Poll<io::Result<ReadyEvent>> {
        // Keep track of task budget
        let coop = ready!(crate::runtime::coop::poll_proceed(cx));
        let ev = ready!(self.shared.poll_readiness(cx, direction));

        if ev.is_shutdown {
            return Poll::Ready(Err(gone()));
        }

        coop.made_progress();
        Poll::Ready(Ok(ev))
    }
```

poll_write_ready()는 위 함수를 호출하는 것이 전부입니다. 

poll_ready()에서 ready! 매크로는 Pending일 경우 Pending을 리턴합니다. 
따라서, 뭔가 준비되어 호출되기 전에는 다시 호출되지 않습니다. 

어디서 이 Future를 실행하는 Task를 깨우게 될까요?
`poll_proceed(cx)`와 `poll_readiness(cx, direction)`에 비밀이 숨겨져 있을 듯 합니다. 

- [ ] 이 비밀을 찾으세요

# client::set() 분석 

set_cmd()가 핵심입니다. 

```rust
    /// The core `SET` logic, used by both `set` and `set_expires.
    async fn set_cmd(&mut self, cmd: Set) -> crate::Result<()> {
        // Convert the `Set` command into a frame
        let frame = cmd.into_frame();

        debug!(request = ?frame);

        // Write the frame to the socket. This writes the full frame to the
        // socket, waiting if necessary.
        self.connection.write_frame(&frame).await?;

        // Wait for the response from the server. On success, the server
        // responds simply with `OK`. Any other response indicates an error.
        match self.read_response().await? {
            Frame::Simple(response) if response == "OK" => Ok(()),
            frame => Err(frame.to_error()),
        }
    }
```

into_frame()은 Set에 구현된 메서드입니다. 
write_frame()은 Connection의 함수입니다. 

read_response().await?를 match 해서 처리하는 기능은 러스트의
편리한 장점입니다. 에러 리턴까지 자동으로 되니 간결해집니다. 

Frame이 통신 프로토콜이라는 점을 알 수 있습니다. 













