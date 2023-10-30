//! A builder, compiler, and runner for brainfuck programs written in pure Rust.

#![feature(array_windows)]
#![deny(unsafe_op_in_unsafe_fn, missing_docs, missing_debug_implementations)]

use crate::builder::core::Builder;

pub mod builder;
pub mod program;
pub mod runner;

fn main() -> Result<(), &'static str> {
    let builder = Builder::<65536>::new();

    let runner = builder.run(b"")?;

    println!("{builder:#?}");
    println!("{runner:#?}");

    Ok(())
}
