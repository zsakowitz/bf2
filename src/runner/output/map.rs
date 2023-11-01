//! Provides a struct implementing `RunnerOutput` that maps the values passed to `.write()`.

use super::RunnerOutput;
use std::marker::PhantomData;

/// A struct implementing `RunnerOutput` thats maps the values passed to `.write()`.
#[derive(Debug)]
pub struct Map<I, B: RunnerOutput<I>, O, T: FnMut(O) -> I> {
    pub(super) mapper: T,
    pub(super) _phantom: PhantomData<(I, O)>,
    pub(super) base: B,
}

impl<I, B: RunnerOutput<I>, O, T: FnMut(O) -> I> RunnerOutput<O> for Map<I, B, O, T> {
    fn write(&mut self, value: O) {
        self.base.write((self.mapper)(value))
    }
}
