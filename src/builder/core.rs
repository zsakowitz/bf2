//! The core implementation details of the brainfuck allocator.

use super::{string::CellString, types::CellValue};
use crate::{program::Program, runner::Runner};
use std::{
    cell::RefCell,
    fmt,
    marker::PhantomData,
    ops::{AddAssign, SubAssign},
};

/// An allocating builder for brainfuck programs.
///
/// Debugging a builder shows its current source code, as well as a binary string showing its
/// current allocations, where T::ZEROs represent unallocated cells, 1s represent allocated cells, and the
/// currently pointed at cell is represented with an underline.
pub struct Builder<const N: usize, T: CellValue> {
    source: RefCell<String>,
    pointer: RefCell<usize>,
    allocations: RefCell<[bool; N]>,
    lowest_unallocated_value: RefCell<usize>,
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

    /// Compiles this builder and runs it on a given input string.
    pub fn run(&self, input: &[T]) -> Result<Runner<N, T>, &'static str> {
        Ok(self.compile()?.run(input))
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

    /// Creates a new cell containing the next byte of input.
    pub fn read(&self) -> Cell<N, T> {
        let mut cell = unsafe { self.cell_uninit() };
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

        allocations += "...";

        f.debug_struct("Builder")
            .field("source", &self.source.borrow())
            .field("allocations", &VerbatimDebug(allocations))
            .finish()
    }
}

/// An allocated cell.
pub struct Cell<'a, const N: usize, T: CellValue> {
    builder: &'a Builder<N, T>,
    location: usize,
}

impl<'a, const N: usize, T: CellValue> Cell<'a, N, T> {
    /// Gets the underlying allocator this cell was created with.
    pub fn builder(&self) -> &'a Builder<N, T> {
        self.builder
    }

    /// Goes to this cell in memory.
    pub fn goto(&self) {
        let mut source = self.builder.source.borrow_mut();
        let mut pointer = self.builder.pointer.borrow_mut();

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
        self.builder.source.borrow_mut().push('+');
    }

    /// Decrements this cell.
    pub fn dec(&mut self) {
        self.goto();
        self.builder.source.borrow_mut().push('-');
    }

    /// Reads a character from input into this cell.
    pub fn read(&mut self) {
        self.goto();
        self.builder.source.borrow_mut().push(',');
    }

    /// Writes the character encoded in this cell into output.
    pub fn write(&self) {
        self.goto();
        self.builder.source.borrow_mut().push('.');
    }

    /// Runs code while the value of this cell is nonzero.
    pub fn while_nonzero(&self, f: impl FnOnce()) {
        {
            self.goto();
            *self.builder.source.borrow_mut() += "[";
        }

        f();

        {
            self.goto();
            *self.builder.source.borrow_mut() += "]";
        }
    }

    /// Runs code while the value of this cell is nonzero, and provides mutable access to this cell
    /// in the process.
    pub fn while_nonzero_mut(&mut self, f: impl FnOnce(&mut Self)) {
        {
            self.goto();
            *self.builder.source.borrow_mut() += "[";
        }

        f(self);

        {
            self.goto();
            *self.builder.source.borrow_mut() += "]";
        }
    }

    /// Sets the value of this cell to zero.
    pub fn zero(&mut self) {
        self.goto();
        *self.builder.source.borrow_mut() += "[-]";
    }

    /// Sets the value of this cell to a given value.
    pub fn set(&mut self, value: T) {
        self.zero();
        *self += value;
    }
}

impl<'a, const N: usize, T: CellValue> Drop for Cell<'a, N, T> {
    fn drop(&mut self) {
        self.zero();

        let mut allocations = self.builder.allocations.borrow_mut();
        allocations[self.location] = false;

        self.builder
            .lowest_unallocated_value
            .replace_with(|value| (*value).min(self.location));
    }
}

impl<'a, const N: usize, T: CellValue> fmt::Debug for Cell<'a, N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CellU8").field(&self.location).finish()
    }
}

// This is only implemented in `core` for performance reasons.
impl<'a, const N: usize, T: CellValue> AddAssign<T> for Cell<'a, N, T> {
    fn add_assign(&mut self, rhs: T) {
        self.goto();

        let mut source = self.builder.source.borrow_mut();

        let size = rhs.into_isize();

        if size < 0 {
            for _ in size..0 {
                source.push('-');
            }
        } else {
            for _ in 0..size {
                source.push('+');
            }
        }
    }
}

// This is only implemented in `core` for performance reasons.
impl<'a, const N: usize, T: CellValue> SubAssign<T> for Cell<'a, N, T> {
    fn sub_assign(&mut self, rhs: T) {
        self.goto();

        let mut source = self.builder.source.borrow_mut();

        let size = rhs.into_isize();

        if size < 0 {
            for _ in size..0 {
                source.push('+');
            }
        } else {
            for _ in 0..size {
                source.push('-');
            }
        }
    }
}

// This is implemented in `core` to get around the fact that `Clone::clone()` takes an immutable
// reference, but the underlying data is actually mutated during the process.
impl<'a, const N: usize, T: CellValue> Clone for Cell<'a, N, T> {
    fn clone(&self) -> Self {
        let temp = self.builder.cell(T::ZERO);
        let output = self.builder.cell(T::ZERO);
        let source = &self.builder.source;

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
        let temp = self.builder.cell(T::ZERO);
        let source = &self.builder.source;

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
impl<'a, const N: usize, T: CellValue> AddAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
    fn add_assign(&mut self, rhs: &Cell<'a, N, T>) {
        let temp = rhs.builder.cell(T::ZERO);
        let source = &rhs.builder.source;

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
impl<'a, const N: usize, T: CellValue> SubAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
    fn sub_assign(&mut self, rhs: &Cell<'a, N, T>) {
        let temp = rhs.builder.cell(T::ZERO);
        let source = &rhs.builder.source;

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
