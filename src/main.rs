//! A builder, compiler, and runner for brainfuck programs written in pure Rust.

#![feature(array_windows)]
#![deny(unsafe_op_in_unsafe_fn, missing_docs, missing_debug_implementations)]

use crate::builder::core::Builder;
use std::{
    io::{stdin, stdout, Read},
    num::Wrapping,
};

pub mod builder;
pub mod program;
pub mod runner;

fn main() -> Result<(), &'static str> {
    let builder = Builder::<65536, Wrapping<u8>>::new();

    let mut value = builder.cell(Wrapping(0));

    let mut input = builder.read();
    input.while_nonzero_mut(|input| {
        value *= Wrapping(10);
        *input -= Wrapping(48);
        value += &*input;
        input.zero();
        input.read();
    });

    value.write();

    eprintln!("{builder:?}");

    builder.run(stdin().bytes().map(|x| Wrapping(x.unwrap())), stdout())?;

    Ok(())
}
