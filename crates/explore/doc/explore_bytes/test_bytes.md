# test_bytes

## use

```rust
use core::iter::FromIterator;
use core::ops::{Deref, RangeBounds};
```

- Deref
- RangeBounds

```rust
use core::{cmp, fmt, hash, mem, ptr, slice, usize};
```

```rust
use alloc::{
    alloc::{dealloc, Layout},
    borrow::Borrow,
    boxed::Box,
    string::String,
    vec::Vec,
};

use crate::buf::IntoIter;
#[allow(unused)]
use crate::loom::sync::atomic::AtomicMut;
use crate::loom::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use crate::Buf;
```

C++에서 functional에서 include한 부분은 아래와 같다. 

```c++
#include <yvals_core.h>
#include <exception>
#include <tuple>
#include <typeinfo>
#include <xmemory>
#include <xstddef>
#include <unordered_map>
#include <compare> 
```

functional은 조금 복잡할 수 있는 구현이다. 

memory는 어떤가? 

```c++
#include <exception> 
#include <iosfwd>
#include <typeinfo>
#include <xmemory>
```

그 내부 구현은 어떠한가? 
_Altorithm_int_t<_Diff> _Count = _Count_raw에서 _Algorithm_int_t는 무엇인가? 
C++로 코딩할 때 라이브러리를 진정으로 다 이해하고 코딩한 적이 있는가? 

위 정도의 use면 C++에 비해 core와 alloc의 몇 개에 해당하는 적은 수의 의존을 갖는다. 


## 탐색의 진행 

러스트를 매우 잘 사용하는 것이 목표이다. AI의 시대라고 하지만, 여전히 고전적인 
프로그래밍으로도 마법을 시전하여 사람들에게 유용한 도구를 만들 수 있다. 특히,
내가 하고자 하는 사람들이 함께 어울려 노는 공간에 AI를 통합한 도구를 만들 수 있다. 

- use를 다 보는 건 천천히 한다. 
  - 특히, core에 사로잡히지 않는다. 
  - 어떻게 사용했는지 위주로 본다. 
  - 그래도 사용하는 트레이트와 중요 struct는 이해해야 한다. 
  - 따라서, 마지막에 본다 

- E1. 문서를 꼼꼼히 읽는다. 
  - C++과 달리 문서화를 매우 꼼꼼하게 잘 한다. 
  - 커뮤니티의 차이이자 언어의 차이다. 
    - C++은 구현을 문서화하기가 거의 불가능하다. 사용법이라도 잘 정리되면 다행 

- E2. doctest로 제시된 모듈의 문서 코드를 이해한다.
  - 다양한 변형을 테스트 한다. 
  - 동작을 이해하기위한 여러 테스트를 작성한다. 
  - 창의적으로 멈추지 않고 여러 테스트를 추가한다. 

- E3. 모든 pub 함수를 위와 같이 테스트 한다. 
  - experimental, unstable은 제외하고 진행한다. 

- E4. 모든 pub 함수의 구현을 이해한다. 
  - 트레이트 간의 관계 
  - 다른 구조화의 방법들 

## E1. 모듈 문서 

### Module::Memory Layout 

- 4 usize fields 
  - used to track information about which segment of the underlying memory the 
    `Bytes` handle has access to. 

- a pointer to the shared state containing the full memory slice 
- a pointer to the start of the region visible by the handle 
- the length of its view into the memory

4 usize 필드를 갖는다. 

### Module::Sharing

- `Bytes` contains a vtable, which allows implementations of `Bytes` to define 
  how sharing/cloning is implemented in detail 

- backing storage에 대한 view가 `Bytes`이다. 어디서 많이 들어본 듯. Slice와 매우 
  비슷하다. 

- backing storage의 예시로 static memory와 Arc<[u8]>을 얘기한다. 
  - Arc<[u8]>을 어떻게 만드는지 확인하기위해 test_arc.rs 탐색을 했다. 
    - Arc를 보면서 PhantomData에 대해 한번 더 보았다. 여전히 노미콘을 이해해야  하는 
      문제가 남았고 4일 연휴 동안 봐야겠다. 

## E3. pub 메서드, trait

methods:

```rust
- impl Bytes: 
  - pub const fn new() -> Self
  - pub fn new() -> Self 
  - pub const fn from_static(bytes : &'static [u8]) -> Self
  - pub fn from_static(bytes : &'static [u8]) -> Self
  - pub const fn len() -> usize
  - pub const fn is_empty() -> bool
  - pub fn copy_from_slice(data: &[u8]) -> Self
  - pub fn slice(&self, range : impl RangeBounds<usize>) -> Self 
  - pub fn slice_ref(&self, subset: &[u8]) -> Self
  - pub fn split_off(&mut seff, at: usize) -> Self
  - pub fn split_to(&mut self, at : usize) -> Self 
  - pub fn truncate(&mut self, len: usize) 
  - pub fn clear(&mut self) 
```

traits:

```rust
- impl Buf for Bytes:
  - remaining()
  - chunk()
  - advance()
  - copy_to_bytes()
```

```rust
- Default 
- From<&'static [u8]>
- From<&'static str>
- From<Vec<u8>>
- From<Box<[u8]>>
- From<String>
- From<Bytes> for Vec<u8>
- 
```

Bytes는 Buf이며 연속된 메모리에 대한 Slice와 비슷하여 Slice / Vec / Box에서 
생성하고 쪼개기가 가능하다. 

## E4. pub struct와 내부 구현 

- VTable 기법
  - 이 쪽은 보기 좀 어렵다. 도전 과제로 남겨둔다. 
  - vtable 자체는 함수 포인터 지정이라 어렵지 않다. 
  - clone() 등의 내부 구현이 어렵다. 

- PartialEq, PartialOrd 
  - 많은 타잎 특수화에 대한 구현이 있다. 

















