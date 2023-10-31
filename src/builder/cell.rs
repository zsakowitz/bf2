//! Adds functionality to `Cell` that does not depend on implementation details.

use super::{core::Cell, types::CellValue};
use std::ops;

impl<'a, const N: usize, T: CellValue> Cell<'a, N, T> {
    /// Moves the contents of this cell into a new cell, zeroing this cell in the process. Returns
    /// the new cell.
    pub fn move_and_zero(&mut self) -> Self {
        let mut output = self.builder().cell(T::ZERO);

        self.while_nonzero_mut(|this| {
            this.dec();
            output.inc();
        });

        output
    }

    /// Moves the contents of this cell into another cell, dropping this cell afterwards.
    pub fn move_into(mut self, output: &mut Cell<N, T>) {
        self.while_nonzero_mut(|this| {
            this.dec();
            output.inc();
        });
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
