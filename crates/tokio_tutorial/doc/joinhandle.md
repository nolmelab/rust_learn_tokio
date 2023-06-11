# JoinHandle

```rust
    pub struct JoinHandle<T> {
        raw: RawTask,
        _p: PhantomData<T>,
    }
```

JoinHandle은 Send+Sync이다. 또 UnwindSafe, RefUnwindSafe이다. 

RawTask는 왜 분리해야 할까? 

```rust
- impl: 
  - new() 
  - abort()
  - is_finished() -> bool
  - set_join_waker()
  - id() -> super::Id 
```

JoinHandle은 Unpin이다. 

또 Future이다. 

## Spawning 

튜토리얼의 태스크로 백그라운드에서 실행되는 퓨처를 생성하여 관리하는 방법에 대해 나온다. 

handle에 대해 대기하는 코드의 빌드 에러가 난다. 

JoinHandle을 참조로 외부에서 얻은 다음 이를 통해 접근하기가 어렵다. 

is_finished()를 호출하면서 대기하도록 했는데 마음에 안 든다. 

Rc<JoinHandle<()>>로 벡터에 보관해서 RawTask를 통해 is_finished()를 호출한다. 
항상 안전한가? 그렇지 않아 보인다. 

channel을 두고 종료를 처리하도록 한다. 
channel 패턴은 대규모 시스템에도 적합하다. RabbitMQ나 카프카를 넘어서는 어떤 아키텍처 
구성의 핵심으로 만들 수 있을 듯 하다. 



