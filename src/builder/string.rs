//! Defines a string which can be easily maniuplated for use in brainfuck programs.

use super::{
    core::{Builder, Cell},
    types::CellValue,
};
use std::{
    fmt,
    ops::{AddAssign, Sub, SubAssign},
};

#[must_use]
/// A cell containing a string. The string is not stored in the memory of the brainfuck program
/// until it is instantiated. Until then, it may be manipulated by other methods.
pub struct CellString<'a, 'b, const N: usize, T: CellValue> {
    pub(super) builder: &'a Builder<N, T>,
    pub(super) source: &'b str,
}

impl<'a, 'b, const N: usize, T: CellValue> fmt::Debug for CellString<'a, 'b, N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CellString")
            .field("source", &self.source)
            .finish()
    }
}

impl<'a, 'b, const N: usize, T: CellValue> CellString<'a, 'b, N, T> {
    /// Writes the entire contents of this string, using only two cells in the process.
    pub fn write(&mut self)
    where
        T: PartialOrd + Sub,
        u8: Into<T>,
        Cell<'a, N, T>: AddAssign<<T as Sub>::Output>,
        Cell<'a, N, T>: SubAssign<<T as Sub>::Output>,
    {
        let mut cell = self.builder.cell(T::ZERO);
        let mut cell_value = T::ZERO;

        for char in self.source.bytes() {
            let char: T = char.into();

            if char < cell_value {
                cell -= cell_value - char;
            } else if char > cell_value {
                cell += char - cell_value;
            }

            cell.write();
            cell_value = char;
        }
    }
}
