# mini http 

https://github.com/tokio-rs/tokio-minihttp/tree/master

이전 버전의 tokio로 작성한 mini http이다. 이를 현재 tokio에 맞게 수정한다. 
예전에는 Encoder와 Decoder가 분리되어 있었고, 지금은 Codec으로 합쳐졌다. 
또 다른 차이도 많기 때문에 이를 찾아서 정리해 나가는 과정은 괜찮아 보인다. 

먼저, tcp_json으로 Codec을 연습하고 진행한다. 




