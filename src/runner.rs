//! Defines a runner for brainfuck programs.

use std::fmt;

/// A structure which can quickly run brainfuck programs.
///
/// Debugging a runner shows a portion of its internal memory tape (called `data`), with the
/// currently pointed at cell highlighted with arrow brackets, as well as its `input` and `output`.
/// A runner may also be printed as LowerHex or UpperHex, in which case everything will appear
/// identical to its Debug representation, but with hex strings instead of base-10 numerals for its
/// data.
pub struct Runner<const N: usize> {
    memory: [u8; N],
    pointer: usize,
    input: Vec<u8>,
    output: Vec<u8>,
}

impl<const N: usize> Runner<N> {
    /// Constructs a new runner given some input.
    pub fn new(input: &[u8]) -> Self {
        if N == 0 {
            panic!("cannot create a runner of size zero");
        }

        let mut input = input.to_vec();
        input.reverse();

        Self {
            memory: [0; N],
            pointer: 0,
            input,
            output: Vec::new(),
        }
    }

    #[inline]
    /// Increments the currently pointed at cell.
    pub fn inc(&mut self) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1);
    }

    #[inline]
    /// Decrements the currently pointed at cell.
    pub fn dec(&mut self) {
        self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1);
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
        self.memory[self.pointer] = self.input.pop().unwrap_or(0);
    }

    #[inline]
    /// Writes the current cell into `self.output`.
    pub fn write(&mut self) {
        self.output.push(self.memory[self.pointer]);
    }

    #[inline]
    /// Repeats code while the currently pointed at cell is nonzero.
    pub fn repeat(&mut self, mut f: impl FnMut(&mut Self)) {
        while self.memory[self.pointer] != 0 {
            f(self);
        }
    }
}

struct RunnerData<'a>(&'a [u8], usize);

struct LowerHex<T>(T);

impl<T: fmt::LowerHex> fmt::Debug for LowerHex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

struct UpperHex<T>(T);

impl<T: fmt::UpperHex> fmt::Debug for UpperHex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Debug for RunnerData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut output = if i == self.1 {
                        "<".to_string()
                    } else {
                        String::new()
                    };

                    let mut value = v.to_string();

                    if value.len() == 1 {
                        value.insert(0, '0');
                    }

                    output += &value;

                    if i == self.1 {
                        output.push('>');
                    }

                    output
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
        )
    }
}

impl fmt::LowerHex for RunnerData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut output = if i == self.1 {
                        "<".to_string()
                    } else {
                        String::new()
                    };

                    output.push(
                        "0123456789abcdef"
                            .chars()
                            .nth(*v as usize / 16)
                            .expect("there will always be a character here"),
                    );

                    output.push(
                        "0123456789abcdef"
                            .chars()
                            .nth(*v as usize % 16)
                            .expect("there will always be a character here"),
                    );

                    if i == self.1 {
                        output.push('>');
                    }

                    output
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
        )
    }
}

impl fmt::UpperHex for RunnerData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut output = if i == self.1 {
                        "<".to_string()
                    } else {
                        String::new()
                    };

                    output.push(
                        "0123456789ABCDEF"
                            .chars()
                            .nth(*v as usize / 16)
                            .expect("there will always be a character here"),
                    );

                    output.push(
                        "0123456789ABCDEF"
                            .chars()
                            .nth(*v as usize % 16)
                            .expect("there will always be a character here"),
                    );

                    if i == self.1 {
                        output.push('>');
                    }

                    output
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
        )
    }
}

impl<const N: usize> fmt::Debug for Runner<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DEBUG_DATA_WIDTH: i32 = 8;

        f.debug_struct("Runner")
            .field(
                "data",
                &RunnerData(
                    &self.memory[0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize
                        ..(N as i32).min(self.pointer as i32 + DEBUG_DATA_WIDTH) as usize],
                    self.pointer - 0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize,
                ),
            )
            .field("input", &self.input.iter().rev().collect::<Vec<_>>())
            .field("output", &self.output)
            .finish()
    }
}

impl<const N: usize> fmt::LowerHex for Runner<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DEBUG_DATA_WIDTH: i32 = 8;

        f.debug_struct("Runner")
            .field(
                "data",
                &LowerHex(RunnerData(
                    &self.memory[0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize
                        ..(N as i32).min(self.pointer as i32 + DEBUG_DATA_WIDTH) as usize],
                    self.pointer - 0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize,
                )),
            )
            .field("input", &self.input.iter().rev().collect::<Vec<_>>())
            .field("output", &self.output)
            .finish()
    }
}

impl<const N: usize> fmt::UpperHex for Runner<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const DEBUG_DATA_WIDTH: i32 = 8;

        f.debug_struct("Runner")
            .field(
                "data",
                &UpperHex(RunnerData(
                    &self.memory[0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize
                        ..(N as i32).min(self.pointer as i32 + DEBUG_DATA_WIDTH) as usize],
                    self.pointer - 0i32.max(self.pointer as i32 - DEBUG_DATA_WIDTH) as usize,
                )),
            )
            .field("input", &self.input.iter().rev().collect::<Vec<_>>())
            .field("output", &self.output)
            .finish()
    }
}
