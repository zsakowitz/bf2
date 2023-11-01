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

macro_rules! cell_value_impl_u {
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

macro_rules! cell_value_impl_i {
    ($($x:ty)+) => {
        $(
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

cell_value_impl_u! { u8 u16 u32 u64 u128 }

// Note that regular signed integers are intentionally unimplemented, because it would complicate
// many of the existing algorithms. Only `Wrapping` variants of them are implemented.

cell_value_impl_i! { i8 i16 i32 i64 i128 }
