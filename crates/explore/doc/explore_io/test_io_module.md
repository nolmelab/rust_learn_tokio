# test io module

std::io는 입출력의 핵심이다. explore_io에서 개별 요소들을 보려고 했으나 
크고 또 중요하므로 한꺼번에 보면서 필요한 부분을 나눠서 추가로 본다. 

`Read`와 `Write` 트레이트가 매우 중요하다. 이를 구현하는 구조체를 `Reader`, `Writer`라고 
부른다. 

`Seek`, `BufRead`, `BufReader`, `BufWriter`가 추가로 있다. 

IoSlice와 IoSliceMut는 플래폼의 효율적인 Io wrapper이고 윈도우에서는 WSABUF를 사용하는 
IOCP 버퍼이다. 모든 std 내부를 이해하고 코딩을 하겠다는 건 어리석은 생각이다. 참참참. ㅎ

## E1. pub 

```rust
- pub trait Read
- pub fn read_to_string<R: Read>(mut reader: R) -> Result<String>
- pub struct IoSliceMut<'a>
- pub struct IoSlice<'a>

- pub trait Write 
- pub trait Seek 
- pub enum SeekFrom 
- pub trait BufRead: Read 
- pub struct Take<T>
- pub struct Bytes<R>
```

## 버퍼 

러스트에서 다루는 대상은 trait들이고 많은 trait들이 있다. 실제 적용되는 트레이트가 
어떤 것인지, 어느 트레이트를 사용해야 하는지 어려울 때가 많다. 이에 익숙해져야 
편하게 코딩할 수 있다. 

또 다른 하나는 컴파일러와 상호 작용하는 부분이다. 우회하거나 컴파일러의 기능을 
사용하거 할 때 확신을 갖지 어려운 점이 있다. 

약간 상위 레벨에서 코딩을 좀 하는 것이 좋겠다. 

