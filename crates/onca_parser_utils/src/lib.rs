#![feature(let_chains)]
#![feature(pattern)]

pub mod str_parser;

/// Parser error
#[derive(Clone, Copy, Debug)]
pub struct ParserError {
    pub line   : usize,
    pub column : usize,
    pub msg    : &'static str,
}