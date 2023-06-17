use futures::stream::{StreamExt};
use futures::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::{LengthDelimitedCodec, Decoder};
use std::error::Error;
use std::fmt::{self};
use bytes::Bytes;

#[tokio::main]
async fn main() {
    
}

#[derive(Debug)]
struct FramedError {
}

impl fmt::Display for FramedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FramedError")
    }
}

impl Error for FramedError {

}

async fn process_accepted(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {

    let codec = LengthDelimitedCodec::builder()
        .length_field_offset(0) // length of hdr1
        .length_field_length(4)
        .length_adjustment(0) // length of hdr2
        .num_skip(4) // leaving the header byte , Does not work if 0
        .new_codec();

    // frame의 stream은 AsyncRead + AsyncWrite + Send이다. 
    // AsyncRead는 AsyncReadExt를 통해서 사용. 
    // 
    
    let mut framed = codec.framed(stream);

    while let Some(request) = framed.next().await {
        if let Ok(buf) = request {
            // 이렇게 이동시키면 안 될 것 같긴 한데...
            let bytes : Bytes = buf.into();

            // futures::SinkExt가 send로 확장한다. 
            framed.send(bytes).await?;
        }
        else {
            // error to read into a frame 
        }
    }

    Result::Err(Box::new(FramedError{}))
}

// TODO: 
// - async fn의 단위 테스트 
// - SinkExt와 StreamExt 살피기
// 

#[cfg(test)]
mod json {
    use tokio::fs::{self, File};
    use tokio::io::AsyncWriteExt; // for write_all()
    use tokio::io::Error;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_tokio_fs() {
        let result = fs::write("test.bin", b"hello").await;
        assert!(result.unwrap() == ());

        let result = fs::read("test.bin").await;
        if let Ok(vec) = result {
            let result = String::from_utf8(vec);
            if let Ok(s) = result {
                assert!(s == "hello");
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_file_write_all() -> Result<(), Error> {
        let mut file = File::create("test.bin").await?;
        file.write_all(b"hello2").await?;

        Ok(())

        // AsyncRead, AsyncWrite는 거의 Future이고 
        // AsyncReadExt, AsyncWriteExt는 Future를 돌려주는 함수들을 갖는다. 
        // AsyncRead, AsyncWrite이면 Ext들을 통해 await할 수 있다. 
    }

    #[test]
    fn test_json_serialize() {
        // File이 AsyncRead, AsyncWrite이다. 
        // Buf를 직접 제어하는 방법이 있을까?
        // 

    }


}