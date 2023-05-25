# main

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }

                socket
                    .write_all(&buf[0..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
```

connect 예제보다 훨씬 더 단순합니다. 통신 프로그램을 작성하려면 코덱을 사용하는 FramedRead, 
FramedWrite를 사용해야 하므로 connect 예제가 더 나아 보입니다. 

connect와 echo를 사용하여 에코 통신을 하는 걸 테스트 하면서 더 살펴보는 걸 
연습으로 합니다. 

## 러스트 디버깅 

러스트의 생태계의 중요한 개발도구들이 다 맘에 드는데 디버깅은 아직 vscode에서 만족스럽지 
않습니다. 변수들 내용을 찾기가 까다롭고 코드 진행도 매끄럽지 않고 어셈블리 화면으로 
넘어가는 경우도 있습니다. 

vscode 기준으로 살펴보면서 부족한 부분들을 나열하고 해결 방법을 찾아 보겠습니다. 

echo 서버의 socket.read() 부분에 중단점을 설정하고 connect로 접속하면 중단점에서 
멈춥니다. 여기서 F10으로 진행하면 어셈블리 화면이 나옵니다. 

<core::future::from_generator::GenFuture<T> as core::future::Future>::poll() 함수가 
그렇습니다. 왜 그럴까요?

WSL이 설치되어 있어 리눅스 환경에서는 어떤지 살펴보겠습니다. 

- rustup 설치 
- vscode 확장 설치
  - rust analyzer 설치 
  - code lldb 설치

아마도 소스 매핑 정도가 달라서 생기는 문제로 보입니다. 
- https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#source-path-remapping

WSL에서 약간 느리기 하나 future::poll로 넘어갔습니다. 소스 매핑만 잘 되면 
윈도우에서도 동작해야 합니다. 어떻게 매핑을 할까요?





