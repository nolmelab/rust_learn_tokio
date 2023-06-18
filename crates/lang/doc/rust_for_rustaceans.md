# rust for rustaceans

# Types 

## Generic Traits 

- Type parameters 
  - Monomorphization
  - 타잎별 구현이라는 매우 강력한 도구를 제공한다. 

- Associated types
  - one implementation
  - code generated for that trait 
  - 내가 제어할 때 사용한다는 다른 설명도 괜찮았다. 


## Coherence and Orphan Rules

Def. Coherence Rule: 
> for any given type and method, there is only ever one
> correct choice for which implementation of the method to use for that
> type.

Def. Orphan Rule: 
> The orphan rule says that you can implement a trait for a type only
> if the trait or the type is local to your crate. 

Debug -> outside 
bool -> outside 

그러므로 위에 대해 내가 직접 구현할 수 없다. 둘 중의 하나는 로컬이어야 한다. 

### Blanket Implementations 

Generic type parameter의 타잎에 대해 트레이트를 구현하는 걸 blanket 구현이라고 한다. 

동일한 규칙이 적용된다. 

### Fundamental Types

#[fundamental] 속성을 갖는 타잎들이다. &, &mut, Box가 그렇다. 

IntoIterator for &MyType에서 IntoIterator가 외부이고 &가 외부 타잎이라 고아 규칙에
어긋난다. (&만 떼어서 따로 타잎으로 본다는 점이 특이하다)

"다른 쪽을 침범할 가능성이 없으면 된다고 생각해도 된다" 규칙으로 봐도 되겠다.

### Covered Implementations

특수한 상황에 다른 타잎으로 고아가 될 수 있다면 허용한다는 규칙이다. 

> a given impl<P1..=Pn> ForeignTrait<T1..=Tn> for T0 is allowed only if at least one Ti
> is a local type and no T before the first such Ti is one of the generic types
> P1..=Pn. Generic type parameters (Ps) are allowed to appear in T0..Ti as long
> as they are covered by some intermediate type. A T is covered if it appears as
> a type parameter to some other type (like Vec<T>), but not if it stands on its
> own (just T) or just appears behind a fundamental type like &T. 


```rust
impl<T> From<T> for MyType
impl<T> From<T> for MyType<T>
impl<T> From<MyType> for Vec<T>
impl<T> ForeignTrait<MyType, T> for Vec<T>
```

뭔가 로컬로 표시가 가능하면 된다고 이해하면 되겠다. 다른 쪽을 침범할 가능성이 없으면 
된다고 생각해도 된다. 

타이 브레이커가 필요해서 로컬 타잎이 타잎 파라미터의 제일 앞에 오도록 했다. 

## Trait Bounds 

러스트는 매우 자유롭고 상세한 트레이트 제약을 지정할 수 있다. 타잎별 트레이트 구현이 
가능하고, 트레이트 구현이 다른 코드와 분리되고, 트레이트 제약이 자유롭다는 점이 
러스트의 트레이트를 강력하게 만든다. 

## Existential Types 

impl Trait로 리턴 타잎을 지정하는 경우에 해당한다. 아마도 더 폭 넓은 개념인 듯 한데 
지금은 그 정도로 충분해 보인다. 

# Designing Interfaces

- unsurprising
- flexible 
- obvious 
- contrained

clippy도 매우 좋다. 깔끔한 코드를 만들도록 한다. 


## Unsurprising (Predictable) 

### Naming Practices

의미 전달 체계로 일관성을 갖춰야 한다. 

### Common Traits for Types

Debug, Clone, Default 

Implement !Send, !Sync if the type is . 

PartialEq, PartialOrd, Hash, Eq, and Ord

Copy는 따로 고려가 필요하다. Move의 의미를 바꾸고 !Copy가 되기 쉽다. 

### Ergonomic Trait Implementations

blanket implementations as appropriate for that trait for &T where T:
Trait, &mut T where T: Trait, and Box<T> where T: Trait.

### Wrapper Types

상속처럼 Deref와 AsRef<T>()가 동작할 수 있다. 파이썬이나 루아에서 상속을 처리하는 방법이기도
하다. 

> Borrow is intended only for when your type is essentially equivalent to another type, 
> whereas Deref and AsRef are intended to be implemented more widely for anything your 
> type can “act as.”

## Flexible 

> you should think carefully about what contract your interface binds you to, 
> because changing it after the fact can be disruptive.

### Generic Arguments







