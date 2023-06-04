# test bytes_mut

## E1. Module doc

> A unique reference to a contiguous slice of memory.
> Owners of `BytesMut` handles are able to mutate the memory.

## E2. pub fn

```rust
impl BytesMut: 
- pub fn with_capacity(capacity: usize) -> Self 
- pub fn new() -> Self 
- pub fn len(&self) -> Self 
- pub fn is_empty(&self) -> Self 
- pub fn capacity(&self) -> Self 
- pub fn freeze(mut self) -> Bytes 
- pub fn zeroed(len : usize) -> BytesMut
- pub fn split_off(&mut self, at: usize) -> BytesMut
- pub fn split(&mut self) -> BytesMut
- pub fn split_to(&mut self, at: usize) ->BytesMut
- pub fn truncate(&mut self, len: usize) 
- pub fn clear(&mut self) 
- pub fn resize(&mut self, new_len: usize, value: u8)
- pub unsafe fn set_len(&mut self, len: usize)
- pub fn reseve(&mut self, additional: usize)
- pub fn extend_from_slice(&mut self, extend: &[u8])
- pub fn unsplit(&mut self, other: BytesMut)

impl Drop for BytesMut: 
- drop()

impl Buf for BytesMut: 
- remaining()
- chunk(&self) -> &[u8]
- advance(&mut self, cnt: usize)

impl BufMut for BytesMut: 
- remaining_mut(&self) -> usize
- advance_mut(&mut self, cnt: usize)
- chunk_mut(&mut self) -> &mut UninitSlice
- put<T: crate::Buf>(&mut self, mut src: T)
- put_slice(&mut self, src: &[u8])
- put_bytes(&mut self, val: u8, cnt: usize)

impl AsRef<[u8]> for BytesMut:
- fn as_ref(&self) -> &[u8]

impl Deref for BytesMut: 
- fn deref(&ref) -> &[u8]

impl AsMut<[u8]> for BytesMut: 
- fn as_mut(&mut self) -> &mut [u8]

impl DerefMut for BytesMut: 
- fn deref_mut(&mut self) -> &mut [u8]

- impl From for BytesMut 
  - &'a [u8]
  - &'a str

- impl From for Bytes 
  - BytesMut

- impl Borrow<[u8]> for BytesMut
- impl BorrowMut<[u8]> for BytesMut
- impl fmt::Write for BytesMut
- impl Clone for BytesMut 
- impl IntoIterator for BytesMut 
- impl Extend<u8> for BytesMut
- impl Extend<&'a u8> for BytesMut
- impl Extend<Bytes> for BytesMut
- impl FromIterator<u8> for BytesMut
- impl FromIterator<&'a u8> for BytesMut

- impl From<BytesMut> for Vec<u8>
```

BytesMut는 변경 가능한 BufMut이다. 

메모리가 하나로 고정되고 여러 ByetsMut를 동일 데이터에 대해 갖는다. 
여기서 safety 문제가 발생할 수 있으므로 주의해서 사용해야 한다. 

예를 들어, split 함수들로 같은 메모리를 갖는 여러 BytesMut를 가질 경우 
promote_to_shared(), increment_shared()와 같은 도구로 여러 곳에서 
볼 수 있다. 

메모리가 변경될 경우는 어떻게 되는가? 그게 가능한가? 

## E3. 내부 구현 이해 


