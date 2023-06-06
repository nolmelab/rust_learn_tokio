#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{self, BufReader, BufWriter, Error, BufRead, Write};

    #[test]
    fn test_write_file() -> Result<(), Error> {
        let mut fs = File::create("test.txt")?;

        // let _result = writer.write(&[1, 2, 3]);
        writeln!(fs, "Hello File!")?;
        writeln!(fs, "Hello File 2!")?;

        Ok(())
    }

    #[test]
    fn test_read_file() -> Result<(), Error> {

      let fs = File::open("test.txt")?;
      let reader = BufReader::new(fs);

      for line in reader.lines() {
        if let Ok(l) = line {
          println!("{}", l);
        }
      }
      
      Ok(())
    }
}
