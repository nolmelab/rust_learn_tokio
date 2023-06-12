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
    // async fn 들을 어떻게 단위 테스트할 수 있는가?
    // 그 기능을 알아야 정확하게 쓸 수 있다. 

    #[test]
    fn test_json_deserialize() {

    }

    #[test]
    fn test_json_serialize() {

    }

}