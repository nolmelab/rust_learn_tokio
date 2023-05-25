# The Future trait

Future 특성은 Rust에서 비동기 프로그래밍의 중심에 있습니다. Future는 값을 생성할 수 있는
비동기 연산입니다(예: ()와 같이 값이 비어 있을 수 있음). Future 트레이트트의 단순화된 
버전은 다음과 같이 보일 수 있습니다:
```rust
trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

poll() 함수를 호출하여 미래를 가능한 한 완료까지 진행시킬 수 있습니다. 퓨처가 완료되면
Poll::Ready(결과)를 반환합니다. 퓨처가 아직 완료되지 않은 경우 Poll::Pending을 반환하고
퓨처가 더 진전될 준비가 되면 wake() 함수가 호출되도록 준비합니다. wake()가 호출되면 퓨처를
구동하는 실행자는 퓨처가 더 많은 진전을 이룰 수 있도록 poll을 다시 호출합니다.

wake()가 없다면 실행자는 특정 퓨처가 언제 진전을 이룰 수 있는지 알 수 없으며, 모든 퓨처를
계속 폴링해야 합니다. wake()를 사용하면 실행자는 어떤 퓨처가 폴링할 준비가 되었는지 정확히
알 수 있습니다.

예를 들어, 이미 데이터가 있을 수도 있고 없을 수도 있는 소켓에서 데이터를 읽으려는 경우를
생각해 봅시다. 데이터가 있으면 데이터를 읽어 Poll::Ready(data)를 반환할 수 있지만, 데이터가
준비되지 않으면 퓨처가 차단되어 더 이상 진행할 수 없습니다. 사용할 수 있는 데이터가 없는
경우 소켓에서 데이터가 준비되면 호출되도록 wake를 등록해야 실행자에게 퓨처가 진행할 준비가
되었음을 알릴 수 있습니다. 간단한 SocketRead 퓨처는 다음과 같이 보일 수 있습니다:

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

이 퓨처 모델을 사용하면 중간 할당 없이 여러 비동기 연산을 함께 구성할 수 있습니다. 한 번에 여러 개의 퓨처를 실행하거나 퓨처를 체인으로 연결하는 것은 이와 같은 할당 없는 상태 머신을 통해 구현할 수 있습니다:
```rust
/// A SimpleFuture that runs two other futures to completion concurrently.
///
/// Concurrency is achieved via the fact that calls to `poll` each future
/// may be interleaved, allowing each future to advance itself at its own pace.
pub struct Join<FutureA, FutureB> {
    // Each field may contain a future that should be run to completion.
    // If the future has already completed, the field is set to `None`.
    // This prevents us from polling a future after it has completed, which
    // would violate the contract of the `Future` trait.
    a: Option<FutureA>,
    b: Option<FutureB>,
}

impl<FutureA, FutureB> SimpleFuture for Join<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        // Attempt to complete future `a`.
        if let Some(a) = &mut self.a {
            if let Poll::Ready(()) = a.poll(wake) {
                self.a.take();
            }
        }

        // Attempt to complete future `b`.
        if let Some(b) = &mut self.b {
            if let Poll::Ready(()) = b.poll(wake) {
                self.b.take();
            }
        }

        if self.a.is_none() && self.b.is_none() {
            // Both futures have completed -- we can return successfully
            Poll::Ready(())
        } else {
            // One or both futures returned `Poll::Pending` and still have
            // work to do. They will call `wake()` when progress can be made.
            Poll::Pending
        }
    }
}
```

위 코드는 별도의 할당 없이 여러 개의 Future를 동시에 실행하여 보다 효율적인 비동기
프로그램을 실행할 수 있는 방법을 보여줍니다. 이와 유사하게 여러 개의 순차적 Future도 
다음과 같이 차례로 실행할 수 있습니다:

```rust
/// A SimpleFuture that runs two futures to completion, one after another.
//
// Note: for the purposes of this simple example, `AndThenFut` assumes both
// the first and second futures are available at creation-time. The real
// `AndThen` combinator allows creating the second future based on the output
// of the first future, like `get_breakfast.and_then(|food| eat(food))`.
pub struct AndThenFut<FutureA, FutureB> {
    first: Option<FutureA>,
    second: FutureB,
}

impl<FutureA, FutureB> SimpleFuture for AndThenFut<FutureA, FutureB>
where
    FutureA: SimpleFuture<Output = ()>,
    FutureB: SimpleFuture<Output = ()>,
{
    type Output = ();
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if let Some(first) = &mut self.first {
            match first.poll(wake) {
                // We've completed the first future -- remove it and start on
                // the second!
                Poll::Ready(()) => self.first.take(),
                // We couldn't yet complete the first future.
                Poll::Pending => return Poll::Pending,
            };
        }
        // Now that the first future is done, attempt to complete the second.
        self.second.poll(wake)
    }
}
```

이 예제는 여러 개의 할당된 객체와 깊이 중첩된 콜백 없이도 Future 특성을 사용하여 비동기 
제어 흐름을 표현하는 방법을 보여줍니다. 기본적인 제어 흐름에 대해 설명했으니 이제 실제 
Future 특성과 그 차이점에 대해 이야기해 보겠습니다.

```rust
trait Future {
    type Output;
    fn poll(
        // Note the change from `&mut self` to `Pin<&mut Self>`:
        self: Pin<&mut Self>,
        // and the change from `wake: fn()` to `cx: &mut Context<'_>`:
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output>;
}
```
가장 먼저 눈에 띄는 변화는 셀프 유형이 더 이상 &mut Self가 아니라 Pin<&mut Self>로 
변경되었다는 점입니다. 뒷부분에서 고정에 대해 자세히 설명하겠지만, 지금은 고정이 움직이지
않는 퓨처를 생성할 수 있게 해준다는 점만 알아두세요. Pinned 객체는 필드 사이에 포인터를
저장할 수 있습니다(예: 구조체 MyFut { a: i32, ptr_to_a: *const i32 }. async/await를 
활성화하려면 고정이 필요합니다. 

둘째, wake: fn()이 &mut Context<'_>로 변경되었습니다. SimpleFuture에서는 함수 포인터(fn())
에 대한 호출을 사용해 퓨처 실행자에게 해당 퓨처를 폴링해야 한다는 것을 알렸습니다. 하지만 
fn()은 함수 포인터일 뿐이므로 어떤 퓨처가 깨어났는지에 대한 데이터를 저장할 수 없습니다.

실제 시나리오에서 웹 서버와 같은 복잡한 애플리케이션에는 깨우기를 모두 개별적으로 관리해야 하는 수천 개의 서로 다른 연결이 있을 수 있습니다. 컨텍스트 유형(Context type)은 특정 작업을 깨우는 데 사용할 수 있는 Waker 유형의 값에 대한 액세스를 제공함으로써 이 문제를 해결합니다.





