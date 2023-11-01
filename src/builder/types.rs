//! Defines traits that values can implement to allow storing them in brainfuck cells.

use std::num::Wrapping;

/// A value that may be stored inside a brainfuck memory cell.
pub trait CellValue: PartialEq + Copy {
    /// The zero value of this type.
    const ZERO: Self;

    /// Increments this value by one. Wrapping is undefined behavior unless in a `Wrapping<T>`.
    fn inc(self) -> Self;

    /// Decrements this value by one. Wrapping is undefined behavior unless in a `Wrapping<T>`.
    fn dec(self) -> Self;

    /// Converts this value into an isize.
    ///
    /// ## Panics
    ///
    /// Panics if the value of this cell is not possible to fit in an `isize`.
    fn into_isize(self) -> isize;
}

/// A value that may be debugged in a brainfuck `Runner`'s input or output.
pub trait DebuggableCellValue: CellValue {
    /// Converts this cell value into a valid Unicode character.
    fn into_char(self) -> char;
}

macro_rules! direct_cell_value_impl {
    ($($x:ty)+) => {
        $(
            impl CellValue for $x {
                const ZERO: Self = 0;
                fn inc(self) -> Self { self + 1 }
                fn dec(self) -> Self { self - 1 }
                fn into_isize(self) -> isize { self.try_into().unwrap() }
            }

            impl DebuggableCellValue for $x {
                fn into_char(self) -> char { self.into() }
            }

            impl CellValue for Wrapping<$x> {
                const ZERO: Self = Wrapping(0);
                fn inc(self) -> Self { self + Wrapping(1) }
                fn dec(self) -> Self { self - Wrapping(1) }
                fn into_isize(self) -> isize { self.0.try_into().unwrap() }
            }

            impl DebuggableCellValue for Wrapping<$x> {
                fn into_char(self) -> char { self.0.into() }
            }
        )+
    };
}

macro_rules! cell_value_impl {
    ($($x:ty)+) => {
        $(
            impl CellValue for $x {
                const ZERO: Self = 0;
                fn inc(self) -> Self { self + 1 }
                fn dec(self) -> Self { self - 1 }
                fn into_isize(self) -> isize { self.try_into().unwrap() }
            }

            impl DebuggableCellValue for $x {
                fn into_char(self) -> char { u32::try_from(self).unwrap().try_into().unwrap() }
            }

            impl CellValue for Wrapping<$x> {
                const ZERO: Self = Wrapping(0);
                fn inc(self) -> Self { self + Wrapping(1) }
                fn dec(self) -> Self { self - Wrapping(1) }
                fn into_isize(self) -> isize { self.0.try_into().unwrap() }
            }

            impl DebuggableCellValue for Wrapping<$x> {
                fn into_char(self) -> char { u32::try_from(self.0).unwrap().try_into().unwrap() }
            }
        )+
    };
}

direct_cell_value_impl! { u8 }
cell_value_impl! { i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize }
