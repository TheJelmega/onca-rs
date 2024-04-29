use crate::*;

const HORIZONTAL_WHITESPACE_CHARS: [char; 19] = [
    '\u{0009}',
    '\u{0020}',
    '\u{00A0}',
    '\u{1680}',
    '\u{180E}',
    '\u{2000}',
    '\u{2001}',
    '\u{2002}',
    '\u{2003}',
    '\u{2004}',
    '\u{2005}',
    '\u{2006}',
    '\u{2007}',
    '\u{2008}',
    '\u{2009}',
    '\u{200A}',
    '\u{202F}',
    '\u{205F}',
    '\u{3000}',
];

const VERTICAL_WHITESPACE_CHARS: [char; 7] = [
	'\u{000A}',
	'\u{000B}',
	'\u{000C}',
	'\u{000D}',
	'\u{0085}',
	'\u{2028}',
	'\u{2029}',
];

const WHITESPACE_CHARS: [char; 26] = [
	'\u{0009}',
	'\u{0029}',
	'\u{00A0}',
	'\u{1680}',
	'\u{180E}',
	'\u{2000}',
	'\u{2001}',
	'\u{2002}',
	'\u{2003}',
	'\u{2004}',
	'\u{2005}',
	'\u{2006}',
	'\u{2007}',
	'\u{2008}',
	'\u{2009}',
	'\u{200A}',
	'\u{202F}',
	'\u{205F}',
	'\u{3000}',
	'\u{000A}',
	'\u{000B}',
	'\u{000C}',
	'\u{000D}',
	'\u{0085}',
	'\u{2028}',
	'\u{2029}',
];

const NEWLINE_CHARS: [char; 7] = [
	'\r',
	'\n',
	'\u{000b}',
	'\u{000c}',
	'\u{0085}',
	'\u{2028}',
	'\u{2029}',
];

const NEWLINE_CHARS_STR: [&'static str; 8] = [
	"\r",
	"\n",
	"\r\n",
	"\u{000b}",
	"\u{000c}",
	"\u{0085}",
	"\u{2028}",
	"\u{2029}",
];

const LINE_WRAPS: [&'static str; 2] = [
	"\n",
	"\r\n"
];

pub(crate) struct Matcher<'a> {
    flags:          RegexFlags,
    orig:           &'a str,
    cursor:         &'a str,
    index:          usize,
	atomic_index:   usize,
	captures:       Vec<RegexRange>,
	capture_names:  &'a HashMap<String, Vec<u16>>,
	enable_capture: bool,
    start_from_0:   bool,
}

impl<'a> Matcher<'a> {
    pub fn new(s: &'a str, flags: RegexFlags, capture_names: &'a HashMap<String, Vec<u16>>, start_from_0: bool) -> Self {
        Self {
		    flags,
		    orig: s,
		    cursor: s,
		    index: 0,
			atomic_index: 0,
		    captures: Vec::new(),
			capture_names,
			enable_capture: true,
            start_from_0,
		}
    }

    pub fn is_empty(&self) -> bool {
        self.cursor.is_empty()
    }

    pub fn find(&mut self, node: &RegexNode) -> Option<Vec<RegexRange>> {
        if self.find_and_match(node) {
            Some(core::mem::take(&mut self.captures))
        } else {
            None
        }
    }

    pub fn find_and_match(&mut self, node: &RegexNode) -> bool {
        match node {
			RegexNode::None => true,
			RegexNode::Unit(nodes) => {
				for node in nodes {
					if !self.find_and_match(node) {
						return false;
					}
				}
				true
			}
		    RegexNode::Literal(literal) => if self.flags.contains(RegexFlags::Caseless) {
                let lower_lit = literal.to_lowercase();
                let lower_cursor = self.cursor.to_lowercase();

                if lower_cursor.starts_with(&lower_lit) {
                    self.move_equivalent(literal)
                } else {
                    false
                }
            } else {
                if self.cursor.starts_with(literal) {
		    		self.move_cursor(literal.len())
		    	} else {
		    		false
		    	}
            },
			RegexNode::LiteralChar(ch) => if self.flags.contains(RegexFlags::Caseless) {
                let lower_ch = ch.to_lowercase();
                let Some(first_ch) = self.cursor.chars().next() else { return false; };
                let first_ch = first_ch.to_lowercase();

                let res = lower_ch.clone().len() == first_ch.clone().len() && lower_ch.zip(first_ch).all(|(a, b)| a == b);

                if res {
                    self.move_cursor(ch.len_utf8())
                } else {
                    false
                }
            } else {
                if self.cursor.starts_with(*ch) {
			    	self.move_cursor(ch.len_utf8())
			    } else {
			    	false
			    }
            },
			RegexNode::Dot => if self.flags.contains(RegexFlags::DotAll) || LINE_WRAPS.iter().any(|s| !self.cursor.starts_with(s)) {
				let mut chars = self.cursor.chars();
				let Some(ch) = chars.next() else { return false; };
				self.move_cursor(ch.len_utf8())
			} else {
				false
			},
			RegexNode::CharacterClass(class, expected) => {
				let mut chars = self.cursor.chars();
				let Some(ch) = chars.next() else { return false; };
				let res = match class {
        			CharacterClass::HorizontalWhitespace => HORIZONTAL_WHITESPACE_CHARS.contains(&ch),
        			CharacterClass::VerticalWhitespace   => VERTICAL_WHITESPACE_CHARS.contains(&ch),
        			CharacterClass::Whitespace           => WHITESPACE_CHARS.contains(&ch),
        			CharacterClass::Word                 => ch == '_' || ch.is_alphanumeric(),
					CharacterClass::NonNewLine           => !NEWLINE_CHARS.contains(&ch),
					CharacterClass::Category(cat)        => unicode::get_category(ch as u32).is_some_and(|val| val.intersects(*cat)),
					CharacterClass::Script(script)       => unicode::get_script(ch).map_or(false, |val| val == *script) ||
						                                    unicode::get_script_extensions(ch).is_some_and(|val| val.contains(script)),
					CharacterClass::PosixSpace           => ch == '\u{0C}' || unicode::get_category(ch as u32).is_some_and(|val| val.intersects(unicode::Category::Separator)),
					CharacterClass::UNC                  => ch == '$' || ch == '@'|| ch == '`' || (ch as u32 >= 0xA0 && ((ch as u32) < 0xD800 || ch as u32 > 0xDFFF )),
					CharacterClass::PosixAscii           => (ch as u32) <= 127,
					CharacterClass::PosixGraph           |
					CharacterClass::PosixPrint           => ch != '\u{061C}' &&
						ch  != '\u{180E}' &&
						!(ch >= '\u{2066}' && ch <= '\u{2069}') &&
						unicode::get_category(ch as u32).is_some_and(|val| val.intersects(
							unicode::Category::Letter |
							unicode::Category::Mark |
							unicode::Category::Number |
							unicode::Category::Punctuation |
							unicode::Category::Symbol |
							unicode::Category::Format |
							if *class == CharacterClass::PosixPrint { unicode::Category::SpaceSeparator } else { unicode::Category::None }
						)),
					CharacterClass::PosixXDigit          => (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F'),
					CharacterClass::Any                  => true,
					_ => todo!(),
    			};

				// Either value needs to be true, i.e. (false, true) or (true, false) only
				if res == *expected {
					self.move_cursor(ch.len_utf8())
				} else {
					false
				}
			},
			RegexNode::Alternation(options) => {
				'outer: for nodes in options {
					for node in nodes {
						if !self.find_and_match(node) {
							continue 'outer;
						}
					}
					return true;
				}
				false
			}
			RegexNode::Repetition(sub, tail, mode, algo) => {
				let (min, max) = match mode {
    			    RepetitionMode::Exactly(n)          => (*n, *n),
    			    RepetitionMode::AtLeast(n)          => (*n, u16::MAX),
    			    RepetitionMode::AtLeastAtMost(n, m) => (*n, *m),
    			};

				// Collect the minimum amount of values, the rest depends on the algo
				for _ in 0..min {
					if !self.find_and_match(sub) {
						return false;
					}
				}

				match algo {
    			    RepetitionStrategy::Greedy => {
						// Collect remainder greedily
						// Start by collecting as many elements as possible, and store the indices each starts at, so we don't do redundant checks

						let idx = self.index;

						// TODO: Stack instead of Vec
						let mut idx_stack = vec![idx];
						for _ in min..max {
							let prev_idx = self.index;
							if self.find_and_match(sub) {
								// Early out if pass didn't capture anything, this is defined in the documentation as expected behavior and to prevent runaway matches
								if self.index == prev_idx {
									break;
								}

								idx_stack.push(self.index);
							} else {
								break;
							}
						}

						// Not iterate from last the first and see if we can match anything else
						'rep_loop: for idx in idx_stack.iter().rev() {
							// Early out for atomic captures
							if *idx < self.atomic_index {
								return false;
							}

							self.reset(*idx);
							// Process tail
							for elem in tail {
								if !self.find_and_match(elem) {
									continue 'rep_loop;
								}
							}

							if !tail.is_empty() && self.index == *idx {
								return false;
							} else {
								return true;
							}
						}
						false
					},
    			    RepetitionStrategy::Possessive => {
						// Collect remainder possessivly
						for _ in min..max {
							let prev_idx = self.index;
							if !self.find_and_match(sub) {
								// Early out if pass didn't capture anything, this is defined in the documentation as expected behavior and to prevent runaway matches
								if self.index == prev_idx {
									break;
								}

								break;
							}
						}
						self.atomic_index = self.index;
					
						// Process tail
						for elem in tail {
							if !self.find_and_match(elem) {
								return false;
							}
						}
						true
					},
    			    RepetitionStrategy::Lazy => {
						// Incrementally collect new elements and try to match the rest of the string
						'outer: for i in min..max {
							let prev_idx = self.index;
							if i != min {
								if !self.find_and_match(sub) {
									return false;
								}
							}

							let idx = self.index;
							for elem in tail {
								if !self.find_and_match(elem) {
									// Early out if pass didn't capture anything, this is defined in the documentation as expected behavior and to prevent runaway matches
									if self.index == prev_idx {
										return tail.is_empty();
									}

									self.reset(idx);
									continue 'outer;
								}
							}
							return true;
						}
						true
					},
    			}
			},
			RegexNode::Group{ capture_idx, sub_node, atomic } => {
				let start_idx = self.index;
				if !self.find_and_match(sub_node) {
					return false;
				}

				if self.enable_capture {
					if let Some(capture_idx) = capture_idx {
						if self.captures.len() <= *capture_idx as usize {
							self.captures.resize(*capture_idx as usize + 1, RegexRange::default());
						}
						self.captures[*capture_idx as usize] = RegexRange{ begin: start_idx as u16, end: self.index as u16 };
					}
				}	

				// Atomic groups consume no matter what, so make sure to do this here
				if *atomic {
					self.atomic_index = self.index;
				}

				true
			},
			RegexNode::ClassDef(chars, ranges, nodes, expected) => {
				let Some(ch) = self.cursor.chars().next() else { return false };
				let start_idx = self.index;
				if chars.contains(&ch) ||
					ranges.iter().any(|(begin, end)| *begin <= ch && ch <= *end) ||
					nodes.iter().any(|node| self.find_and_match(node))
				{
					if *expected {
						if self.index == start_idx {
							self.move_cursor(ch.len_utf8());
						}
						return true;
					} else {
						return false;
					}
				}

				if *expected {
					false
				} else {
					self.move_cursor(ch.len_utf8())
				}
			},
			RegexNode::CharacterClassChar(_) => panic!("A CharacterClassChar should never appear in a compiled regex"),
			RegexNode::StartOfString => self.is_at_start_boundary(),
			RegexNode::EndOfString => self.is_at_end_boundary(),
			RegexNode::InternalOptionSetting(flag_change) => {
                if flag_change.contains(RegexFlagChange::CaselessOff) {
                    self.flags &= !RegexFlags::Caseless;
                } else if flag_change.contains(RegexFlagChange::CaselessOn) {
                    self.flags |= RegexFlags::Caseless;
                }

                if flag_change.contains(RegexFlagChange::MultilineOff) {
                    self.flags &= !RegexFlags::Multiline;
                } else if flag_change.contains(RegexFlagChange::MultilineOn) {
                    self.flags |= RegexFlags::Multiline;
                }

                if flag_change.contains(RegexFlagChange::DotAllOff) {
                    self.flags &= !RegexFlags::DotAll;
                } else if flag_change.contains(RegexFlagChange::DotAllOn) {
                    self.flags |= RegexFlags::DotAll;
                }

				true
			},
			RegexNode::WordBoundary(expected) => {
				let is_prev_word = if self.index == 0 {
					true
				} else {
					let Some(ch) = self.orig[self.index..].chars().next() else { return false };
					ch == '_' || ch.is_alphanumeric()
				};

				let is_next_char = if let Some(ch) = self.cursor.chars().next() {
					ch == '_' || ch.is_alphanumeric()
				} else {
					return false;
				};

				let same = is_prev_word == is_next_char;
				same == *expected
			},
			RegexNode::SubjectStart => self.index == 0,
			RegexNode::SubjectEndOrNewline => {
				self.cursor.is_empty() ||
					NEWLINE_CHARS_STR.iter().any(|wrap| self.cursor == *wrap)
			}
			RegexNode::SubjectEndOnly => self.cursor.is_empty(),
			RegexNode::AbsBackRef(idx) => {
				let idx = *idx as usize;
				if idx >= self.captures.len() {
					return self.flags.contains(RegexFlags::AllowEmtpyBackRefs);
				}

				let capture = self.captures[idx];
				if capture.is_empty() {
					return self.flags.contains(RegexFlags::AllowEmtpyBackRefs);
				}

				let capture_s = &self.orig[capture.to_range()];
				if self.cursor.starts_with(capture_s) {
					self.move_cursor(capture_s.len());
					true
				} else {
					false
				}
			},
			RegexNode::NamedBackRef(name) => {
				let Some(indices) = self.capture_names.get(name) else { return false };
				for idx in indices.iter().rev() {
					let idx = *idx as usize;
					if idx >= self.captures.len() {
						continue;
					}

					let capture = self.captures[idx];
					if capture.is_empty() {
						continue;
					}

					let capture_s = &self.orig[capture.to_range()];
					if self.cursor.starts_with(capture_s) {
						self.move_cursor(capture_s.len());
						return true;
					}
				}
				self.flags.contains(RegexFlags::AllowEmtpyBackRefs)
			}
			RegexNode::Lookahead(inner, expected) => {
				let idx = self.index;
				let prev_enable_capture = self.enable_capture;
				self.enable_capture = false;

				let res = self.find_and_match(inner) == *expected;

				self.reset(idx);
				self.enable_capture = prev_enable_capture;
				res
			},
			RegexNode::Lookbehind(nodes, fixed_lengths, expected) => {
				let idx = self.index;
				let prev_enable_capture = self.enable_capture;
				self.enable_capture = false;

				let mut res = false;
				for (node, len) in nodes.iter().zip(fixed_lengths.iter()) {
					let len = *len as usize;
					if len > idx {
						continue;
					}

					if !self.reset_to_char_boundary(idx - len) {
						continue;
					}

					let tmp = self.find_and_match(node);
					res |= tmp == *expected;
				}

				self.reset(idx);
				self.enable_capture = prev_enable_capture;
				res
			},
			RegexNode::AbsConditional(capture, yes, no) => {
				if self.captures.get(*capture as usize).map_or(false, |range| !range.is_empty()) {
					self.find_and_match(yes)
				} else if let Some(no) = no {
					self.find_and_match(no)
				} else {
					false
				}
			},
			RegexNode::NamedConditional(capture, yes, no) => {
				let Some(indices) = self.capture_names.get(capture) else { return false; };
				let mut cond = false;
				for idx in indices {
					 if self.captures.get(*idx as usize).map_or(false, |range| !range.is_empty()) {
						cond = true;
						break;
					 }
				}

				if cond {
					self.find_and_match(yes)
				} else if let Some(no) = no {
					self.find_and_match(no)
				} else {
					false
				}
			}
			RegexNode::RecursiveConditional(_recursion, _yes, _no) => {
				todo!("Recursion is currently unsupported")
			},
			RegexNode::NamedRecursiveConditional(_recursion, _yes, _no) => {
				todo!("Recursion is currently unsupported")
			},
			RegexNode::AssertConditional(cond, yes, no) => {
				let idx = self.index;
				let prev_enable_capture = self.enable_capture;
				self.enable_capture = false;

				let cond = self.find_and_match(cond);

				self.enable_capture = prev_enable_capture;
				self.reset(idx);

				if cond {
					self.find_and_match(yes)
				} else if let Some(no) = no {
					self.find_and_match(no)
				} else {
					false
				}
			},
			RegexNode::FirstMatchPos => self.start_from_0 && self.index == 0,
			RegexNode::DefineConditional(_) => true,
			RegexNode::MatchStartReset => unreachable!("Match start reset should have been optimized out"),
			RegexNode::ParsedGroup(..) => unreachable!("Parsed groups should have been optimized out"),
		}
    }
    
	fn move_cursor(&mut self, offset: usize) -> bool {
		let index = self.index + offset;
		if index <= self.orig.len() {
			self.index = index;
			self.cursor = &self.orig[index..];
			true
		} else {
			false
		}
	}

    fn move_equivalent(&mut self, s: &str) -> bool {
        let num_chars = s.chars().count();
        let Some((index, _)) = self.cursor.char_indices().nth(num_chars) else { return false; };
        self.move_cursor(index)
    }

	fn reset(&mut self, idx: usize) {
		self.index = idx;
		self.cursor = &self.orig[self.index..];
	}

	fn reset_to_char_boundary(&mut self, idx: usize) -> bool {
		if self.cursor.is_char_boundary(idx) {
			self.reset(idx);
			true
		} else {
			false
		}
	}

	fn is_at_start_boundary(&self) -> bool {
		self.index == 0 ||
			(self.flags.contains(RegexFlags::Multiline) && 
			 NEWLINE_CHARS_STR.iter().any(|wrap| self.cursor.starts_with(wrap)) &&
			 self.cursor.len() != 1 &&
			 self.cursor != "\r\n"
			)
	}

	fn is_at_end_boundary(&self) -> bool {
		self.cursor.is_empty() ||
		NEWLINE_CHARS_STR.iter().any(|wrap| self.cursor == *wrap) ||
			(!self.flags.contains(RegexFlags::DollarEndOnly) && 
				LINE_WRAPS.iter().any(|wrap| self.cursor.starts_with(wrap)))
	}
}