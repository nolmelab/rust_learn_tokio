# test cursor 

Cursor는 Read / Write에 Seek 능력을 부여한다. 

> A `Cursor` wraps an in-memory buffer and provides it with a
> [`Seek`] implementation.

## E1. pub fun / impl traits

```rust
- impl<T> for Cursor<T>:
  - pub const fn new(inner: T) -> Cursor<T>
  - pub fn into_inner() -> T
  - pub const fn get_ref(&self) -> &T 
  - pub fn get_mut(&mut self) -> &mut T
  - pub const fn position(&self) -> u64
  - pub fn set_position(&mut self, pos: u64)

- impl<T> for Cursor<T> where T : AsRef<[u8]>
  - pub fn remaining_slice(&self) -> &[u8]
  - pub fn is_empty(&self) -> bool

- impl<T> Clone for Cursor<T>

- impl<T> io::Seek for Cursor<T>
  - fn seek(&mut self, style : SeekFrom) -> io::Result<u64>
  - fn stream_len(&mut self) -> io::Result<u64>
  - fn stream_position(&mut self) -> io::Result<u64>

- impl<T> Read for Cursor<T>
  - fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> 
  - fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> io::Result<()>
  - fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize>
  - fn is_read_vectored() -> bool
  - fn read_exact(&mut self, buf : &mut [u8]) -> io::Result<()>

- impl<T> BufRead for Cursor<T>
   - fn fill_buf(&mut self) -> io::Result<&[u8]>
   - fn consume(&mut self, amt: uszie)
```

## E2. Impl Details


