use std::{collections::HashMap, io, mem};

pub type Code = u16;

pub type EncDict = HashMap<Vec<u8>, Code>;
pub type DecDict = HashMap<Code, Vec<u8>>;

/// Encodes the given data.
///
/// # Errors
///
/// Fails if any of the underlying I/O operations fail (i.e., reading from `src`
/// or writing to `out`).
pub fn enc(src: &mut dyn io::Read, out: &mut dyn io::Write) -> io::Result<()> {
    enc_returning_dict(src, out)?;
    Ok(())
}

#[doc(hidden)]
pub fn enc_returning_dict(src: &mut dyn io::Read, out: &mut dyn io::Write) -> io::Result<EncDict> {
    let mut dict = build_default_enc_dict();
    let mut seq = Vec::<u8>::default();

    // Advance while the next char forms a key which is in the map.
    // When the next char forms a string which is not in the map, emits it
    // and inserts (it + the char) in the map.
    while let Some(c) = read_u8(src)? {
        seq.push(c);
        if !dict.contains_key(&seq) {
            let prev_seq = &seq[..(seq.len() - 1)];
            emit(prev_seq, &dict, out)?;

            let code = dict.len().try_into().unwrap();
            dict.insert(mem::replace(&mut seq, vec![c]), code);
        }
    }
    emit(&seq, &dict, out)?;

    Ok(dict)
}

macro_rules! read_fn {
    ($(fn $name:ident() -> $ty:ty ;)+) => {
        $(
            #[inline(always)]
            fn $name(src: &mut dyn io::Read) -> io::Result<Option<$ty>> {
                let mut buf = [0; mem::size_of::<$ty>()];
                match src.read_exact(&mut buf) {
                    Ok(_) => Ok(Some(<$ty>::from_be_bytes(buf))),
                    Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
                    Err(error) => Err(error),
                }
            }
        )+
    };
}

read_fn!(
    fn read_u8() -> u8;
);

fn emit(seq: &[u8], dict: &EncDict, out: &mut dyn io::Write) -> io::Result<()> {
    let code = Code::to_be_bytes(dict[seq]);
    out.write_all(&code)
}

fn build_default_enc_dict() -> EncDict {
    let mut dict = HashMap::new();
    for i in u8::MIN..=u8::MAX {
        dict.insert(vec![i], i.into());
    }
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_seq() {
        let mut out = Vec::new();
        let mut src = b"ABBABBBABBA".as_ref();
        enc(&mut src, &mut out).unwrap();
        assert_eq!(out, c(&[65, 66, 66, 256, 257, 259, 65]));
    }

    fn c(codes: &[Code]) -> Vec<u8> {
        let mut out = Vec::new();
        for code in codes {
            let data = Code::to_be_bytes(*code);
            out.extend(data);
        }
        out
    }
}
