//! Onca's regex library
//! 
//! This library supports a combined version of both PCRE and Javascript regex syntax.
//! 
//! # Limitations
//! 
//! - Only supports UTF-8, so no UTF-16, UTF-32, or EBCDIC support
//! - Unicode property support is always enabled
//! - No control of optimization
//! - Always uses any new line, meaning: CR, LF, CRLF,  VT (Vertical Tab, U+000B), FF (Form Feed, U+000C), NEL (NExt Line, U+0085), LS (Line Separator, U+2028), and PS (Paragraph Separator, U+2029)
//! - No support for PCRE options, i.e. '(*UTF)'
//! - In PCRE Mode, the \C escape sequence is not suppported
//! - No support for \cx
//! - Any digit other than 0 in an escape sequence, i.e. \0, is not allowed, it is preferable to use \o{..} for octal numbers and \g{..} for back references

#![feature(is_ascii_octdigit)]
#![feature(let_chains)]
#![feature(iter_advance_by)]
#![feature(round_char_boundary)]

use std::{collections::HashMap, ops::Range};

use matcher::Matcher;
use onca_common::prelude::*;

use onca_common_macros::flags;
use onca_unicode_info as unicode;
use opt_process::RegexProcessor;
use parse::Parser;

mod parse;
mod opt_process;
mod matcher;

/// Regex flags
#[flags]
pub enum RegexFlags {
	/// Case is ignored when matching literals.
	Caseless,
	/// Support multi line regexes.
	Multiline,
	/// Only match dollar at the end of a string and not on line wraps.
	DollarEndOnly,
	/// A dot matches all characters, without exception.
	DotAll,
	/// Allow a name to be used for multiple captures with a different index.
	DuplicateNames,
	/// If a regex tries to match an empty regex, it will pass and be treated as if the back reference doesn't exist
	AllowEmtpyBackRefs,
}


#[derive(Clone, Copy, Default)]
struct RegexRange {
	pub begin: u16,
	pub end:   u16
}

impl RegexRange {
	pub fn to_range(self) -> Range<usize> {
		Range { start: self.begin as usize, end: self.end as usize }
	}

	pub fn is_empty(self) -> bool {
		self.begin == self.end
	}
}


enum RepetitionMode {
	Exactly(u16),
	AtLeast(u16),
	AtLeastAtMost(u16, u16),
}

enum RepetitionStrategy {
	Greedy,
	Possessive,
	Lazy,
}

#[derive(PartialEq, Eq)]
enum CharacterClass {
	HorizontalWhitespace,
	VerticalWhitespace,
	Whitespace,
	Word,
	NonNewLine,
	Category(unicode::Category),
	Script(unicode::Script),
	PosixSpace,
	UNC,
	PosixAscii,
	PosixGraph,
	PosixPrint,
	PosixXDigit,
	AtomicNewLine,
	ExtendedGraphemeCluster,
	Any,
}

#[flags]
enum RegexFlagChange {
	CaselessOff,
	CaselessOn,
	MultilineOff,
	MultilineOn,
	DotAllOff,
	DotAllOn,
	ExtendedOff,
	ExtendedOn,
}
 
enum RegexNode {
	// Special node that does nothing, used to handle things like \Q and \E
	None,
	// Collection of regex node without any other special meaning, used for Self::parse_nodes
	Unit(Vec<RegexNode>),
	Literal(String),
	LiteralChar(char),
	Dot,
	CharacterClass(CharacterClass, bool),
	CharacterClassChar(char),
	Alternation(Vec<Vec<RegexNode>>),
	Repetition(Box<RegexNode>, Vec<RegexNode>, RepetitionMode, RepetitionStrategy),
	StartOfString,
	EndOfString,
	InternalOptionSetting(RegexFlagChange),
	MatchStartReset,
	WordBoundary(bool),
	SubjectStart,
	SubjectEndOrNewline,
	SubjectEndOnly,
	FirstMatchPos,

	AbsBackRef(u16),
	NamedBackRef(String),

	Lookahead(Box<RegexNode>, bool),
	Lookbehind(Vec<RegexNode>, Vec<u16>, bool),

	AbsConditional(u16, Box<RegexNode>, Option<Box<RegexNode>>),
	NamedConditional(String, Box<RegexNode>, Option<Box<RegexNode>>),
	RecursiveConditional(u16, Box<RegexNode>, Option<Box<RegexNode>>),
	NamedRecursiveConditional(String, Box<RegexNode>, Option<Box<RegexNode>>),
	DefineConditional(Box<RegexNode>),
	AssertConditional(Box<RegexNode>, Box<RegexNode>, Option<Box<RegexNode>>),
	
	// Intermediate node to be split
	ParsedGroup(RegexFlagChange, Option<u16>, Box<RegexNode>, bool),
	Group{
		capture_idx: Option<u16>,
		sub_node:    Box<RegexNode>,
		atomic:      bool,
	},
	ClassDef(Vec<char>, Vec<(char, char)>, Vec<RegexNode>, bool),
}

impl RegexNode {
	fn allow_repetition(&self) -> bool {
		match self {
			Self::Literal(_) |
			Self::LiteralChar(_) |
			Self::CharacterClass(_, _) |
			Self::ClassDef(..) |
			Self::ParsedGroup(..) |
			Self::Group { .. } |
			Self::Dot => true,
			_ => false,
		}
	}

	fn get_fixed_length(&self) -> Option<u16> {
		match self {
			Self::Unit(nodes) => {
				let mut len = 0;
				for node in nodes {
					len += node.get_fixed_length()?;
				}
				Some(len)
			},
			Self::Literal(lit) => Some(lit.len() as u16),
			Self::LiteralChar(ch) => Some(ch.len_utf8() as u16),
			Self::Dot => Some(1),
			Self::CharacterClass(_, _) => todo!(),
			Self::CharacterClassChar(_) => Some(1),
			Self::Alternation(_) => None,
			Self::Repetition(node, tail, mode, _) => {
				let RepetitionMode::Exactly(count) = mode else { return None };
				let len = node.get_fixed_length()?;

				let mut tail_len = 0;
				for tail_node in tail {
					tail_len += tail_node.get_fixed_length()?;
				}
				Some(len * count + tail_len)
			},
			Self::StartOfString => Some(0),
			Self::EndOfString => Some(0),
			Self::InternalOptionSetting(_) => Some(0),
			Self::MatchStartReset => Some(0),
			Self::WordBoundary(_) => Some(0),
			Self::SubjectStart => Some(0),
			Self::SubjectEndOrNewline => Some(0),
			Self::SubjectEndOnly => Some(0),
			Self::FirstMatchPos => Some(0),
			Self::Lookahead(..) => Some(0),
			Self::Lookbehind(..) => Some(0),
			Self::ParsedGroup(..) => unreachable!("This node should already have been replaced before this call"),
			Self::Group { sub_node, .. } => {
				Some(sub_node.get_fixed_length()?)
			},
			Self::ClassDef(_, _, nodes, _) => {
				let mut len = None;
				for node in nodes {
					match len {
						None => len = Some(node.get_fixed_length()?),
						Some(len) => {
							let node_len = node.get_fixed_length()?;
							if node_len != len {
								return None;
							}
						}
					}
				}
				len
			},
			_ => None
		}
	}
}

/// Regex parsing error
#[derive(Debug)]
pub struct RegexError{
	/// Regex being processed
	pub regex: String,
    /// Error message.
    pub msg:   String,
    /// Index of error range start.
    pub begin: usize,
    /// Index of error range end (inclusive).
    pub end:   usize,
}

impl RegexError {
	fn new(msg: String, begin: usize, end: usize) -> Self {
		Self { regex: String::new(), msg, begin, end }
	}

	fn new_str(msg: &str, begin: usize, end: usize) -> Self {
		Self { regex: String::new(), msg: msg.to_string(), begin, end }
	}
}

// TODO: Match and Recursion limits
// TODO: Composable regexes (need to change parsing to only store relative capture indices)
pub struct RegexOptions {
	pub flags:      RegexFlags,

}

pub struct Regex {
	node:          RegexNode,
	capture_names: HashMap<String, Vec<u16>>,
	flags:         RegexFlags,
}

impl Regex {
	pub fn new(regex: &str, flags: RegexFlags) -> Result<Self, RegexError> {
		let parser = Parser::new(regex, flags);
		let (mut node, capture_names) = match parser.parse() {
			Ok(tup) => tup,
			Err(mut err) => {
				err.regex = regex.to_string();
				return Err(err);
			}
		};
		
		let processor = RegexProcessor::new();
		if let Err(mut err) = processor.process_and_optimize(&mut node) {
			err.regex = regex.to_string();
			return Err(err);
		}

		Ok(Self { node, capture_names, flags })
	}

	/// Check if a string matches the regex entirely, if so, return a result with the captures.
	pub fn is_match<'a>(&'a self, s: &'a str) -> Option<MatchResult<'a>> {
		let mut matcher = Matcher::new(s, self.flags, &self.capture_names, true);
		if let Some(captures) = matcher.find(&self.node) && matcher.is_empty() {
			return Some(MatchResult{
				regex: self,
				s,
				captures,
			})
		} else {
			None
		}
	}

	/// Check if a string contains the regex, if so, return the byte index into the string and a result with the captures.
	pub fn contains<'a>(&'a self, s: &'a str) -> Option<(usize, MatchResult<'a>)> {
		for (idx, _) in s.char_indices() {
			let mut matcher = Matcher::new(&s[idx..], self.flags, &self.capture_names, idx == 0);
			if let Some(captures) = matcher.find(&self.node) {
				return Some((idx, MatchResult{
    			    regex: self,
    			    s,
    			    captures,
    			}))
			}
		}
		None
	}
}

pub struct MatchResult<'a> {
	regex:    &'a Regex,
	s:        &'a str,
	captures: Vec<RegexRange>,
}

impl MatchResult<'_> {
	pub fn has_capture(&self, idx: u16) -> bool {
		let idx = idx as usize;
		idx < self.captures.len() && !self.captures[idx].is_empty()
	}

	pub fn has_capture_by_name(&self, name: &str) -> bool {
		let Some(indices) = self.regex.capture_names.get(name) else { return false; };
		for idx in indices {
			if self.has_capture(*idx) {
				return true;
			}
		}
		false
	}

	pub fn get_capture(&self, idx: u16) -> Option<&str> {
		let idx = idx as usize;
		if idx < self.captures.len() && self.captures[idx].is_empty() {
			let range = self.captures[idx];
			Some(&self.s[range.to_range()])
		} else {
			None
		}

		// let range = self.captures.get(&idx)?;
		// Some(&self.s[range.to_range()])
	}

	pub fn get_capture_by_name(&self, name: &str) -> Option<&str> {
		let indices = self.regex.capture_names.get(name)?;
		for idx in indices.iter().rev() {
			if let Some(s) = self.get_capture(*idx) {
				return Some(s);
			}
		}
		None
	}
}


#[cfg(test)]
mod tests;