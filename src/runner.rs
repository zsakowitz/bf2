//! Defines a runner for brainfuck programs.

use crate::builder::types::CellValue;
use std::fmt::{self, Write};

/// A structure which can quickly run brainfuck programs.
///
/// Debugging a runner shows a portion of its internal memory tape (called `data`), with the
/// currently pointed at cell highlighted with arrow brackets, as well as its `input` and `output`.
/// A runner may also be printed as LowerHex or UpperHex, in which case everything will appear
/// identical to its Debug representation, but with hex strings instead of base-10 numerals for its
/// data.
///
/// The `N` const parameter is the size of the memory array, and the `T` type generic is the type of
/// value stored inside. All integer values may be used, along with their `Wrapping` and
/// `Saturating` variants.
pub struct Runner<const N: usize, T: CellValue> {
    memory: [T; N],
    pointer: usize,
    input: Vec<T>,
    output: Vec<T>,
}

impl<const N: usize, T: CellValue> Runner<N, T> {
    /// Constructs a new runner given some input.
    pub fn new(input: &[T]) -> Self {
        if N == 0 {
            panic!("cannot create a runner of size zero");
        }

        let mut input = input.to_vec();
        input.reverse();

        Self {
            memory: [T::ZERO; N],
            pointer: 0,
            input,
            output: Vec::new(),
        }
    }

    #[inline]
    /// Increments the currently pointed at cell.
    pub fn inc(&mut self) {
        self.memory[self.pointer] = self.memory[self.pointer].inc();
    }

    #[inline]
    /// Decrements the currently pointed at cell.
    pub fn dec(&mut self) {
        self.memory[self.pointer] = self.memory[self.pointer].dec();
    }

    #[inline]
    /// Moves the pointer to the left.
    pub fn shl(&mut self) {
        self.pointer -= 1;
    }

    #[inline]
    /// Moves the pointer to the right.
    pub fn shr(&mut self) {
        self.pointer += 1;
    }

    #[inline]
    /// Reads a value from `self.input` into the current cell.
    pub fn read(&mut self) {
        if let Some(input) = self.input.pop() {
            self.memory[self.pointer] = input;
        }
    }

    #[inline]
    /// Writes the current cell into `self.output`.
    pub fn write(&mut self) {
        self.output.push(self.memory[self.pointer]);
    }

    #[inline]
    /// Repeats code while the currently pointed at cell is nonzero.
    pub fn repeat(&mut self, mut f: impl FnMut(&mut Self)) {
        while self.memory[self.pointer] != T::ZERO {
            f(self);
        }
    }
}

struct RunnerData<'a, T: CellValue> {
    data: &'a [T],
    pointer: Option<usize>,
    includes_start: bool,
    includes_end: bool,
}

struct VerbatimDebug(String);

impl fmt::Debug for VerbatimDebug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T: CellValue + fmt::Debug> fmt::Debug for RunnerData<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.includes_start {
            f.write_str("... ")?;
        }

        if self.data.len() == 0 {
            f.write_str("(empty)")?;
        }

        for (index, value) in self.data.iter().enumerate() {
            if index != 0 {
                f.write_char(' ')?;
            }

            if Some(index) == self.pointer {
                f.write_char('<')?;
            }

            value.fmt(f)?;

            if Some(index) == self.pointer {
                f.write_char('>')?;
            }
        }

        if !self.includes_end {
            f.write_str(" ...")?;
        }

        Ok(())
    }
}

/// The number of values to show of memory away from where the pointer is located.
const DEBUG_DATA_WIDTH: i32 = 8;

impl<const N: usize, T: CellValue + Into<char> + fmt::Debug> fmt::Debug for Runner<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = {
            let start = 0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize;
            let end = (N as i32).min(self.pointer as i32 + DEBUG_DATA_WIDTH) as usize;
            let pointer = self.pointer - 0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize;

            RunnerData {
                data: &self.memory[start..end],
                pointer: Some(pointer),
                includes_start: start == 0,
                includes_end: end == N,
            }
        };

        if f.sign_minus() {
            let input = RunnerData {
                data: &self.input.iter().rev().copied().collect::<Vec<_>>(),
                pointer: None,
                includes_start: true,
                includes_end: true,
            };

            let output = RunnerData {
                data: &self.output.iter().copied().collect::<Vec<_>>(),
                pointer: None,
                includes_start: true,
                includes_end: true,
            };

            f.debug_struct("Runner")
                .field("data", &data)
                .field("input", &input)
                .field("output", &output)
                .finish()
        } else {
            let input = self
                .input
                .iter()
                .rev()
                .map(|x| (*x).into())
                .collect::<String>();

            let output = self.output.iter().map(|x| (*x).into()).collect::<String>();

            f.debug_struct("Runner")
                .field("data", &data)
                .field("input", &input)
                .field("output", &output)
                .finish()
        }
    }
}
