#![allow(overflowing_literals)]

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use npy_stream_writer::{DType, NpyWriterBuilder};

#[derive(Clone, clap::ValueEnum)]
enum TypeName {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

#[derive(clap::Parser)]
struct App {
    #[clap(short, long)]
    r#type: TypeName,

    #[clap(short, long)]
    output: PathBuf,
}

impl App {
    fn run(self) -> anyhow::Result<()> {
        let mut file = File::create(self.output).unwrap();
        let mut buf = BufWriter::new(&mut file);

        match self.r#type {
            TypeName::I8 => write_npy::<i8, _>(&mut buf),
            TypeName::U8 => write_npy::<u8, _>(&mut buf),
            TypeName::I16 => write_npy::<i16, _>(&mut buf),
            TypeName::U16 => write_npy::<u16, _>(&mut buf),
            TypeName::I32 => write_npy::<i32, _>(&mut buf),
            TypeName::U32 => write_npy::<u32, _>(&mut buf),
            TypeName::I64 => write_npy::<i64, _>(&mut buf),
            TypeName::U64 => write_npy::<u64, _>(&mut buf),
            TypeName::F32 => write_npy::<f32, _>(&mut buf),
            TypeName::F64 => write_npy::<f64, _>(&mut buf),
        }
    }
}

fn write_npy<T, W>(buf: &mut W) -> anyhow::Result<()>
where
    T: DType + TryFrom<u16> + std::ops::Add<Output = T>,
    <T as TryFrom<u16>>::Error: std::fmt::Debug,
    W: Write,
{
    let height: u16 = 10;
    let width: u16 = 10;

    let mut writer =
        NpyWriterBuilder::<T, _>::new(buf).begin(&[height as usize, width as usize])?;

    let row_range = (0..height * width).step_by(width as usize);

    for row in row_range {
        for col in 0..width {
            let src = T::try_from(row).unwrap() + T::try_from(col).unwrap();
            writer.put(src)?;
        }
    }

    Ok(())
}

fn main() {
    if let Err(err) = App::parse().run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
