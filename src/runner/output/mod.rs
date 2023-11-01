//! Provides a trait that can be implemented to take output from a runner.

pub mod map;

use std::io::{Stdout, Write};
use std::marker::PhantomData;
use std::num::Wrapping;

use self::map::Map;

/// Something which can provide input to a runner.
pub trait RunnerOutput<T> {
    /// Writes a value to output.
    fn write(&mut self, value: T);

    /// Maps the values passed to `.write()` through a function.
    fn map<I, F: FnMut(I) -> T>(self, f: F) -> Map<T, Self, I, F>
    where
        Self: Sized,
    {
        Map {
            base: self,
            mapper: f,
            _phantom: PhantomData,
        }
    }
}

impl<T> RunnerOutput<T> for Vec<T> {
    /// Writes a value to this `Vec`, by pushing it onto the end.
    fn write(&mut self, value: T) {
        self.push(value)
    }
}

macro_rules! direct_runner_output_impl {
    ($($x:ty)+) => {
        $(
            impl RunnerOutput<$x> for Stdout {
                fn write(&mut self, value: $x) {
                    self.write_all(&[value][..]).unwrap();
                }
            }


            impl RunnerOutput<Wrapping<$x>> for Stdout {
                fn write(&mut self, value: Wrapping<$x>) {
                    self.write_all(&[value.0][..]).unwrap();
                }
            }
        )+
    }
}

macro_rules! runner_output_impl {
    ($($x:ty)+) => {
        $(
            impl RunnerOutput<$x> for Stdout {
                fn write(&mut self, value: $x) {
                    self
                        .write_all(char::from_u32(value as u32)
                            .unwrap()
                            .encode_utf8(&mut [0; 4])
                            .as_bytes())
                        .unwrap();

                    self.flush().unwrap();
                }
            }


            impl RunnerOutput<Wrapping<$x>> for Stdout {
                fn write(&mut self, value: Wrapping<$x>) {
                    self
                        .write_all(char::from_u32(value.0 as u32)
                            .unwrap()
                            .encode_utf8(&mut [0; 4])
                            .as_bytes())
                        .unwrap();

                    self.flush().unwrap();
                }
            }
        )+
    }
}

direct_runner_output_impl! { u8 }
runner_output_impl! { u16 u32 u64 u128 }

/// A runner input which can be debugged.
pub trait DebuggableRunnerOutput<T> {
    /// Gets the next value of input.
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T> DebuggableRunnerOutput<T> for Stdout {
    fn debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(stdout)")
    }
}
