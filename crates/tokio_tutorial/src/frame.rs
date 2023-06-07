use mini_redis::Frame;
use std::io::Cursor;

fn main() {
  // Cursor를 만들고 parse로 결과를 확인한다. 
  // 좋은 연습이다. 

  let mut frames = Vec::<u8>::new();

  frames.push(b'+');

  // as_ref()를 &b"new line"[..] 대신에 사용할 수 있다.
  frames.extend_from_slice(b"new line\r\n".as_ref());
  frames.extend_from_slice("+hello\r\n".to_string().as_bytes());

  // Vec<T>에서 &[T] 슬라이스 참조를 얻을 때도 as_ref() 트레이트 구현이 편리하다.
  let mut cursor = Cursor::new(frames.as_ref());

  // new line
  let result = Frame::parse(&mut cursor);
  if let Ok(frame) = result {
    println!("Parsed a frame: {}", frame);
  }
  
  // hello
  let result = Frame::parse(&mut cursor);
  if let Ok(frame) = result {
    println!("Parsed a frame: {}", frame);
  }
}