use std::{
    collections::VecDeque,
    io::{BufRead, ErrorKind, Read},
    str, u8,
};

use rio_api::parser::LineBytePosition;

use super::{error::*, MAX_STACK_SIZE};

/// Reads the file in streaming
pub struct LookAheadByteReader<R: Read> {
    inner: R,
    buffer: VecDeque<u8>,
    current: Option<u8>,
    line_number: u64,
    byte_number: u64,
    stack_size: usize,
}

const DEFAULT_CAPACITY: usize = 8 * 1024;

impl<R: BufRead> LookAheadByteReader<R> {
    pub fn new(inner: R) -> Self {
        let mut buffer = VecDeque::with_capacity(DEFAULT_CAPACITY);
        buffer.push_back(b'\n');
        Self {
            inner,
            buffer,
            current: Some(b'\n'),
            line_number: 0,
            byte_number: 1,
            stack_size: 0,
        }
    }

    /// Returns the current byte if it exists
    pub fn current(&self) -> Option<u8> {
        self.current
    }
    /// Returns the current byte if it exists or fail if it does not
    pub fn required_current(&self) -> Result<u8, TurtleError> {
        self.current()
            .ok_or_else(|| self.parse_error(TurtleErrorKind::PrematureEof))
    }

    /// Returns the next byte if it exists
    pub fn next(&mut self) -> Result<Option<u8>, TurtleError> {
        self.ahead(1)
    }

    /// Returns the next byte if it exists or fail if it does not
    pub fn required_next(&mut self) -> Result<u8, TurtleError> {
        self.ahead(1)?
            .ok_or_else(|| self.parse_error(TurtleErrorKind::PrematureEof))
    }

    /// Returns a future byte if it exists
    pub fn ahead(&mut self, count: usize) -> Result<Option<u8>, TurtleError> {
        loop {
            let mut position = count;
            let (first, second) = self.buffer.as_slices();
            if position < first.len() {
                return Ok(Some(first[position]));
            }
            position -= first.len();
            if position < second.len() {
                return Ok(Some(second[position]));
            }
            if self.fill_and_is_end()? {
                return Ok(None);
            }
        }
    }

    /// Consumes the current char and moves to the next one
    pub fn consume(&mut self) -> Result<(), TurtleError> {
        self.consume_many(1)
    }

    /// Consumes the many chars and moves to the next one
    pub fn consume_many(&mut self, count: usize) -> Result<(), TurtleError> {
        for _ in 0..count {
            if self.buffer.is_empty() {
                self.fill_and_is_end()?;
            }
            if let Some(c) = self.buffer.pop_front() {
                if c == b'\n' {
                    self.line_number += 1;
                    self.byte_number = 1;
                } else {
                    self.byte_number += 1;
                }
            } else {
                return Err(self.parse_error(TurtleErrorKind::PrematureEof));
            }
        }
        if self.buffer.is_empty() {
            self.fill_and_is_end()?;
        }
        self.current = self.buffer.front().cloned();
        Ok(())
    }

    /// Returns the line number of the current byte starting at 1
    pub fn line_number(&self) -> u64 {
        self.line_number
    }
    /// Returns the byte number of the current byte in the line starting at 1
    pub fn byte_number(&self) -> u64 {
        self.byte_number
    }

    /// Returns if the current buffer starts with a given byte string. Does not work cross line boundaries
    pub fn starts_with(&mut self, prefix: &[u8]) -> bool {
        self.starts_with_with_eq(prefix, |a, b| a == b)
    }

    /// Returns if the current buffer starts with a given byte string in an ASCII case insensitive manner.
    /// Does not work cross line boundaries
    pub fn starts_with_ignore_ascii_case(&mut self, prefix: &[u8]) -> bool {
        self.starts_with_with_eq(prefix, |a, b| a.eq_ignore_ascii_case(b))
    }

    pub fn unexpected_char_error<T>(&self) -> Result<T, TurtleError> {
        Err(self.parse_error(match self.current() {
            Some(c) => TurtleErrorKind::UnexpectedByte(c),
            None => TurtleErrorKind::PrematureEof,
        }))
    }

    pub fn check_is_current(&self, expected: u8) -> Result<(), TurtleError> {
        if self.current() == Some(expected) {
            Ok(())
        } else {
            self.unexpected_char_error()
        }
    }

    pub fn check_is_next(&mut self, expected: u8) -> Result<(), TurtleError> {
        if self.next()? == Some(expected) {
            Ok(())
        } else {
            self.unexpected_char_error()
        }
    }

    pub fn parse_error(&self, kind: TurtleErrorKind) -> TurtleError {
        TurtleError {
            kind,
            position: Some(LineBytePosition::new(
                self.line_number(),
                self.byte_number(),
            )),
        }
    }

    pub fn _consume_line_end(&mut self) -> Result<(), TurtleError> {
        loop {
            match self.current() {
                None => return Ok(()),
                Some(b'\n') => return self.consume(),
                _ => self.consume()?,
            }
        }
    }

    fn fill_and_is_end(&mut self) -> Result<bool, TurtleError> {
        loop {
            let mut buf = [0; DEFAULT_CAPACITY]; //TODO: increase?
            match self.inner.read(&mut buf) {
                Ok(0) => return Ok(true),
                Ok(read) => {
                    self.buffer.extend(buf[..read].iter());
                    return Ok(false);
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e.into()),
            }
        }
    }

    fn starts_with_with_eq(&mut self, prefix: &[u8], eq: impl Fn(&[u8], &[u8]) -> bool) -> bool {
        loop {
            let (first, second) = self.buffer.as_slices();
            if prefix.len() <= first.len() {
                return eq(&first[..prefix.len()], prefix);
            } else if prefix.len() <= first.len() + second.len() {
                return eq(first, &prefix[..first.len()])
                    && eq(
                        &second[..prefix.len() - first.len()],
                        &prefix[first.len()..],
                    );
            }
            if let Ok(true) | Err(_) = self.fill_and_is_end() {
                return false;
            }
        }
    }

    pub fn increment_stack_size(&mut self) -> Result<(), TurtleError> {
        self.stack_size += 1;
        if self.stack_size > MAX_STACK_SIZE {
            Err(self.parse_error(TurtleErrorKind::StackOverflow))
        } else {
            Ok(())
        }
    }

    pub fn decrement_stack_size(&mut self) {
        self.stack_size -= 1;
    }
}

#[derive(Default)]
pub struct StringBufferStack {
    inner: Vec<String>,
    len: usize,
}

impl StringBufferStack {
    pub fn _with_capacity(cap: usize) -> Self {
        StringBufferStack {
            inner: Vec::with_capacity(cap),
            len: 0,
        }
    }
    pub fn push(&mut self) -> &mut String {
        self.len += 1;
        if self.len > self.inner.len() {
            self.inner.push(String::default())
        }
        &mut self.inner[self.len - 1]
    }

    pub fn push2(&mut self) -> (&mut String, &mut String) {
        self.push();
        self.push();
        let (a1, a2) = self.inner.split_at_mut(self.len - 1);
        (&mut a1[a1.len() - 1], &mut a2[0])
    }

    pub fn pop(&mut self) {
        self.inner[self.len - 1].clear();
        self.len -= 1;
    }

    pub fn _clear(&mut self) {
        self.inner.clear();
        self.len = 0;
    }
}

#[derive(Default)]
pub struct BlankNodeIdGenerator {
    //TODO: avoid collisions
    counter: u64,
}

impl BlankNodeIdGenerator {
    pub fn generate(&mut self) -> BlankNodeId {
        let mut id: [u8; 12] = [
            // IMPORTANT: if this is modified, disambiguate must be updated accordingly
            b'r', b'i', b'o', b'g', b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0',
        ];
        self.counter += 1;
        write_u64_to_slice(self.counter, &mut id[4..]);
        BlankNodeId { id }
    }

    /// If label could have been generated by self, turn it into a label that could not.
    pub fn disambiguate(&self, label: &mut String) {
        const SUFFIX: u8 = b'd';
        let bytes = label.as_bytes();
        if bytes.len() >= 12
            && &bytes[..4] == b"riog"
            && bytes[4..12].iter().all(u8::is_ascii_digit)
            && bytes[12..].iter().all(|b| b == &SUFFIX)
        {
            label.push(SUFFIX as char)
        }
    }
}

fn write_u64_to_slice(mut v: u64, s: &mut [u8]) {
    for i in (0..s.len()).rev() {
        s[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct BlankNodeId {
    id: [u8; 12],
}

impl AsRef<str> for BlankNodeId {
    fn as_ref(&self) -> &str {
        // We know what id is and it's always valid UTF8
        str::from_utf8(&self.id).unwrap()
    }
}
