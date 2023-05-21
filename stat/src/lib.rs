use std::io::{Read, Write};

/// A reader/writer that collects statistics on reads and writes.
#[derive(Debug)]
pub struct Stat<I> {
    inner: I,
    read_count: u64,
    write_count: u64,
}

impl<I> Stat<I> {
    /// Constructs a new [`Stat`].
    pub fn new(inner: I) -> Self {
        Self {
            inner,
            read_count: 0,
            write_count: 0,
        }
    }

    /// Returns the number of bytes read.
    pub fn read_count(&self) -> u64 {
        self.read_count
    }

    /// Returns the number of bytes written.
    pub fn write_count(&self) -> u64 {
        self.write_count
    }

    /// Returns the inner reader/writer.
    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl<I: Read> Read for Stat<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.read_count += u64::try_from(n).unwrap();
        Ok(n)
    }
}

impl<I: Write> Write for Stat<I> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.write_count += u64::try_from(n).unwrap();
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufWriter};

    use super::*;

    #[test]
    fn test_read() {
        let mut src = "olá, mundo!".as_bytes();
        let mut stat_r = Stat::new(&mut src);

        io::read_to_string(&mut stat_r).unwrap();

        assert_eq!(stat_r.read_count(), 12);
        assert_eq!(stat_r.write_count(), 0);
    }

    #[test]
    fn test_write() {
        let out = Vec::<u8>::new();
        let mut stat_w = Stat::new(out);

        write!(&mut stat_w, "olá, mundo!").unwrap();

        assert_eq!(stat_w.read_count(), 0);
        assert_eq!(stat_w.write_count(), 12);
        assert_eq!(stat_w.into_inner(), "olá, mundo!".as_bytes());
    }

    #[test]
    fn test_three_level_composition_with_buffering() {
        let out = Vec::<u8>::new();
        let stat = Stat::new(out);
        let mut buf_w = BufWriter::with_capacity(5, stat);

        write!(&mut buf_w, "olá, mundo!").unwrap();

        let stat_w = buf_w.into_inner().unwrap();

        assert_eq!(stat_w.read_count(), 0);
        assert_eq!(stat_w.write_count(), 12);
        assert_eq!(stat_w.into_inner(), "olá, mundo!".as_bytes());
    }
}
