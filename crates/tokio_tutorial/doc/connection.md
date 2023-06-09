# Connection 

mini_redis::Connection은 미니 레디스에서 실제 프로토콜을 처리하는 역할을 맡습니다. 

Connection의 구현을 자세히 살펴보는 일은 흥미롭고 다른 프로토콜을 구현할 때도 많은 
도움이 됩니다. 왜냐하면 모든 통신이 프레임과 버퍼 처리를 갖고 있기 때문입니다. 

## use 

```rust
use crate::frame::{self, Frame};

use bytes::{Buf, BytesMut};
use std::io::{self, Cursor};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
```

bytes에서 Buf, BytesMut를 가져옵니다. bytes 패키지는 버퍼 관리를 매우 효율적으로 러스트에 
맞게 잘 구현한 라이브러리입니다. 문서의 설명을 믿고 잘 사용하는 것에서 시작해서 내부 
구현까지 이해하면 러스트 이해에 상당한 도움이 됩니다. 

### Cursor<T>

```rust
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}
```

Cursor의 정의는 위와 같습니다. T를 소유하고 현재 위치를 갖고 있습니다. 

Cursor에 대한 한 줄 설명입니다. cursor.rs의 주석에 있습니다. 

`Cursor` wraps an in-memory buffer and provides it with a [`Seek`] implementation.

파일 포인터로 위치를 옮기면서 읽고 쓰는 것과 비슷합니다. 

AsRef 제약을 갖는 경우 Read, Write 트레이트를 구현하고 fs::File에 대한 Mockup으로 
Cursor를 쓸 수 있는 등 트레이트 기반의 러스트의 강력한 추상화를 활용하는 좋은 예시이기도
합니다. 

### tokio::io::AsyncReadExt 

Read들, AsyncRead를 use로 포함한다.  BufMut도 사용한다. 
내부에 ReadBuf가 있다. MaybeUninit 버퍼를 사용한다. ReadBuf는 Future이다.  

AsyncReadExt는 AsyncRead를 구현하고 AsyncRead는 poll_read()를 갖고 있다. 
Future와 비슷한 메서드인데 어떻게 사용하는걸까? 

### tokio::io::AsyncWriteExt

AsyncWriteExt는 AsyncWrite를 super trait로 갖는다. 
AsyncWriter는 poll_write(), poll_flush(), poll_shutdown(), poll_write_vectored(), 
is_Write_vectored()를 갖는다. 

AsyncWriteExt는 write(), write_vectored(), wirte_buf(), write_all_buf(), 
wirte_all(), flush(), shutdown()을 갖는다. 

AsyncWrite인 모든 것은 AsyncWriteExt가 구현되어 있다. 

### tokio::io::BufWriter

BufWriter의 구현은 흥미롭다. 

```rust
    pub struct BufWriter<W> {
        #[pin]
        pub(super) inner: W,
        pub(super) buf: Vec<u8>,
        pub(super) written: usize,
        pub(super) seek_state: SeekState,
    }
```

pin_project_lite를 사용하여 inner에 대해 Pin 안전하게 만든다. 

frame.rs가 프로토콜 처리를 한다. 
Cursor<T>, BytesMut, ReadBuf 등 많은 코드들이 연관된다. 

Frame::parse(), Frame::check(), get_line(), get_decimal() 등의 함수가 핵심이다. 
parse()는 Cursor와 BytesMut를 사용하여 처리한다. 

## read_frame() 함수 

이 함수가 처리의 핵심이다. Frame과 ReadBuf를 사용하여 처리한다. 
이 점이 기묘하면서 재미있는 부분이다. 

















