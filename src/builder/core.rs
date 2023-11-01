//! The core implementation details of the brainfuck allocator.

use super::{cell::Cell, string::CellString, types::CellValue};
use crate::{
    program::Program,
    runner::{output::RunnerOutput, Runner},
};
use std::{
    cell::RefCell,
    fmt,
    io::{stdin, stdout, Read, Stdout},
    marker::PhantomData,
    ops,
};

/// An allocating builder for brainfuck programs.
///
/// Debugging a builder shows its current source code, as well as a binary string showing its
/// current allocations, where T::ZEROs represent unallocated cells, 1s represent allocated cells, and the
/// currently pointed at cell is represented with an underline.
pub struct Builder<const N: usize, T: CellValue> {
    pub(super) source: RefCell<String>,
    pub(super) pointer: RefCell<usize>,
    pub(super) allocations: RefCell<[bool; N]>,
    pub(super) lowest_unallocated_value: RefCell<usize>,
    _phantom: PhantomData<T>,
}

impl<const N: usize, T: CellValue> Builder<N, T> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Builder {
            source: RefCell::new(String::new()),
            pointer: RefCell::new(0),
            allocations: RefCell::new([false; N]),
            lowest_unallocated_value: RefCell::new(0),
            _phantom: PhantomData,
        }
    }

    /// Compiles this builder into a program.
    pub fn compile(&self) -> Result<Program, &'static str> {
        Program::new(self.source.borrow().as_str())
    }

    /// Compiles this builder and runs it on a given input.
    pub fn run<I: Iterator<Item = T>, O: RunnerOutput<T>>(
        &self,
        input: I,
        output: O,
    ) -> Result<Runner<N, I, O, T>, &'static str> {
        Ok(self.compile()?.run(input, output))
    }

    /// Compiles this builder and runs it, using stdin and stdout as input and output respectively.
    pub fn run_interactive<I: FnMut(u8) -> T, O: FnMut(T) -> u8>(
        &self,
        input_adapter: I,
        output_adapter: O,
    ) -> Result<Runner<N, impl Iterator<Item = T>, impl RunnerOutput<T>, T>, &'static str> {
        struct Output<T, O: FnMut(T) -> u8> {
            stdout: Stdout,
            adapter: O,
            _phantom: PhantomData<T>,
        }

        impl<T, O: FnMut(T) -> u8> RunnerOutput<T> for Output<T, O> {
            fn write(&mut self, value: T) {
                self.stdout.write((self.adapter)(value));
            }
        }

        Ok(self.compile()?.run(
            stdin().bytes().map(|x| x.unwrap()).map(input_adapter),
            Output {
                stdout: stdout(),
                adapter: output_adapter,
                _phantom: PhantomData,
            },
        ))
    }

    /// Creates an array of cells guaranteed to be consecutive in memory.
    ///
    /// ## Safety
    ///
    /// Make sure the cells are initialized before being passed to outside functions.
    pub unsafe fn array_uninit<const U: usize>(&self) -> [Cell<N, T>; U] {
        let location = *self.lowest_unallocated_value.borrow();
        let mut allocations = self.allocations.borrow_mut();

        let Some(chunk_start) = allocations
            .array_windows::<U>()
            .enumerate()
            .skip(location)
            .find(|(_, chunk)| chunk.iter().all(|value| !*value))
            .map(|x| x.0)
        else {
            if U == 1 {
                panic!("not enough memory to allocate 1 cell")
            } else {
                panic!(
                    "{}",
                    format!("not enough memory to allocate {U} consecutive cells")
                );
            }
        };

        for index in chunk_start..chunk_start + U {
            allocations[index] = true;
        }

        for next_location in location.. {
            if !allocations[next_location] {
                self.lowest_unallocated_value.replace(next_location);

                return (chunk_start..chunk_start + U)
                    .map(|location| Cell {
                        builder: self,
                        location,
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
            }
        }

        panic!("out of memory");
    }

    /// Creates an array of initialized cells guaranteed to be consecutive in memory.
    pub fn array<const U: usize>(&self, value: [T; U]) -> [Cell<N, T>; U] {
        let mut cells = unsafe { self.array_uninit() };

        for index in 0..U {
            cells[index].set(value[index]);
        }

        cells
    }

    /// Creates a new uninitialized cell.
    ///
    /// ## Safety
    ///
    /// Make sure the cell is initialized before being passed to outside functions.
    pub unsafe fn cell_uninit(&self) -> Cell<N, T> {
        let location = *self.lowest_unallocated_value.borrow();
        let mut allocations = self.allocations.borrow_mut();
        allocations[location] = true;

        for next_location in location + 1.. {
            if !allocations[next_location] {
                self.lowest_unallocated_value.replace(next_location);

                return Cell {
                    builder: self,
                    location,
                };
            }
        }

        panic!("out of memory");
    }

    /// Creates a new cell with a specific value.
    pub fn cell(&self, value: T) -> Cell<N, T> {
        let mut cell = unsafe { self.cell_uninit() };
        cell.set(value);
        cell
    }

    /// Creates a new `CellString` with a specific value.
    pub fn str<'a, 'b>(&'a self, source: &'b str) -> CellString<'a, 'b, N, T> {
        CellString {
            builder: self,
            source,
        }
    }

    /// Creates a new `CellString` with a specific value and writes it.
    pub fn write<'a>(&'a self, source: &str)
    where
        T: PartialOrd + ops::Sub,
        u8: Into<T>,
        Cell<'a, N, T>: ops::AddAssign<<T as ops::Sub>::Output>,
        Cell<'a, N, T>: ops::SubAssign<<T as ops::Sub>::Output>,
    {
        self.str(source).write();
    }

    /// Creates a new cell containing the next byte of input, or `T::ZERO` if there is no input
    /// left.
    pub fn read(&self) -> Cell<N, T> {
        let mut cell = self.cell(T::ZERO);
        cell.read();
        cell
    }

    /// Creates a new cell containing the next byte of input, or `default` if there is no input
    /// left.
    pub fn read_or(&self, default: T) -> Cell<N, T> {
        let mut cell = self.cell(default);
        cell.read();
        cell
    }
}

struct VerbatimDebug(String);

impl fmt::Debug for VerbatimDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<const N: usize, T: CellValue> fmt::Debug for Builder<N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let allocations = self.allocations.borrow();

        let final_filled_index = allocations
            .iter()
            .enumerate()
            .rev()
            .find(|(_, is_filled)| **is_filled)
            .map(|(index, _)| index)
            .unwrap_or(4)
            .max(4);

        let pointer = *self.pointer.borrow();

        let mut allocations = allocations[0..final_filled_index]
            .iter()
            .enumerate()
            .map(|(index, is_filled)| {
                if index == pointer {
                    if *is_filled {
                        "1̲"
                    } else {
                        "0̲"
                    }
                } else {
                    if *is_filled {
                        "1"
                    } else {
                        "0"
                    }
                }
            })
            .collect();

        allocations += "..";

        f.debug_struct("Builder")
            .field("source", &self.source.borrow())
            .field("allocations", &VerbatimDebug(allocations))
            .finish()
    }
}
