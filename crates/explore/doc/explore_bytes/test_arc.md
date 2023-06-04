# test_arc

explore_bytes의 distraction으로 Arc를 본다. 

- 모듈 문서 읽기 
- doctest 코드 연습과 변형 
- pub 함수들 사용법
- pub 내부 구현 이해 
- core 내부는 절제해서 봄 

## E1. 모듈 문서 읽기 

- Arc<T> == std::shared_ptr<T>
- inner value 
- immutable 
  - use Mutex, RwLock, Atomic 

- thread safety 
  - Rc<T> : thread unsafe version of Arc<T>
  - Rc<T> is not Send 
  - Arc<T> is Send and Sync as long as T is
    - !Send, !Sync일 경우 Arc<Mutex<T>>와 같이 Sync를 inner value에 대해 보장하는 타잎을 사용
- Weak<T> == std::weak_ptr<T>

- Deref 
  - Deref trait는 deref(&self) -> &T로 T를 빌리는 함수만 있다. 
  - Deref가 구현되면 여러 곳에서 자동으로 deref() 호출을 한다. 
  - `let v = arc[0]; `와 같은 구문이 가능하다. 
  - 러스트에서 &는 거의 포인터와 같다.  
  - Arc<T>와 &T 간에 ambiguity가 있으면 Fully qualified name을 사용한다.



## E2. doctest 코드 연습과 변형 

- Arc<T>::new() 
- Arc<T>::clone()



## E3. pub 함수들 사용법


## E4. pub 내부 구현 이해 

### structs

```rust
pub struct Arc<T: ?Sized> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}
```

NonNull로 NonNull이 보장되는 포인터를 갖는다. 
PhantomData<ArcInner<T>>로 ArcInner<T>를 소유함을 명시한다. 

<details>
<summary> Q. 왜 PhantomData로 표기해야 하는가? </summary>

라이브러리 코드 문서가 가장 정확하다. 

> Zero-sized type used to mark things that "act like" they own a `T`.
> Adding a `PhantomData<T>` field to your type tells the compiler that your
> type acts as though it stores a value of type `T`, even though it doesn't
> really. This information is used when computing certain safety properties.

nomicon을 이해하면 알 수 있다. PhantomData는 라이프타임 표기와 소유권 
표기 두 가지로 사용한다. 소유권 표기와 관련하여 Drop 트레이트 구현 여부로 
소유권 체크를 하는 방법이 있으나 여전히 표준 라이브러리에서 PhantomData를 
사용하고 있다. 실제 NonNull<ArcInner<T>>에 대한 해제는 Arc의 Drop 구현에서 
할 것이다. 
</details>

Arc<T>는 ?Sized + Send + Sync인 T에 대해 Send, Sync를 구현한다. 

```rust
struct ArcInner<T: ?Sized> {
    strong: atomic::AtomicUsize,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    weak: atomic::AtomicUsize,

    data: T,
}
```

ArcInner<T>는 data와 ref를 추적하기위한 것이다. 왜 strong과 weak과 필요한가?
ArcInner<T>는 ?Sized + Send + Sync인 T에 대해 Send, Sync를 구현한다. 

```rust
pub struct Weak<T: ?Sized> {
    // This is a `NonNull` to allow optimizing the size of this type in enums,
    // but it is not necessarily a valid pointer.
    // `Weak::new` sets this to `usize::MAX` so that it doesn’t need
    // to allocate space on the heap.  That's not a value a real pointer
    // will ever have because RcBox has alignment at least 2.
    // This is only possible when `T: Sized`; unsized `T` never dangle.
    ptr: NonNull<ArcInner<T>>,
}
```

Weak는 ArcInner<T>를 PhantomData<ArcInner<T>>로 ArcInner<T>에 대한 소유권을 
명시하지 않는다.  

### impl 

#### pub fn new(data: T) -> Arc<T>

  - Box를 사용하여 힙에서 ArcInner를 T를 포함한 상태로 할당
  - Box::leak()로 누수시켜서 Arc의 NonNull로 이동

```rust
unsafe { Self::from_inner(Box::leak(x).into()) }
```

In this line, Box::leak is indeed used to convert the Box into a raw pointer, and then 
into() is called to convert the raw pointer into a Arc<T>.

Here's a more detailed explanation:

Box::leak(x) takes ownership of the Box x and "leaks" it, which means that it converts 
the Box into a raw pointer without freeing the memory. This raw pointer has a 
'static lifetime.

into() is called on the raw pointer returned by Box::leak(x). This conversion is possible
because into() is an inherent method of the *const T type (raw pointer) that converts it
into the desired type *const ArcInner<T>.

Self::from_inner(...) is called to create an Arc<T> from the converted raw pointer. The
from_inner function is an associated function of the Arc<T> type and is likely defined 
within the same module or struct as the new function. It takes a raw pointer to the ArcInner<T> and constructs an Arc<T> by incrementing the reference counts appropriately.

The usage of unsafe indicates that the code is performing low-level operations that 
require manual memory management and bypass some of Rust's safety guarantees. 
The implementation of from_inner likely includes the necessary safety checks to ensure 
correct handling of the reference counts and the lifetime of the underlying data.

무서운 ChatGPT.

#### pub fn new_cyclic(data_fn : F) where F : FnOnce(&Weak<T>) -> T

러스트 빌림 관련 우회의 끝판왕이다. 사용은 
```rust
struct Gadget {
  me: Weak<Gadget>
}
```

위와 같이 자기 참조를 갖는 struct에 대한 Arc<Gadget>이 필요한 경우이다. 


## E5. core 내부 일부 


