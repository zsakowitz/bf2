//! A builder, compiler, and runner for brainfuck programs written in pure Rust.

#![feature(array_windows)]
#![deny(unsafe_op_in_unsafe_fn, missing_docs, missing_debug_implementations)]

use std::fmt::format;

use crate::builder::core::Builder;

pub mod builder;
pub mod program;
pub mod runner;

fn main() -> Result<(), &'static str> {
    let builder = Builder::<65536, u8>::new();

    let a = builder.cell(23);
    let b = builder.cell(45);

    builder.str("Hello, world!").write();

    let runner = builder.run(b"Hello world")?;

    println!("{builder:#?}");
    println!("{runner:0>#2?}");

    Ok(())
}
