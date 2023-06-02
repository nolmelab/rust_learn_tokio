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










