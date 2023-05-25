# main

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
```
`#[tokio::main]`은 토키오 런타임을 초기화하고 `main` 함수를 `Future`로 호출하는 
코드를 생성하는 속성 매크로(attribute macro)입니다.

```rust
    // Determine if we're going to run in TCP or UDP mode
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let tcp = match args.iter().position(|a| a == "--udp") {
        Some(i) => {
            args.remove(i);
            false
        }
        None => true,
    };
```

`env::args()` 함수는 `Args`를 돌려주고, `Args`는 `Iteartor`를 구현하니다. 
`skip()`은 `Iterator`의 함수이고 `collect`는 `Vec`의 `FromIterator` 구현으로 
동작합니다. 


```rust
    // Parse what address we're going to connect to
    let addr = args
        .first()
        .ok_or("this program requires at least one argument")?;
    let addr = addr.parse::<SocketAddr>()?;
```

`addr`의 타잎은 `&String`이 됩니다. 이는 `ok_or`() 뒤에 `?` 연산자로 `unwrap()`하기 때문입니다. 
`first()`는 Vec가 구현한 slice에 대해 동작합니다. (과제 / 연습 참조)

`String::parse()`는 `FromString` 트레이트로 파싱합니다. 큰 물고기 문법(big fish syntax)으로 
타잎을 지정하면 그 타잎의 FromString 구현이 불립니다. 여기서는 
`SocketAddr`이고 `IP:port` 형식을 파싱해서 `SocketAddr`로 만듭니다. 

위 코드에서 가리기(shadowing)로 `addr`를 다시 사용했습니다. 더 이상 
사용하지 않고 유사한 의미를 갖는 이와 같은 곳에는 적절한 기법으로 보입니다.

```rust
    let stdin = FramedRead::new(io::stdin(), BytesCodec::new());
    let stdin = stdin.map(|i| i.map(|bytes| bytes.freeze()));
    let stdout = FramedWrite::new(io::stdout(), BytesCodec::new());

    if tcp {
        tcp::connect(&addr, stdin, stdout).await?;
    } else {
        udp::connect(&addr, stdin, stdout).await?;
    }

    Ok(())
}
```

`BytesCodec`은 사용 가능한 바이트를 입력에서 버퍼로, 버퍼에서 출력으로 옮겨주는 
코덱이니다. 

`FramedRead::new()`는 `AsyncRead`를 `inner`로 `Decoder` 함께 받아서 처리합니다. 

`FramedRead`는 `Stream (poll_next)`, `Sink (poll_ready, start_send, poll_flush, poll_close)`를 구현합니다. `poll_next()`로 다음 코덱의 아이템을 얻는 기능이 읽기에 해당합니다. 

`FramedWrite`도 유사한 구조로 예상합니다. 자세한 정리는 통신을 만들고 (에코 수준의) 
다시 보겠습니다.

<details>

<summary> 과제 / 연습 </summary>

- `Vec<_>.first()`가 어떻게 `slice::first()`라고 알 수 있을까요?

  - [스택 오버플로우의 설명](https://stackoverflow.com/questions/39785597/how-do-i-get-a-slice-of-a-vect-in-rust)을 참고하세요.
  - deref coercion도 검색하여 이해하세요. 
  - coerciion과 elision 규약들이 자동이므로 명확하게 구조화하여 기억하세요

</details>
