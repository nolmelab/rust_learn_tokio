# learn pin-project

lite 버전으로 공부한다. 

매크로의 향연이다. 잔치다. 난리다. 

# pin project에 대한 블로그 

https://blog.yoshuawuyts.com/safe-pin-projections-through-view-types/

- stack pinning 
- heap pinning 
- pin projection

## what are pin projections

```rust
// before projecting
self: Pin<&mut Sleep { timer: Timer, completed: bool }>

// after projecting
self: &mut Sleep { timer: Pin<&mut Timer>, completed: bool }>
```

이것이 pin project가 하는 일이다. 그런데 왜 필요한가?

```rust
#[pin_project]
pub struct Sleep {
    #[pin]
    timer: Timer,
    completed: bool,
}

impl Future for Sleep {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        assert!(!self.completed, "future polled after completing");

        // This is where the pin projection happens. We go from a `Pin<&mut Self>`
        // to a new type which can be thought of as:
        // `{ timer: Pin<&mut Timer, completed: bool }`
        let this = self.project();
        match this.timer.poll(cx) {
            Poll::Ready(instant) => {
                *this.completed = true;
                Poll::Ready(instant.into())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
```

위 코드에서 `*this.completed = true`를 가능하게 만든다. 왜 project() 없이는 불가능한가? 

컴파일러 에러가 난다. self에 대해 DerefMut 자체가 위법하다. Deref는 괜찮은가? 괜찮다. 

Pin<&mut T>가 적법한 리시버(Receiver)이다. 또 project()를 하면 이 값은 이동된다. 
이후 self에 대한 접근은 불가능하다. project 된 후에 DerefMut가 필드에 대해 
동작하도록 하는 코드가 매크로를 통해 생성된다. 

Pin<T>는 DerefMut를 막는다. as_mut()는 안전하다고 하는데 왜 그럴까?

이와 같이 여러 복잡한 과정을 걸쳐 이동에 안전하게 만든다. 

https://www.youtube.com/watch?v=DkMwYxfSYNQ&t=124s
- 30분 정도에 설명하는 코드가 있다. 

거의 맞게 이해했지만 혼자 공부할 때 어려움은 확신에 있다. 



