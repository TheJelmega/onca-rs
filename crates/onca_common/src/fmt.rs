use std::{borrow::BorrowMut, fmt::Formatter};

enum IndentationFormat<'a> {
    Spaced(usize),
    Str(&'a str),
    Custom(&'a mut Inserter)
}

pub type Inserter = dyn FnMut(&mut dyn core::fmt::Write, usize, &str) -> core::fmt::Result;

pub struct Indenter<'a, 'b> {
    inner: &'a mut core::fmt::Formatter<'b>,
    format: IndentationFormat<'a>,
    needs_indent: bool,
}

impl<'a, 'b> Indenter<'a, 'b> {
    pub fn new(f: &'a mut Formatter<'b>) -> Self {
        Self {
            inner: f,
            format: IndentationFormat::Spaced(4),
            needs_indent: true,
        }
    }

    pub fn with_spaced(f: &'a mut Formatter<'b>, spaces: usize) -> Self {
        Self {
            inner: f,
            format: IndentationFormat::Spaced(spaces),
            needs_indent: true,
        }
    }
    
    pub fn with_str(f: &'a mut Formatter<'b>, s: &'a str) -> Self {
        Self {
            inner: f,
            format: IndentationFormat::Str(s),
            needs_indent: true,
        }
    }

    pub fn set_spaces(&mut self, spaces: usize) {
        self.format = IndentationFormat::Spaced(spaces)
    }

    pub fn set_str(&mut self, s: &'a str) {
        self.format = IndentationFormat::Str(s)
    }

    pub fn write_indent(&mut self, idx: usize, line: &str) -> core::fmt::Result {
        match &mut self.format {
            IndentationFormat::Spaced(size) => write!(self.inner, "{: >size$}", ""),
            IndentationFormat::Str(s) => write!(self.inner, "{s}"),
            IndentationFormat::Custom(inserter) => inserter(self.inner, idx, line),
        }
    }
}

impl core::fmt::Write for Indenter<'_, '_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for (idx, line) in s.split('\n').enumerate() {
            if idx > 0 {
                self.inner.write_char('\n');

                // Update so we draw another indent at the start of the line
                self.needs_indent = true;
            }
            if line.is_empty() {
                continue;
            }

            if self.needs_indent {
                self.write_indent(idx, line)?;
                // Make sure not to indent elements being formatted into the string
                self.needs_indent = false;
            }

            self.inner.write_str(line)?;
        }
        Ok(())
    }
}