macro_rules! read_fn {
    ($($vis:vis fn $name:ident() -> $ty:ty ;)+) => {
        $(
            #[inline(always)]
            $vis fn $name(src: &mut dyn ::std::io::Read) -> std::io::Result<Option<$ty>> {
                let mut buf = [0; ::std::mem::size_of::<$ty>()];
                match src.read_exact(&mut buf) {
                    Ok(_) => Ok(Some(<$ty>::from_be_bytes(buf))),
                    Err(error) if error.kind() == ::std::io::ErrorKind::UnexpectedEof => Ok(None),
                    Err(error) => Err(error),
                }
            }
        )+
    };
}

read_fn!(
    pub(crate) fn read_u8() -> u8;
    pub(crate) fn read_u16() -> u16;
);
