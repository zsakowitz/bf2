//! Adds functionality to `CellU8` that does not depend on implementation details.

use super::core::CellU8;
use std::ops;

impl<'a, const N: usize> CellU8<'a, N> {
    /// Moves the contents of this cell into a new cell, zeroing this cell in the process. Returns
    /// the new cell.
    pub fn move_and_zero(&mut self) -> Self {
        let mut output = self.memory().u8(0);

        self.while_nonzero_mut(|this| {
            this.dec();
            output.inc();
        });

        output
    }

    /// Moves the contents of this cell into another cell, dropping this cell afterwards.
    pub fn move_into(mut self, output: &mut CellU8<N>) {
        self.while_nonzero_mut(|this| {
            this.dec();
            output.inc();
        });
    }
}

impl<'a, const N: usize> ops::AddAssign<CellU8<'a, N>> for CellU8<'a, N> {
    fn add_assign(&mut self, mut rhs: CellU8<'a, N>) {
        rhs.while_nonzero_mut(|rhs| {
            rhs.dec();
            self.inc();
        })
    }
}

impl<'a, const N: usize> ops::Add<CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(mut self, rhs: CellU8<'a, N>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize> ops::Add<&CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(mut self, rhs: &CellU8<'a, N>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize> ops::Add<CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(self, mut rhs: CellU8<'a, N>) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl<'a, const N: usize> ops::Add<&CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(self, rhs: &CellU8<'a, N>) -> Self::Output {
        self.clone() + rhs
    }
}

impl<'a, const N: usize> ops::Add<u8> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(mut self, rhs: u8) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, const N: usize> ops::Add<u8> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn add(self, rhs: u8) -> Self::Output {
        self.clone() + rhs
    }
}

impl<'a, const N: usize> ops::Add<CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn add(self, mut rhs: CellU8<'a, N>) -> Self::Output {
        rhs += self;
        rhs
    }
}

impl<'a, const N: usize> ops::Add<&CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn add(self, rhs: &CellU8<'a, N>) -> Self::Output {
        rhs.clone() + self
    }
}

impl<'a, const N: usize> ops::SubAssign<CellU8<'a, N>> for CellU8<'a, N> {
    fn sub_assign(&mut self, mut rhs: CellU8<'a, N>) {
        rhs.while_nonzero_mut(|rhs| {
            rhs.dec();
            self.dec();
        })
    }
}

impl<'a, const N: usize> ops::Sub<CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(mut self, rhs: CellU8<'a, N>) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Sub<&CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(mut self, rhs: &CellU8<'a, N>) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Sub<CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(self, rhs: CellU8<'a, N>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize> ops::Sub<&CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(self, rhs: &CellU8<'a, N>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize> ops::Sub<u8> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(mut self, rhs: u8) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Sub<u8> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn sub(self, rhs: u8) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'a, const N: usize> ops::Sub<CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn sub(self, rhs: CellU8<'a, N>) -> Self::Output {
        rhs.memory().u8(self) - rhs
    }
}

impl<'a, const N: usize> ops::Sub<&CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn sub(self, rhs: &CellU8<'a, N>) -> Self::Output {
        rhs.memory().u8(self) - rhs
    }
}

impl<'a, const N: usize> ops::MulAssign<&CellU8<'a, N>> for CellU8<'a, N> {
    fn mul_assign(&mut self, rhs: &CellU8<'a, N>) {
        let mut x = self.move_and_zero();

        x.while_nonzero_mut(|x| {
            *self += rhs;
            x.dec();
        });
    }
}

impl<'a, const N: usize> ops::MulAssign<CellU8<'a, N>> for CellU8<'a, N> {
    fn mul_assign(&mut self, rhs: CellU8<'a, N>) {
        *self += &rhs;
    }
}

impl<'a, const N: usize> ops::MulAssign<u8> for CellU8<'a, N> {
    fn mul_assign(&mut self, rhs: u8) {
        *self *= &self.memory().u8(rhs);
    }
}

impl<'a, const N: usize> ops::Mul<CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(mut self, rhs: CellU8<'a, N>) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Mul<&CellU8<'a, N>> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(mut self, rhs: &CellU8<'a, N>) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Mul<CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(self, mut rhs: CellU8<'a, N>) -> Self::Output {
        rhs *= self;
        rhs
    }
}

impl<'a, const N: usize> ops::Mul<&CellU8<'a, N>> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(self, rhs: &CellU8<'a, N>) -> Self::Output {
        self.clone() * rhs
    }
}

impl<'a, const N: usize> ops::Mul<u8> for CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(mut self, rhs: u8) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, const N: usize> ops::Mul<u8> for &CellU8<'a, N> {
    type Output = CellU8<'a, N>;

    fn mul(self, rhs: u8) -> Self::Output {
        self.clone() * rhs
    }
}

impl<'a, const N: usize> ops::Mul<CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn mul(self, mut rhs: CellU8<'a, N>) -> Self::Output {
        rhs *= self;
        rhs
    }
}

impl<'a, const N: usize> ops::Mul<&CellU8<'a, N>> for u8 {
    type Output = CellU8<'a, N>;

    fn mul(self, rhs: &CellU8<'a, N>) -> Self::Output {
        rhs.clone() * self
    }
}
