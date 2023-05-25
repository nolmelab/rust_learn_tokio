# use

```rust
use futures::StreamExt;
use tokio::io;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

use std::env;
use std::error::Error;
use std::net::SocketAddr;
```

* `futures::StreamExt`는 `Stream`을 확장한 `trait`입니다.
  * 1692라인에 달하고 다른 여러 모듈을 사용하는 큰 트레이트입니다.&#x20;
  * [futures::StreamExt 문서](https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html)를 보면 `StreamExt`는 `Stream`과 `Future`를 결합한 기능을 제공합니다.
  * 지금은 문서 위주로 살펴보면서 깊은 이해가 필요한 곳이 있으면 더 살펴보도록 합니다.&#x20;
* `tokio::io`는 `AsyncRead`, `AsyncWrite` trait를 제공하는 기능이 핵심입니다.&#x20;
  * `tokio::fs::File`과 함께 동작하는 경우가 많습니다.&#x20;
  * 네트워크나 메모리스트림에 대해서도 구현이 있을 것으로 예상합니다.
* `tokio_util::codec`은 스트림을 해석하여 정보 단위를 얻는 역할을 합니다.&#x20;
  * Frame이 그 단위이며 코덱(Codec)을 통해 만듭니다.&#x20;
  * `FramedRead`는 `Decoder`로 `AsyncRead`에서 읽습니다. 정확한 동작은 디버깅 등으로 추가 분석해야 합니다.&#x20;
  * `FramedWrite`도 비슷할 것으로 예상합니다.&#x20;
  * `BytesCodec`은 unit type ()을 갖는 `struct`입니다.&#x20;
    * `Decoder` 구현은 `BytesMut` 버퍼에 무언가 있으면 전체를 돌려줍니다.
    * `Encoder<Bytes>`, `Encoder<BytesMut>` 구현은 입력에서 바로 출력 버퍼로 옮겨줍니다.&#x20;
    * 따라서, `BytesCodec`은 바이트 단위로 양방향 통신을 하는 연결된 스트림 간에 사용할 수 있습니다.
*
