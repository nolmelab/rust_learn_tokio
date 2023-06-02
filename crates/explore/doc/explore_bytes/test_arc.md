# test_arc

explore_bytes의 distraction으로 Arc를 본다. 

- 모듈 문서 읽기 
- doctest 코드 연습과 변형 
- pub 함수들 사용법
- pub 내부 구현 이해 
- core 내부는 절제해서 봄 

## E1. 모듈 문서 읽기 

- Arc<T> == std::shared_ptr<T>
- inner value 
- immutable 
  - use Mutex, RwLock, Atomic 

- thread safety 
  - Rc<T> : thread unsafe version of Arc<T>
  - Rc<T> is not Send 
  - Arc<T> is Send and Sync as long as T is
    - !Send, !Sync일 경우 Arc<Mutex<T>>와 같이 Sync를 inner value에 대해 보장하는 타잎을 사용
- Weak<T> == std::weak_ptr<T>

- 


## E2. doctest 코드 연습과 변형 


## E3. pub 함수들 사용법


## E4. pub 내부 구현 이해 


## E5. core 내부 일부 


