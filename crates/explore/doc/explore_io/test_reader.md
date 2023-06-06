# test reader

```rust
 let mut fs = File::create("test.txt")?;
 // let _result = writer.write(&[1, 2, 3]);
 writeln!(fs, "Hello File!")?;
 writeln!(fs, "Hello File 2!")?;
 Ok(())
```

text 파일, 바이너리 파일 모두 같은 옵션으로 비슷하게 처리할 수 있다. 

```rust
 let fs = File::open("test.txt")?;
 let reader = BufReader::new(fs);
 for line in reader.lines() {
   if let Ok(l) = line {
     println!("{}", l);
   }
 }
 
 Ok(())
```

