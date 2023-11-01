//! Adds functionality to `Cell` that does not depend on implementation details.

use super::{core::Builder, types::CellValue};
use std::{fmt, ops};

/// An allocated cell.
#[must_use]
pub struct Cell<'a, const N: usize, T: CellValue> {
    pub(super) builder: &'a Builder<N, T>,
    pub(super) location: usize,
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

    /// Swaps the values of two cells.
    pub fn swap(&mut self, other: &mut Cell<N, T>) {
        let temp = self.move_and_zero();
        other.move_into_and_zero(self);
        temp.move_into(other);
    }

    /// Turns this cell into several new cells that are copies of the original, and destroys the
    /// original. If you need to keep the original cell intact after copying, use `.copy()` instead.
    pub fn into_copies<const U: usize>(mut self) -> [Cell<'a, N, T>; U] {
        let mut cells = [(); U].map(|_| self.builder.cell(T::ZERO));

        self.while_nonzero_mut(|this| {
            this.dec();
            cells.iter_mut().for_each(|cell| cell.inc());
        });

        cells
    }

    /// Turns this cell into several new cells that are copies of the original. Prefer
    /// `.into_copies()` when possible, as it generates much shorter code by not needing a temporary
    /// cell.
    pub fn copy<const U: usize>(&self) -> [Cell<'a, N, T>; U] {
        let mut cells = [(); U].map(|_| self.builder.cell(T::ZERO));
        let mut temp = self.builder.cell(T::ZERO);

        // This is a handwritten implementation. We take a non-mutable reference, but `self` needs
        // to be mutated, even if it doesn't stay that way.

        self.goto();
        *self.builder.source.borrow_mut() += "[-";
        cells.iter_mut().for_each(|cell| cell.inc());
        temp.inc();
        self.goto();
        *self.builder.source.borrow_mut() += "]";
        temp.goto();
        *self.builder.source.borrow_mut() += "[-";
        self.goto();
        *self.builder.source.borrow_mut() += "+";
        temp.goto();
        *self.builder.source.borrow_mut() += "]";

        cells
    }

    /// Adds the contents of this cell into several others, zeroing this cell in the process.
    pub fn add_into_all_and_zero<const U: usize>(&mut self, mut others: [&mut Cell<N, T>; U]) {
        self.while_nonzero_mut(|this| {
            this.dec();
            others.iter_mut().for_each(|cell| cell.inc());
        });
    }

    /// Moves the contents of this cell into a new cell, zeroing this cell in the process. Returns
    /// the new cell.
    pub fn move_and_zero(&mut self) -> Cell<'a, N, T> {
        let mut output = self.builder().cell(T::ZERO);
        self.add_into_all_and_zero([&mut output]);
        output
    }

    /// Moves the contents of this cell into another cell, zeroing this cell afterwards.
    pub fn move_into_and_zero(&mut self, output: &mut Cell<N, T>) {
        output.zero();
        self.add_into_all_and_zero([output]);
    }

    /// Moves the contents of this cell into another cell, dropping this cell afterwards.
    pub fn move_into(mut self, output: &mut Cell<N, T>) {
        self.move_into_and_zero(output);
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

impl<'a, const N: usize, T: CellValue> Clone for Cell<'a, N, T> {
    fn clone(&self) -> Self {
        let [copy] = self.copy();
        copy
    }

    fn clone_from(&mut self, other: &Self) {
        let temp = self.builder.cell(T::ZERO);
        let source = &self.builder.source;

        self.zero();

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

impl<'a, const N: usize, T: CellValue> fmt::Debug for Cell<'a, N, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CellU8").field(&self.location).finish()
    }
}

impl<'a, const N: usize, T: CellValue> ops::DivAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
    fn div_assign(&mut self, rhs: &Cell<'a, N, T>) {
        let mut temp0 = self.builder.cell(T::ZERO);
        let mut temp1 = self.builder.cell(T::ZERO);
        let mut temp2 = self.builder.cell(T::ZERO);
        let mut temp3 = self.builder.cell(T::ZERO);
        self.move_into_and_zero(&mut temp0);

        temp0.while_nonzero_mut(|mut temp0| {
            // Implemented manually because we take a non-mutable reference to `rhs`.
            rhs.goto();
            *self.builder.source.borrow_mut() += "[-";
            temp1.inc();
            temp2.inc();
            rhs.goto();
            *self.builder.source.borrow_mut() += "]";

            temp2.while_nonzero_mut(|temp2| {
                temp2.dec();
                rhs.goto();
                *self.builder.source.borrow_mut() += "+";
            });

            temp1.while_nonzero_mut(|temp1| {
                temp2.inc();
                temp0.dec();

                temp0.while_nonzero_mut(|temp0| {
                    temp2.zero();
                    temp3.inc();
                    temp0.dec();
                });

                temp3.add_into_all_and_zero([&mut temp0]);

                temp2.while_nonzero_mut(|temp2| {
                    temp1.dec();

                    temp1.while_nonzero_mut(|temp1| {
                        self.dec();
                        temp1.zero();
                    });

                    temp1.inc();
                    temp2.dec();
                });

                temp1.dec();
            });

            self.inc();
        });
    }
}

impl<'a, const N: usize, T: CellValue> ops::AddAssign<T> for Cell<'a, N, T> {
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

impl<'a, const N: usize, T: CellValue> ops::AddAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
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

impl<'a, const N: usize, T: CellValue> ops::AddAssign<Cell<'a, N, T>> for Cell<'a, N, T> {
    fn add_assign(&mut self, mut rhs: Cell<'a, N, T>) {
        rhs.while_nonzero_mut(|rhs| {
            rhs.dec();
            self.inc();
        })
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(mut self, rhs: Cell<'a, N, T>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<&Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(mut self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(self, mut rhs: Cell<'a, N, T>) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<&Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self.clone() + rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<T> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(mut self, rhs: T) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Add<T> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn add(self, rhs: T) -> Self::Output {
        self.clone() + rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::SubAssign<T> for Cell<'a, N, T> {
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

impl<'a, const N: usize, T: CellValue> ops::SubAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
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

impl<'a, const N: usize, T: CellValue> ops::SubAssign<Cell<'a, N, T>> for Cell<'a, N, T> {
    fn sub_assign(&mut self, mut rhs: Cell<'a, N, T>) {
        rhs.while_nonzero_mut(|rhs| {
            rhs.dec();
            self.dec();
        })
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(mut self, rhs: Cell<'a, N, T>) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<&Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(mut self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(self, rhs: Cell<'a, N, T>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<&Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<T> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(mut self, rhs: T) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Sub<T> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn sub(self, rhs: T) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::MulAssign<&Cell<'a, N, T>> for Cell<'a, N, T> {
    fn mul_assign(&mut self, rhs: &Cell<'a, N, T>) {
        let mut x = self.move_and_zero();

        x.while_nonzero_mut(|x| {
            *self += rhs;
            x.dec();
        });
    }
}

impl<'a, const N: usize, T: CellValue> ops::MulAssign<Cell<'a, N, T>> for Cell<'a, N, T> {
    fn mul_assign(&mut self, rhs: Cell<'a, N, T>) {
        *self += &rhs;
    }
}

impl<'a, const N: usize, T: CellValue> ops::MulAssign<T> for Cell<'a, N, T> {
    fn mul_assign(&mut self, rhs: T) {
        *self *= &self.builder().cell(rhs);
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(mut self, rhs: Cell<'a, N, T>) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<&Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(mut self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(self, mut rhs: Cell<'a, N, T>) -> Self::Output {
        rhs *= self;
        rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<&Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self.clone() * rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<T> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(mut self, rhs: T) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Mul<T> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.clone() * rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::DivAssign<T> for Cell<'a, N, T> {
    fn div_assign(&mut self, rhs: T) {
        *self /= &self.builder().cell(rhs);
    }
}

impl<'a, const N: usize, T: CellValue> ops::DivAssign<Cell<'a, N, T>> for Cell<'a, N, T> {
    fn div_assign(&mut self, rhs: Cell<'a, N, T>) {
        *self /= &rhs;
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(mut self, rhs: Cell<'a, N, T>) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<&Cell<'a, N, T>> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(mut self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(self, rhs: Cell<'a, N, T>) -> Self::Output {
        self.clone() / rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<&Cell<'a, N, T>> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(self, rhs: &Cell<'a, N, T>) -> Self::Output {
        self.clone() / rhs
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<T> for Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(mut self, rhs: T) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<'a, const N: usize, T: CellValue> ops::Div<T> for &Cell<'a, N, T> {
    type Output = Cell<'a, N, T>;

    fn div(self, rhs: T) -> Self::Output {
        self.clone() / rhs
    }
}
