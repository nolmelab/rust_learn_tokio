# tcp::connect 

```rust
mod tcp {
    use bytes::Bytes;
    use futures::{future, Sink, SinkExt, Stream, StreamExt};
    use std::{error::Error, io, net::SocketAddr};
    use tokio::net::TcpStream;
    use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
```

꽤 많은 use가 있습니다. Sink, SinkExt, Stream, StreamExt가 낯섭니다.  
어디에 쓰이는지 아래 connect 처리에서 확인할 수 있습니다. 

```rust
    pub async fn connect(
        addr: &SocketAddr,
        mut stdin: impl Stream<Item = Result<Bytes, io::Error>> + Unpin,
        mut stdout: impl Sink<Bytes, Error = io::Error> + Unpin,
    ) -> Result<(), Box<dyn Error>> {
        let mut stream = TcpStream::connect(addr).await?;
        let (r, w) = stream.split();
        let mut sink = FramedWrite::new(w, BytesCodec::new());
        // filter map Result<BytesMut, Error> stream into just a Bytes stream to match stdout Sink
        // on the event of an Error, log the error and end the stream
        let mut stream = FramedRead::new(r, BytesCodec::new())
            .filter_map(|i| match i {
                //BytesMut into Bytes
                Ok(i) => future::ready(Some(i.freeze())),
                Err(e) => {
                    println!("failed to read from socket; error={}", e);
                    future::ready(None)
                }
            })
            .map(Ok);

        match future::join(sink.send_all(&mut stdin), stdout.send_all(&mut stream)).await {
            (Err(e), _) | (_, Err(e)) => Err(e.into()),
            _ => Ok(()),
        }
    }
}
```

- TcpStream::connect()를 호출하면 TcpStream을 돌려줍니다. 
- stream.split()는 ReadHalf, WriteHalf로 나눕니다. 
- FramedNew()는 Sink<T, T, E>를 구현합니다. 
  - 또, Stream을 구현합니다. Stream은 poll_next()를 구현합니다. 
  - Future와 유사한 면이 있습니다. 
  - split()로 얻은 ReadHalf, WriteHalf는 각 AsyncRead와 AsyncWrite를 구현합니다. 
    - AsyncRead, AsyncWrite는 poll_* 함수들로 읽기와 쓰기 도구를 제공합니다. 
- StreamExt는 Stream을 구현합니다. 
  - StreamExt는 모두 기본 구현을 갖고 있고 Stream을 수퍼 트레이트로 갖습니다. 
  - Stream과 StreamExt의 관계는 특이합니다. 

 future::join으로 하게 되는 일은 입력 받은 걸 전송하고 소켓에서 받은 걸 출력하는 것이다. 따라서, 에코와 같다. 
