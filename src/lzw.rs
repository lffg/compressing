use crate::shared::{read_u16, read_u8};
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
///
/// # Panics
///
/// Panics if the dictionary grows past `u16`'s bounds.
pub fn enc(src: &mut dyn io::Read, out: &mut dyn io::Write) -> io::Result<()> {
    enc_returning_dict(src, out)?;
    Ok(())
}

#[doc(hidden)]
pub fn enc_returning_dict(src: &mut dyn io::Read, out: &mut dyn io::Write) -> io::Result<EncDict> {
    let mut dict = build_default_enc_dict();
    let mut seq = Vec::<u8>::new();

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
    if !seq.is_empty() {
        emit(&seq, &dict, out)?;
    }

    Ok(dict)
}

/// Decodes the given data.
///
/// # Errors
///
/// Fails if any of the underlying I/O operations fail (i.e., reading from `src`
/// or writing to `out`).
///
/// # Panics
///
/// Panics if the dictionary grows past `u16`'s bounds.
pub fn dec(src: &mut dyn io::Read, out: &mut dyn io::Write) -> io::Result<()> {
    let mut dict = build_default_dec_dict();
    let mut seq = Vec::<u8>::new();

    while let Some(code) = read_u16(src)? {
        let decoded = dict.get(&code).map(Clone::clone).unwrap_or_else(|| {
            let mut s = seq.clone();
            s.push(s[0]);
            s
        });
        out.write_all(&decoded)?;

        if !seq.is_empty() {
            let next_code = dict.len().try_into().unwrap();
            dict.insert(next_code, {
                let mut s = mem::take(&mut seq);
                s.push(decoded[0]);
                s
            });
        }

        seq = decoded;
    }

    Ok(())
}

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

fn build_default_dec_dict() -> DecDict {
    let mut dict = HashMap::new();
    for i in u8::MIN..=u8::MAX {
        dict.insert(i.into(), vec![i]);
    }
    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($( ($name:ident, $decoded:expr, $encoded:expr), )+) => {
            paste::paste! {
                $(
                    #[test]
                    fn [< $name _encode >]() {
                        let mut src = ($decoded).as_ref();
                        let mut out = Vec::new();
                        enc(&mut src, &mut out).unwrap();
                        assert_eq!(out, Vec::from($encoded));
                    }

                    #[test]
                    fn [< $name _decode >]() {
                        let src = Vec::from($encoded);
                        let mut out = Vec::new();
                        dec(&mut &*src, &mut out).unwrap();
                        assert_eq!(out, $decoded);
                    }
                )+
            }
        };
    }

    test![
        (
            test_basic_seq_1,
            b"ABBABBBABBA",
            coded(&[65, 66, 66, 256, 257, 259, 65])
        ),
        (test_basic_seq_2, b"ABABA", coded(&[65, 66, 256, 65])),
        (test_basic_seq_3, b"ABABABA", coded(&[65, 66, 256, 258])),
        (
            test_basic_seq_4,
            b"ol\xE1, mundo! como vai?",
            [
                0, 111, 0, 108, 0, 225, 0, 44, 0, 32, 0, 109, 0, 117, 0, 110, 0, 100, 0, 111, 0,
                33, 0, 32, 0, 99, 0, 111, 0, 109, 0, 111, 0, 32, 0, 118, 0, 97, 0, 105, 0, 63
            ]
        ),
    ];

    fn coded(codes: &[Code]) -> Vec<u8> {
        let mut out = Vec::new();
        for code in codes {
            let data = Code::to_be_bytes(*code);
            out.extend(data);
        }
        out
    }
}
