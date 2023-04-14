use std::{
    io::{Cursor, Write},
    marker::PhantomData,
};

pub use crate::error::{Error, Result};

mod error;

const MAGIC_PREFIX: &[u8] = b"\x93NUMPY";
const ARRAY_ALIGN: usize = 64;
const GROWTH_AXIS_MAX_DIGITS: usize = 21;

const V30_BYTES: VersionBytes = &[3, 0];
const V30_PREFIX_LEN: usize = MAGIC_PREFIX.len() + V30_BYTES.len();
const V30_HEADER_BEGIN: usize = V30_PREFIX_LEN + std::mem::size_of::<V30HeaderLentype>();

type VersionBytes = &'static [u8; 2];
type V30HeaderLentype = i32;

pub trait DType: Sized {
    fn descr() -> &'static str;

    fn write_to<W: Write>(self, w: &mut W) -> std::io::Result<usize>;
}

macro_rules! impl_dtype {
    ($rty:ty, $npty:literal) => {
        impl DType for $rty {
            #[inline]
            fn descr() -> &'static str {
                $npty
            }

            #[inline]
            fn write_to<W: Write>(self, w: &mut W) -> std::io::Result<usize> {
                w.write(&<$rty>::to_le_bytes(self))
            }
        }
    };
}

impl_dtype!(i8, "<i1");
impl_dtype!(u8, "<u1");
impl_dtype!(i16, "<i2");
impl_dtype!(u16, "<u2");
impl_dtype!(i32, "<i4");
impl_dtype!(u32, "<u4");
impl_dtype!(i64, "<i8");
impl_dtype!(u64, "<u8");
impl_dtype!(f32, "<f4");
impl_dtype!(f64, "<f8");

struct Header<'a, T> {
    shape: &'a [usize],
    phantom: PhantomData<T>,
}

fn allign_header_buffer(buf: &mut Vec<u8>) -> usize {
    let mut header_len = buf.len();

    let overflow = header_len % ARRAY_ALIGN;
    if overflow > 0 {
        header_len += ARRAY_ALIGN - overflow;
        buf.resize(header_len, b' ');
    }

    header_len
}

impl<T: DType> Header<'_, T> {
    fn put_to<W: Write>(&self, mut buf: W) -> crate::Result<()> {
        let mut header_buf = vec![b' '; ARRAY_ALIGN * 2];
        let mut cursor = Cursor::new(&mut header_buf);

        cursor.write_all(MAGIC_PREFIX)?;
        cursor.write_all(V30_BYTES)?;
        // Skip bytes for header length value
        cursor.set_position(V30_HEADER_BEGIN as u64);

        let pad_len = GROWTH_AXIS_MAX_DIGITS
            - self
                .shape
                .first()
                .unwrap_or(&0)
                .checked_ilog10()
                .unwrap_or(0) as usize
            + 1;

        write!(
            cursor,
            r#"{{'descr':'{descr}','fortran_order':False,'shape':({shape})}}{empty:pad_len$}"#,
            descr = T::descr(),
            shape = self
                .shape
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(","),
            empty = "",
        )?;

        let header_len = allign_header_buffer(cursor.get_mut());
        let header_len = V30HeaderLentype::try_from(header_len - V30_HEADER_BEGIN)
            .map_err(|err| crate::Error::HeaderLen(err, V30_BYTES))?
            .to_le_bytes();

        // Write header length value
        cursor.set_position(V30_PREFIX_LEN as u64);
        cursor.write_all(&header_len)?;

        *header_buf.last_mut().unwrap() = b'\n';

        buf.write_all(&header_buf)?;

        Ok(())
    }
}

pub struct NpyWriterBuilder<T, W> {
    inner: W,
    phantom: PhantomData<T>,
}

impl<T: DType, W: Write> NpyWriterBuilder<T, W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn begin(self, shape: &[usize]) -> crate::Result<NpyWriter<T, W>> {
        let Self { mut inner, phantom } = self;

        let header = Header::<T> { shape, phantom };
        header.put_to(&mut inner)?;

        Ok(NpyWriter { inner, phantom })
    }
}

pub struct NpyWriter<T, W> {
    inner: W,
    phantom: PhantomData<T>,
}

impl<T: DType, W: Write> NpyWriter<T, W> {
    pub fn build(inner: W) -> NpyWriterBuilder<T, W> {
        NpyWriterBuilder::new(inner)
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn put(&mut self, src: T) -> std::io::Result<usize> {
        src.write_to(&mut self.inner)
    }
}
