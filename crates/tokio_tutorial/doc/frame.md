# frame 

```rust
#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}
```

```rust
Frame::check(src: &mut Cursor<&[u8]>) -> Result<(), Error>
```

Cursor로 둘러싸서 get_line(src)를 하더라도 내부 버퍼가 변경되지 않는다. 
별도로 분리된 뷰에서 Frame으로 파싱 가능한지 확인한다. 

parse() 함수도 비슷하고 실제 프레임을 만든다. 

Connection::read_frame()에서 stream.read_buffer()로 self.buffer를 채운다. read_buffer()는 stream에서 읽기를 비동기로 진행한다. 

stream은 TcpStream이고 AsyncReadExt를 구현한다. ReadBuf가 Future이고 이를 통해 비동기
처리가 동작한다. 약간 복잡하지만 C++처럼 이해 못 할 부분은 없다. 

frame.rs에서 +에 대한 단위 테스트를 parse()에 대해 구현했다. 
이와 같이 직접 구현해야 실력이 는다. 

## ReadBuf의 Future 동작 



