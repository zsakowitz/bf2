//! The core implementation details of the brainfuck allocator.

use std::{
    cell::RefCell,
    fmt,
    ops::{AddAssign, SubAssign},
};

use crate::{program::Program, runner::Runner};

/// An allocating builder for brainfuck programs.
///
/// Debugging a builder shows its current source code, as well as a binary string showing its
/// current allocations, where 0s represent unallocated cells, 1s represent allocated cells, and the
/// currently pointed at cell is represented with an underline.
pub struct Builder<const N: usize> {
    source: RefCell<String>,
    pointer: RefCell<usize>,
    memory: RefCell<[bool; N]>,
    lowest_unallocated_value: RefCell<usize>,
}

impl<const N: usize> Builder<N> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Builder {
            source: RefCell::new(String::new()),
            pointer: RefCell::new(0),
            memory: RefCell::new([false; N]),
            lowest_unallocated_value: RefCell::new(0),
        }
    }

    /// Compiles this builder into a program.
    pub fn compile(&self) -> Result<Program, &'static str> {
        Program::new(self.source.borrow().as_str())
    }

    /// Compiles this builder and runs it on a given input string.
    pub fn run(&self, input: &[u8]) -> Result<Runner<N>, &'static str> {
        Ok(self.compile()?.run(input))
    }

    /// Creates an array of cells guaranteed to be consecutive in memory.
    ///
    /// ## Safety
    ///
    /// Make sure the cells are initialized before being passed to outside functions.
    pub unsafe fn u8_array_uninit<const U: usize>(&self) -> [CellU8<N>; U] {
        let location = *self.lowest_unallocated_value.borrow();
        let mut memory = self.memory.borrow_mut();

        let Some(chunk_start) = memory
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
            memory[index] = true;
        }

        for next_location in location.. {
            if !memory[next_location] {
                self.lowest_unallocated_value.replace(next_location);

                return (chunk_start..chunk_start + U)
                    .map(|location| CellU8 {
                        memory: self,
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
    pub fn u8_array<const U: usize>(&self, value: [u8; U]) -> [CellU8<N>; U] {
        let mut cells = unsafe { self.u8_array_uninit() };

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
    pub unsafe fn u8_uninit(&self) -> CellU8<N> {
        let location = *self.lowest_unallocated_value.borrow();
        let mut memory = self.memory.borrow_mut();
        memory[location] = true;

        for next_location in location + 1.. {
            if !memory[next_location] {
                self.lowest_unallocated_value.replace(next_location);

                return CellU8 {
                    memory: self,
                    location,
                };
            }
        }

        panic!("out of memory");
    }

    /// Creates a new cell with a specific value.
    pub fn u8(&self, value: u8) -> CellU8<N> {
        let mut cell = unsafe { self.u8_uninit() };
        cell.set(value);
        cell
    }

    /// Creates a new cell containing the next byte of input.
    pub fn read(&self) -> CellU8<N> {
        let mut cell = unsafe { self.u8_uninit() };
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

impl<const N: usize> fmt::Debug for Builder<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let memory = self.memory.borrow();

        let final_filled_index = memory
            .iter()
            .enumerate()
            .rev()
            .find(|(_, is_filled)| **is_filled)
            .map(|(index, _)| index)
            .unwrap_or(4)
            .max(4);

        let pointer = *self.pointer.borrow();

        let mut memory = memory[0..final_filled_index]
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

        memory += "...";

        f.debug_struct("Builder")
            .field("source", &self.source.borrow())
            .field("memory", &VerbatimDebug(memory))
            .finish()
    }
}

/// An allocated cell.
pub struct CellU8<'a, const N: usize> {
    memory: &'a Builder<N>,
    location: usize,
}

impl<'a, const N: usize> CellU8<'a, N> {
    /// Gets the underlying allocator this cell was created with.
    pub fn memory(&self) -> &'a Builder<N> {
        self.memory
    }

    /// Goes to this cell in memory.
    pub fn goto(&self) {
        let mut source = self.memory.source.borrow_mut();
        let mut pointer = self.memory.pointer.borrow_mut();

        if self.location < *pointer {
            for _ in 0..*pointer - self.location {
                source.push('<');
            }
        } else if self.location > *pointer {
            for _ in 0..self.location - *pointer {
                source.push('>');
            }
        }

        *pointer = self.location;
    }

    /// Increments this cell.
    pub fn inc(&mut self) {
        self.goto();
        self.memory.source.borrow_mut().push('+');
    }

    /// Decrements this cell.
    pub fn dec(&mut self) {
        self.goto();
        self.memory.source.borrow_mut().push('-');
    }

    /// Reads a character from input into this cell.
    pub fn read(&mut self) {
        self.goto();
        self.memory.source.borrow_mut().push(',');
    }

    /// Writes the character encoded in this cell into output.
    pub fn write(&self) {
        self.goto();
        self.memory.source.borrow_mut().push('.');
    }

    /// Runs code while the value of this cell is nonzero.
    pub fn while_nonzero(&self, f: impl FnOnce()) {
        {
            self.goto();
            *self.memory.source.borrow_mut() += "[";
        }

        f();

        {
            self.goto();
            *self.memory.source.borrow_mut() += "]";
        }
    }

    /// Runs code while the value of this cell is nonzero, and provides mutable access to this cell
    /// in the process.
    pub fn while_nonzero_mut(&mut self, f: impl FnOnce(&mut Self)) {
        {
            self.goto();
            *self.memory.source.borrow_mut() += "[";
        }

        f(self);

        {
            self.goto();
            *self.memory.source.borrow_mut() += "]";
        }
    }

    /// Sets the value of this cell to zero.
    pub fn zero(&mut self) {
        self.goto();
        *self.memory.source.borrow_mut() += "[-]";
    }

    /// Sets the value of this cell to a given value.
    pub fn set(&mut self, value: u8) {
        self.zero();
        *self += value;
    }
}

impl<'a, const N: usize> Drop for CellU8<'a, N> {
    fn drop(&mut self) {
        self.zero();

        let mut memory = self.memory.memory.borrow_mut();
        memory[self.location] = false;

        self.memory
            .lowest_unallocated_value
            .replace_with(|value| (*value).min(self.location));
    }
}

impl<'a, const N: usize> fmt::Debug for CellU8<'a, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CellU8").field(&self.location).finish()
    }
}

// This is only implemented in `core` for performance reasons.
impl<'a, const N: usize> AddAssign<u8> for CellU8<'a, N> {
    fn add_assign(&mut self, rhs: u8) {
        self.goto();

        let mut source = self.memory.source.borrow_mut();

        if rhs > 128 {
            for _ in rhs..=255 {
                source.push('-');
            }
        } else {
            for _ in 0..rhs {
                source.push('+');
            }
        }
    }
}

// This is only implemented in `core` for performance reasons.
impl<'a, const N: usize> SubAssign<u8> for CellU8<'a, N> {
    fn sub_assign(&mut self, rhs: u8) {
        self.goto();

        let mut source = self.memory.source.borrow_mut();

        if rhs > 128 {
            for _ in rhs..=255 {
                source.push('+');
            }
        } else {
            for _ in 0..rhs {
                source.push('-');
            }
        }
    }
}

// This is implemented in `core` to get around the fact that `Clone::clone()` takes an immutable
// reference, but the underlying data is actually mutated during the process.
impl<'a, const N: usize> Clone for CellU8<'a, N> {
    fn clone(&self) -> Self {
        let temp = self.memory.u8(0);
        let output = self.memory.u8(0);
        let source = &self.memory.source;

        self.goto();
        *source.borrow_mut() += "[";
        temp.goto();
        *source.borrow_mut() += "+";
        self.goto();
        *source.borrow_mut() += "-]";

        temp.goto();
        *source.borrow_mut() += "[";
        self.goto();
        *source.borrow_mut() += "+";
        output.goto();
        *source.borrow_mut() += "+";
        temp.goto();
        *source.borrow_mut() += "-]";

        output
    }

    fn clone_from(&mut self, other: &Self) {
        let temp = self.memory.u8(0);
        let source = &self.memory.source;

        other.goto();
        *source.borrow_mut() += "[";
        temp.goto();
        *source.borrow_mut() += "+";
        other.goto();
        *source.borrow_mut() += "-]";

        temp.goto();
        *source.borrow_mut() += "[";
        self.goto();
        *source.borrow_mut() += "+";
        other.goto();
        *source.borrow_mut() += "+";
        temp.goto();
        *source.borrow_mut() += "-]";
    }
}

// This is implemented in `core` to avoid unnecessary clones.
impl<'a, const N: usize> AddAssign<&CellU8<'a, N>> for CellU8<'a, N> {
    fn add_assign(&mut self, rhs: &CellU8<'a, N>) {
        let temp = rhs.memory.u8(0);
        let source = &rhs.memory.source;

        rhs.goto();
        *source.borrow_mut() += "[";
        temp.goto();
        *source.borrow_mut() += "+";
        rhs.goto();
        *source.borrow_mut() += "-]";

        temp.goto();
        *source.borrow_mut() += "[";
        rhs.goto();
        *source.borrow_mut() += "+";
        self.goto();
        *source.borrow_mut() += "+";
        temp.goto();
        *source.borrow_mut() += "-]";
    }
}

// This is implemented in `core` to avoid unnecessary clones.
impl<'a, const N: usize> SubAssign<&CellU8<'a, N>> for CellU8<'a, N> {
    fn sub_assign(&mut self, rhs: &CellU8<'a, N>) {
        let temp = rhs.memory.u8(0);
        let source = &rhs.memory.source;

        rhs.goto();
        *source.borrow_mut() += "[";
        temp.goto();
        *source.borrow_mut() += "+";
        rhs.goto();
        *source.borrow_mut() += "-]";

        temp.goto();
        *source.borrow_mut() += "[";
        rhs.goto();
        *source.borrow_mut() += "+";
        self.goto();
        *source.borrow_mut() += "-";
        temp.goto();
        *source.borrow_mut() += "-]";
    }
}
