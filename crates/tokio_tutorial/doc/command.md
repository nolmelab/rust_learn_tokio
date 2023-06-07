# command 

Frame에서 Command로 변환 가능한 부분이 있다. 

```rust
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Publish(Publish),
    Set(Set),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
    Unknown(Unknown),
}
```

## Get command

Get::new()에 impl ToString을 사용한다. 

Command::from_frame()에서 Parse를 사용하여 next_string()으로 키를 얻어 만든다. 
따라서, 여기서 impl ToString은 String이다. 

impl에 대한 매칭은 variance를 따르지 않고 trait 구현의 여부에만 의존하는 것으로 보인다. 
명확하게 해야 할 남은 과제 중 하나이다. 

## from_frame() 

command 이름에 따른 dispatching은 러스트에서 흔히 볼 수 있는 패턴이다. 

OOP 언어라면 parse_frames를 Command의 virtual 함수로 만들고 
Get::parse_frames()와 같이 호출해서 사용했을 것이다. 

러스트에서는 실제 마지막 구체적인 타잎까지 들어가서 처리하는 코드를 분기해서 처리하도록 
구현하는 경우가 많다. 이는 C에 가까운 방법이다. 

이제 거의 C++ 만큼 읽힌다. 더 수월한 부분과 어려운 부분이 아직은 섞여 있다. 


