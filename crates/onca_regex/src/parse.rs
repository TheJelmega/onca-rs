use crate::*;

const PARSE_SPECIAL_CHARS: [char; 12] = [
	'\\',
	'(',
	')',
	'|',
	'?',
	'*',
	'+',
	'{',
    '[',
    ']',
	'^',
	'$',
];

#[derive(PartialEq, Eq)]
enum ParseMode {
	Default,
	ClassDef,
}

pub(crate) struct Parser<'a> {
    orig:                 &'a str,
    cursor:               &'a str,
    idx:                  usize,
    capture_idx:          u16,
	max_capture_idx:      u16,
	dup_subpattern_start: Option<u16>,
	no_special_meaning:   bool,
	capture_names:        HashMap<String, Vec<u16>>,
	flags:                RegexFlags,
}

impl<'a> Parser<'a> {
    pub fn new(regex: &'a str, flags: RegexFlags) -> Self {
        Self {
            orig: regex,
            cursor: regex,
            idx: 0,
            capture_idx: 0,
			max_capture_idx: 1,
			dup_subpattern_start: None,
			no_special_meaning: false,
			capture_names: HashMap::new(),
			flags,
        }
    }

    pub fn parse(mut self) -> Result<(RegexNode, HashMap<String, Vec<u16>>), RegexError> {
        let node = self.parse_nodes(true)?;
        Ok((node, self.capture_names))
    }

    fn consume_count(&mut self, count: usize) {
        self.idx += count;
        self.cursor = &self.cursor[count..];
    }

    fn consume(&mut self, s: &str, err_reason: &str) -> Result<(), RegexError> {
        if self.cursor.starts_with(s) {
            self.consume_count(s.len());
            Ok(())
        } else {
            let len = s.len().max(self.cursor.len());
            let len = s.ceil_char_boundary(len);
            Err(RegexError::new(format!("Expected '{s}', found '{}' (reason: {err_reason})", &self.cursor[..len]), self.idx, self.idx + len))
        }
    }

    fn try_consume(&mut self, s: &str) -> bool {
        if self.cursor.starts_with(s) {
            self.consume_count(s.len());
            true
        } else {
            false
        }
    }

    fn set_idx(&mut self, idx: usize) {
        self.idx = idx;
        self.cursor = &self.orig;
    }

	fn gen_capture(&mut self) -> Result<u16, RegexError> {
		if self.capture_idx == u16::MAX {
			Err(RegexError::new_str("Ran out of capture indices, only 65535 numbered captures are supported", 0, 0))
		} else {	
			self.capture_idx += 1;
			self.max_capture_idx = self.max_capture_idx.max(self.capture_idx);
			Ok(self.capture_idx)
		}
	}

	fn begin_capture_alteration(&mut self) {
		self.dup_subpattern_start = Some(self.capture_idx);
	}

	fn end_capture_alteration(&mut self) {
		self.capture_idx = self.max_capture_idx;
		self.dup_subpattern_start = None;
	}

    fn add_capture_name(&mut self, name: &str, idx: u16) -> Result<(), RegexError> {
        let entry = self.capture_names.get_mut(name);
		if let Some(entry) = entry {
			if entry.contains(&idx) && !self.flags.contains(RegexFlags::DuplicateNames) {
				return Err(RegexError::new(
					format!("Cannot have a duplicate name for multiple captures that have different indices. If you want this, make sure to pass the DuplicateNames flag. Capture name: {name}, index: {idx}"),
					self.idx - name.len(), self.idx)
				);
			}

			entry.push(idx);

		} else {
			self.capture_names.insert(name.to_string(), vec![idx]);
		}
		Ok(())
    }



	fn parse_simple_node(&mut self) -> Result<RegexNode, RegexError> {
		let mut chars = self.cursor.chars();

		let Some(first_char) = chars.next() else { return Err(RegexError::new_str("Not enough characters", self.idx, self.idx)) };
		match first_char {
			'\\' => {
				self.consume_count(1);
				self.parse_escape_code(ParseMode::Default)
			},
			'.'  => {
				self.consume_count(1);
				Ok(RegexNode::Dot)
			},
			'[' => self.parse_class_definition(),
			'(' => self.parse_group(),
			'^' => {
				self.consume_count(1);
				Ok(RegexNode::StartOfString)
			},
			'$' => {
				self.consume_count(1);
				Ok(RegexNode::EndOfString)
			},
			_ => Ok(self.get_literal()),
		}
	}

	fn parse_node(&mut self) -> Result<RegexNode, RegexError> {
		let begin = self.idx;
		let node = self.parse_simple_node()?;

		let mut chars = self.cursor.chars();
		let Some(ch) = chars.next() else { return Ok(node) };

        if ['*', '+', '?', '{'].contains(&ch) {
            self.parse_repetition(ch, node, begin)
        } else {
            Ok(node)
        }
	}

    pub fn parse_repetition(&mut self, ch: char, inner: RegexNode, begin: usize) -> Result<RegexNode, RegexError> {
        if !inner.allow_repetition() {
            return Err(RegexError::new_str("Cannot apply repetition to this element", begin, self.idx))
        }

        // consume first token
        self.consume_count(1);

        let mode = match ch {
            '*' => RepetitionMode::AtLeast(0),
            '+' => RepetitionMode::AtLeast(1),
            '?' => RepetitionMode::AtLeastAtMost(0, 1),
            '{' => if !self.cursor.starts_with(|ch: char| ch.is_numeric()) {
                self.set_idx(self.idx - 1);
                return Ok(inner);
            } else {
                let Some(end) = self.cursor.find('}') else { return Err(RegexError::new_str("Repetition is not closed", self.idx + 1, self.idx + self.cursor.len())) };
                let comma = self.cursor[..end].find(',');

                let n_end = comma.unwrap_or(end);
                let n = self.cursor[..n_end].parse().map_err(|err| RegexError::new(format!("invalid repetition count ({err})"), self.idx, self.idx + end + 1 ))?;

                if let Some(comma) = comma {
                    if comma + 1 == end {
                        self.consume_count(end + 1);
                        RepetitionMode::AtLeast(n)
                    } else {
                        let m = self.cursor[comma + 1..end].parse().map_err(|err| RegexError::new(format!("invalid maximum repetition count ({err})"), self.idx, self.idx + end + 1 ))?;
                        self.consume_count(end + 1);
                        RepetitionMode::AtLeastAtMost(n, m)
                    }
                } else {
                    self.consume_count(end + 1);
                   RepetitionMode::Exactly(n)
                }
            },
            _ => unreachable!(),
        };


        let strat = if self.try_consume("+") {
            RepetitionStrategy::Possessive
        } else if self.try_consume("?") {
            RepetitionStrategy::Lazy
        } else {
            RepetitionStrategy::Greedy
        };

        Ok(RegexNode::Repetition(Box::new(inner), Vec::new(), mode, strat))
    }

    pub fn parse_nodes(&mut self, handle_vert_bar: bool) -> Result<RegexNode, RegexError> {
		let dup_capture = self.dup_subpattern_start.take();

		let mut alterations = Vec::with_capacity(1);
		'outer: loop {
			if let Some(dup_capture) = dup_capture {
				self.capture_idx = dup_capture;
			}

			let mut nodes = Vec::with_capacity(1);
			loop {
				nodes.push(self.parse_node()?);

				let ch = self.cursor.chars().next();
                match ch {
                    None => {
                        alterations.push(nodes);
					    break 'outer;
                    },
                    Some('|') => if handle_vert_bar {
						alterations.push(nodes);
						break;
					} else {
						alterations.push(nodes);
						break 'outer;
					},
                    Some(')') => {
                        alterations.push(nodes);
                        break 'outer;
                    },
                    _ => {},
                }
			}

			// consume `|`
            self.consume_count(1);
		}

		if alterations.len() == 1 {
			Ok(RegexNode::Unit(alterations.pop().unwrap()))
		} else {
			Ok(RegexNode::Alternation(alterations))
		}
	}

	fn get_literal(&mut self) -> RegexNode {
		let end = self.cursor.find(|ch| PARSE_SPECIAL_CHARS.contains(&ch)).unwrap_or(self.cursor.len());
		let res = self.cursor[..end].to_string();
        self.consume_count(end);
		RegexNode::Literal(res)
	}

	fn parse_escape_code(&mut self, mode: ParseMode) -> Result<RegexNode, RegexError> {
		let char_to_node = |s: &mut Self, ch: char, num_chars: usize| {
			s.consume_count(num_chars);
			match mode {
				ParseMode::Default => Ok(RegexNode::LiteralChar(ch)),
				ParseMode::ClassDef => Ok(RegexNode::CharacterClassChar(ch)),
			}
		};

		let mut chars = self.cursor.chars();
		let Some(escaped_char) = self.cursor.chars().next() else { return Err(RegexError::new_str("Not enough characters", self.idx, self.idx)) };

		if self.no_special_meaning {
			if escaped_char == 'E' {
				self.no_special_meaning = false;
				return Ok(RegexNode::None)
			} else {
				return char_to_node(self, escaped_char, 1);
			}
		}

		match escaped_char {
			'a'  => char_to_node(self, '\x07', 1),
			'e'  => char_to_node(self, '\x1B', 1),
			'f'  => char_to_node(self, '\x0C', 1),
			'n'  => char_to_node(self, '\n', 1),
			'r'  => char_to_node(self, '\r', 1),
			't'  => char_to_node(self, '\t', 1),
			'*'  => char_to_node(self, '*', 1),
			'\\' => char_to_node(self, '\\', 1),
			'8' => char_to_node(self, '8', 1),
			'9' => char_to_node(self, '9', 1),
			'0' => {
				let end = self.cursor[1..].find(|ch: char| !ch.is_ascii_octdigit()).map_or(2, |val| val + 1);

				match u32::from_str_radix(&self.cursor[2..end], 8) {
					Ok(val) => match char::from_u32(val) {
						Some(ch) => char_to_node(self, ch, end + 2),
						None => Err(RegexError::new_str("Invalid octal value: Value does not represent a valid character", self.idx, self.idx)),
					},
					Err(err) => Err(RegexError::new(format!("Invalid octal value: {err}"), self.idx, self.idx)),	
				}
			},
			ch if ch.is_ascii_octdigit() => {
				let end = self.cursor[1..].find(|ch: char| !ch.is_ascii_octdigit()).map_or(1, |val| val + 1);

				match u32::from_str_radix(&self.cursor[1..end], 8) {
					Ok(val) => match char::from_u32(val) {
						Some(ch) => char_to_node(self, ch, end + 2),
						None => Err(RegexError::new_str("Invalid octal value: Value does not represent a valid character", self.idx, self.idx)),
					},
					Err(err) => Err(RegexError::new(format!("Invalid octal value: {err}"), self.idx, self.idx)),	
				}
			},
			'o' => {
				let Some(open_brace) = chars.next() else { return Err(RegexError::new_str("Not enough characters", self.idx, self.idx)) };
				if open_brace != '{' {
					return Err(RegexError::new_str("Invalid octal value: missing opening brace", self.idx, self.idx))
				}

				let Some(end) = self.cursor.find('}') else { return Err(RegexError::new_str("Invalid octal value: missing closing brace", self.idx, self.idx)) };
				match u32::from_str_radix(&self.cursor[2..end], 8) {
					Ok(val) => match char::from_u32(val) {
						Some(ch) => char_to_node(self, ch, end + 2),
						None => Err(RegexError::new_str("Invalid octal value (\\0{dd..}): Value does not represent a valid character", self.idx, self.idx + end + 2)),
					},
					Err(err) => Err(RegexError::new(format!("Invalid octal value (\\0{{dd..}}): {err}"), self.idx, self.idx + end + 2)),	
				}
			},
			'x' => if &self.cursor[1..2] == "{" {
				let Some(end) = self.cursor.find('}') else { return Err(RegexError::new_str("Invalid hex value: missing closing brace", self.idx, self.idx)) };
				match u32::from_str_radix(&self.cursor[2..end], 16) {
					Ok(val) => match char::from_u32(val) {
						Some(ch) => char_to_node(self, ch, end + 2),
						None => Err(RegexError::new_str("Invalid hex (\\x{hh..}) value: Value does not represent a valid character", self.idx, self.idx + end + 2))
					},
					Err(err) => Err(RegexError::new(format!("Invalid hex (\\x{{hh..}}): {err}"), self.idx, self.idx + end + 2)),
				}
			} else {
				match u32::from_str_radix(&self.cursor[1..3], 16) {
					Ok(val) => match char::from_u32(val) {
						Some(ch) => char_to_node(self, ch, 4),
						None => Err(RegexError::new_str("Invalid hex (\\x{hh..}) value: Value does not represent a valid character", self.idx, self.idx + 3))
					},
					Err(err) => Err(RegexError::new(format!("Invalid hex (\\x{{hh..}}): {err}"), self.idx, self.idx + 3)),
				}
			},
			'u' => match u32::from_str_radix(&self.cursor[1..5], 16) {
				Ok(val) => match char::from_u32(val) {
					Some(ch) => char_to_node(self, ch, 6),
					None => Err(RegexError::new_str("Invalid unicode (\\uhhhh) value: Value does not represent a valid character", self.idx, self.idx + 5))
				},
				Err(err) => Err(RegexError::new(format!("Invalid unicode (\\uhhhh): {err}"), self.idx, self.idx + 5)),
			},
			'd' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Category(unicode::Category::DecimalNumber), true)) },
			'D' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Category(unicode::Category::DecimalNumber), false)) },
			's' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Whitespace, true)) },
			'S' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Whitespace, false)) },
			'h' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::HorizontalWhitespace, true)) },
			'H' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::HorizontalWhitespace, false)) },
			'v' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::VerticalWhitespace, true)) },
			'V' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::VerticalWhitespace, false)) },
			'N' => if mode == ParseMode::ClassDef {
				Err(RegexError::new_str("'\\N' is not allow in a class definition", self.idx, self.idx + 1))
			} else {
				self.consume_count(1);
				Ok(RegexNode::CharacterClass(CharacterClass::NonNewLine, true))
			},
			'R' => if mode == ParseMode::ClassDef {
				char_to_node(self, 'R', 1)
			} else {
				self.consume_count(1);
				Ok(RegexNode::CharacterClass(CharacterClass::AtomicNewLine, true))
			}, // 
			'w' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Word, true)) },
			'W' => { self.consume_count(1); Ok(RegexNode::CharacterClass(CharacterClass::Word, false)) },
			'X' => if mode == ParseMode::ClassDef {
				char_to_node(self, 'X', 1)
			} else {
				self.consume_count(1);
				Ok(RegexNode::CharacterClass(CharacterClass::ExtendedGraphemeCluster, true))
			},
			'p' => {
				self.consume_count(1);
				match self.parse_unicode_property() {
					Ok((prop, expected)) => Ok(RegexNode::CharacterClass(prop, expected)),
					Err(err) => Err(err)
				}
			},
			'P' => {
				let start = self.idx;
				self.consume_count(1);
				match self.parse_unicode_property() {
					Ok((prop, expected)) => if !expected {
						Err(RegexError::new_str("A unicode category cannot contain '^' in the negative '\\P' version", start, self.idx))
					} else {
						Ok(RegexNode::CharacterClass(prop, false))
					},
					Err(err) => Err(err)
				}
			},
			'K' => { self.consume_count(1); Ok(RegexNode::MatchStartReset) }, // Reset match start
			'b' => { // Word boundary
                self.consume_count(1);
                if mode == ParseMode::ClassDef {
                    Ok(RegexNode::CharacterClassChar('\\'))
                } else {
                    Ok(RegexNode::WordBoundary(true))
                }
            },
			'B' => if mode == ParseMode::ClassDef {
				char_to_node(self, 'B', 1)
			} else {
                self.consume_count(1);
				Ok(RegexNode::WordBoundary(false))
			}, // Non-word boundary
			'A' => { self.consume_count(1); Ok(RegexNode::SubjectStart) }, // Subject start
			'Z' => { self.consume_count(1); Ok(RegexNode::SubjectEndOrNewline) }, // Subject end
			'z' => { self.consume_count(1); Ok(RegexNode::SubjectEndOnly) }, // End of subject
			'G' => { self.consume_count(1); Ok(RegexNode::FirstMatchPos) }, // First match in subject
			'Q' => { 
				self.no_special_meaning = true;
				Ok(RegexNode::None)
			 }
			'E' => Ok(RegexNode::None), // End of special meaning, without preceeding \Q is ignored
			'g' => self.parse_backref_g(),
			'k' => self.parse_backref_k(),
			_ => Err(RegexError::new_str("Invalid escape code", self.idx, self.idx))
		}
	}

	fn parse_class_definition(&mut self) -> Result<RegexNode, RegexError> {
		let not_closed_err = |s: &mut Self| Err(RegexError::new_str("Class definition was not closed", s.idx, s.orig.len()));

        // Consume opening bracket `[`
		self.consume_count(1);

		if self.try_consume(":") {
            let expected = self.try_consume("^");

			let Some(end) = self.cursor.find(':') else {
                return Err(RegexError::new_str("Posix character class is not closed correctly", self.idx, self.orig.len()))
            };
			let sub_str = &self.cursor[..end];

			let class = match sub_str {
				"alnum"  => CharacterClass::Category(unicode::Category::Letter | unicode::Category::Number),
				"alpha"  => CharacterClass::Category(unicode::Category::Letter),
				"ascii"  => CharacterClass::PosixAscii,
				"blank"  => CharacterClass::HorizontalWhitespace,
				"cntrl"  => CharacterClass::Category(unicode::Category::Control),
				"digit"  => CharacterClass::Category(unicode::Category::DecimalNumber),
				"graph"  => CharacterClass::PosixGraph,
				"lower"  => CharacterClass::Category(unicode::Category::LowercaseLetter),
				"print"  => CharacterClass::PosixPrint,
				"punct"  => CharacterClass::Category(unicode::Category::Punctuation),
				"space"  => CharacterClass::Whitespace                                ,
				"upper"  => CharacterClass::Category(unicode::Category::UppercaseLetter),
				"word"   => CharacterClass::Category(unicode::Category::Letter | unicode::Category::Number),
				"xdigit" => CharacterClass::PosixXDigit,
				_ => return Err(RegexError::new(format!("Invalid posix character class: '{sub_str}'"), self.idx, self.idx + end))
			};

			self.consume_count(end + 2);
			return Ok(RegexNode::CharacterClass(class, expected));
		}

		let mut chars = self.cursor.chars();

		let mut first = true;
		let Some(mut ch) = chars.next() else { return not_closed_err(self); };

		let mut expected = true;
		let mut class_chars = Vec::new();
		let mut class_ranges = Vec::new();
		let mut classes = Vec::new();
		while ch != ']' {
			if first && ch == '^' {
				self.consume_count(1);
				expected = false;
			} else {
				let start_idx = self.idx;
				let node = if ch == '\\' {
					self.consume_count(1);
					self.parse_escape_code(ParseMode::ClassDef)?
				} else {
					self.consume_count(1);
					RegexNode::CharacterClassChar(ch)
				};

				let mut tmp_chars = self.cursor.chars();
				if tmp_chars.next().map_or(false, |ch| ch == '-') && tmp_chars.next().map_or(true, |ch| ch != ']') {
					let RegexNode::CharacterClassChar(start_ch) = node else {
						return Err(RegexError::new_str("A class definition may not contain a class as part of a range", start_idx, self.idx))
					};

                    // consume '-'
					self.consume_count(1);
					chars = self.cursor.chars();

					let ch = match chars.next() {
						Some(ch) => ch,
						None     => return not_closed_err(self),
					};
					let end_ch = if ch == '\\' {
						let node = self.parse_escape_code(ParseMode::ClassDef)?;
						let RegexNode::CharacterClassChar(ch) = node else {
							return Err(RegexError::new_str("A class definition may not contain a class as part of a range", start_idx, self.idx))
						};
						ch
					} else {
						self.consume_count(1);
						ch
					};

					if start_ch > end_ch {
						return Err(RegexError::new(format!("The start of the character class range comes after the end"), start_idx, self.idx));
					}
					
					class_ranges.push((start_ch, end_ch));
				} else {
					match node {
						RegexNode::CharacterClassChar(ch)    => class_chars.push(ch),
						node @ RegexNode::CharacterClass(..) => classes.push(node),
						_ => return Err(RegexError::new_str("Encountered invalid node in class definition", start_idx, self.idx)),
					}
				}
			}
			ch = match chars.next() {
				Some(ch) => ch,
				None     => return not_closed_err(self),
			};
			first = false;
		}
		self.consume("]", "Class definition was not closed")?;

		Ok(RegexNode::ClassDef(class_chars, class_ranges, classes, expected))
	}

    fn parse_group(&mut self) -> Result<RegexNode, RegexError> {
		let not_closed_error = |s: &Self| Err(RegexError::new_str("Group was not closed", s.idx, s.orig.len()));

		let mut flag_change = RegexFlagChange::None;

        // consume opening paren '('
        self.consume_count(1);

		let mut chars = self.cursor.chars();
		let Some(ch) = chars.next() else { return not_closed_error(self) };

		let handle_name = |s: &mut Self, prefix: bool, end_ch: char, capture_idx: u16| {
			s.consume_count(1 + prefix as usize);
			let Some(end) = s.cursor.find(end_ch) else {
                return Err(RegexError::new(format!("Group capture name was not closed, expected '{end_ch}'"), s.idx - 1, s.orig.len()))
            };

			let capture = &&s.cursor[..end];
			s.check_capture_name(capture)?;

			s.add_capture_name(capture, capture_idx)?;
			s.consume_count(end + 1);
			Ok(())
		};

		let mut capture_idx = None;
		let mut atomic = false;
		if ch == '?' {
            self.consume_count(1);

			let mut flag_negate = false;
			while let Some(c) = chars.next() {
				match c {
					'i' => {
						self.consume_count(1);
						flag_change |= if flag_negate { RegexFlagChange::CaselessOn  } else { RegexFlagChange::CaselessOff } 
				 	},
					'm' => {
						self.consume_count(1);
						flag_change |= if flag_negate { RegexFlagChange::MultilineOn } else { RegexFlagChange::MultilineOff } 
					},
					's' => {
						self.consume_count(1);
						flag_change |= if flag_negate { RegexFlagChange::DotAllOn    } else { RegexFlagChange::DotAllOff } 
					},
					'x' => {
						self.consume_count(1);
						flag_change |= if flag_negate { RegexFlagChange::ExtendedOn  } else { RegexFlagChange::ExtendedOff } 
					},
					'-' => {
						self.consume_count(1);
						flag_negate = true 
					},
					':' => {
						self.consume_count(1);
						break;
					}
					'|' => {
						self.consume_count(1);
						self.begin_capture_alteration();
						capture_idx = Some(self.gen_capture()?);
						break;
					}
					'<' => {
						let Some(ch) = chars.next() else { return not_closed_error(self) };
						match ch {
							'=' | '!' => {
								self.consume_count(1);
								return self.parse_lookahead_lookbehind(true);
							},
							_ => {
								let capt_idx = self.gen_capture()?;
								handle_name(self, false, '>', capt_idx)?;
								capture_idx = Some(capt_idx);
								break;
							}
						}
					}
					'\'' => {
						let capt_idx = self.gen_capture()?;
						handle_name(self, false, '\'', capt_idx)?;
						capture_idx = Some(capt_idx);
						break;
					}
					'P' => {
						if self.cursor.starts_with("P=") {
							let Some(end) = self.cursor.find(')') else {
                                return Err(RegexError::new_str("Named back reference was not closed", self.idx, self.orig.len()));
                            };
							let capture = &self.cursor[2..end];
							self.check_capture_name(capture)?;
							self.consume_count(end + 1);
							return Ok(RegexNode::NamedBackRef(capture.to_string()));
						} else {	
							let capt_idx = self.gen_capture()?;
							handle_name(self, true, '>', capt_idx)?;
							capture_idx = Some(capt_idx);
							break;
						}
					}
					'>' => {
						atomic = true;
						self.consume_count(1);
						break;
					}
					'=' | '!' => return self.parse_lookahead_lookbehind(false),
					'(' => return self.parse_conditional(),
					_ => {
						return Err(RegexError::new(format!("Unexpected character in subpattern prefix: '{c}'"), self.idx, self.idx + 1));
					}
				}
			}
		} else {
			capture_idx = Some(self.gen_capture()?);
		}

		if self.try_consume(")") { 
			return Ok(RegexNode::InternalOptionSetting(flag_change));
		}

		let inner = self.parse_nodes(true)?;

        self.consume(")", "Group was not closed")?;
		self.end_capture_alteration();

		Ok(RegexNode::ParsedGroup(flag_change, capture_idx, Box::new(inner), atomic))
	}

    fn parse_conditional(&mut self) -> Result<RegexNode,RegexError> {
        let start_idx = self.idx - 2;

        self.consume_count(1);
        let Some(end) = self.cursor.find(')') else { return Err(RegexError::new_str("conditional subpattern was not closed", self.idx, self.orig.len())); };
        let inner = &self.cursor[..end];

        let parse_yes_no_and_close = |s: &mut Self| {
            let yes = Box::new(s.parse_nodes(false)?);

            let Some(ch) = s.cursor.chars().next() else { return Ok((yes, None)) };
            let no = if ch == '|' {
                s.consume_count(1);
                Some(Box::new(s.parse_nodes(false)?))
            } else {
                None
            };

            s.consume(")", "Conditional is not closed")?;
            Ok((yes, no))
        };

        self.consume_count(end + 1);

        if inner == "DEFINE" { 
            let (yes, no) = parse_yes_no_and_close(self)?;
            if no.is_some() {
                return Err(RegexError::new_str("A define conditional node cannot contain a no-patter", start_idx, self.idx));
            }
            return Ok(RegexNode::DefineConditional(yes));
        } else {
            let Some(ch) = inner.chars().next() else { return Err(RegexError::new_str("A conditional subpattern's conditon cannot be empty", self.idx, self.idx + inner.len())); };

            match ch {
                '<' | '\'' => {
                    if !inner.ends_with(if ch == '<' { '>' } else { '\'' }) {
                        return Err(RegexError::new_str("Named conditional name is not closed", self.idx, self.idx + inner.len()))
                    }
                    if inner.len() <= 2 {
                        return Err(RegexError::new_str("A named conditional's name cannot be empty", self.idx, self.idx + inner.len()))
                    }

                    let name = inner[1..inner.len() - 1].to_string();
                    let (yes, no) = parse_yes_no_and_close(self)?;
                    return Ok(RegexNode::NamedConditional(name, yes, no));
                },
                'R' => {
                    return Err(RegexError::new_str("Recursions is currently not supported", self.idx, self.orig.len()))
                    // let inner = &inner[1..];
                    // if inner.starts_with('&') {
                    //     let (yes, no) = parse_yes_no_and_close(self)?;
                    //     return Ok(RegexNode::NamedRecursiveConditional(inner[1..].to_string(), yes, no));
                    // } else {
                    //     let count = match inner.parse::<u16>() {
                    //         Ok(count) => count,
                    //         Err(err) => return Err(RegexError::new(format!("Failed to parse conditional recursion count: {err}"), self.idx, self.idx + inner.len())),
                    //     };
                    //     let (yes, no) = parse_yes_no_and_close(self)?;
                    //     return Ok(RegexNode::RecursiveConditional(count, yes, no));
                    // }
                },
                '+' => {
                    let count = match inner[1..].parse::<u16>() {
                        Ok(count) => count,
                        Err(err) => return Err(RegexError::new(format!("Failed to parse conditional relative count: {err}"), self.idx, self.idx + inner.len())),
                    };
                    let (yes, no) = parse_yes_no_and_close(self)?;

                    let (capture_idx, overflow) = self.capture_idx.overflowing_add(count);
                    if overflow {
                        return Err(RegexError::new(format!("Relative conditional index out of bounds"), start_idx, self.idx));
                    }
                    return Ok(RegexNode::AbsConditional(capture_idx, yes, no));
                },
                '-' => {
                    let count = match inner[1..].parse::<u16>() {
                        Ok(count) => count,
                        Err(err) => return Err(RegexError::new(format!("Failed to parse conditional relative count: {err}"), self.idx, self.idx + inner.len())),
                    };
                    let (yes, no) = parse_yes_no_and_close(self)?;

                    let (capture_idx, overflow) = self.capture_idx.overflowing_sub(count);
                    if overflow {
                        return Err(RegexError::new(format!("Relative conditional index out of bounds"), start_idx, self.idx));
                    }
                    return Ok(RegexNode::AbsConditional(capture_idx + 1, yes, no));
                },
                '?' => {
                    let is_lookbehind = inner[1..].starts_with('<');
                    self.set_idx(start_idx + 5);
                    let cond = Box::new(Self::parse_lookahead_lookbehind(self, is_lookbehind)?);

                    let (yes, no) = parse_yes_no_and_close(self)?;
                    return Ok(RegexNode::AssertConditional(cond, yes, no))
                },
                _ => {
                    let capture_idx = match inner.parse::<u16>() {
                        Ok(idx) => idx,
                        Err(err) => return Err(RegexError::new(format!("Failed to parse conditional absolute count: {err}"), self.idx, self.idx + inner.len())),
                    };
                    let (yes, no) = parse_yes_no_and_close(self)?;
                    return Ok(RegexNode::AbsConditional(capture_idx, yes, no));
                }
            }
        }
    }

	fn parse_lookahead_lookbehind(&mut self, is_lookbehind: bool) -> Result<RegexNode, RegexError> {
		let expected = self.cursor.starts_with('=');
		self.consume_count(1);

		if is_lookbehind {
			let first_node = self.parse_nodes(false)?;
			let mut nodes = vec![first_node];
			while self.try_consume("|") {
				nodes.push(self.parse_nodes(false)?);
			}

			self.consume(")", "Lookbehind was not closed")?;
			Ok(RegexNode::Lookbehind(nodes, Vec::new(), expected))
		} else {
            let inner = self.parse_nodes(true)?;
			self.consume(")", "Lookahead was not closed")?;
			Ok(RegexNode::Lookahead(Box::new(inner), expected))
		}
	}

    fn parse_unicode_property(&mut self) -> Result<(CharacterClass, bool), RegexError> {
        let mut chars = self.cursor.chars();

        let Some(ch) = chars.next() else { return Err(RegexError::new_str("No character class given", self.idx, self.idx)) };
        let expected = chars.next().map_or(false, |ch| ch == '^');

        if ch == '{' {
            let start = !expected as usize + 1;
            let Some(end) = self.cursor.find('}') else { return Err(RegexError::new_str("unicode property was not closed", self.idx, self.orig.len())) };
			let prop_str = &self.cursor[start..end];
            
			// Special case for regex only
			if prop_str == "L&" {
				self.consume_count(end + 1);
				return Ok((CharacterClass::Category(unicode::Category::CasedLetter), expected));
			}
			if prop_str == "Any" {
				self.consume_count(end + 1);
				return Ok((CharacterClass::Any, expected));
			}

			// Generic unicode category class
			if let Some(cat) = unicode::Category::parse(prop_str) {
				self.consume_count(end + 1);
				return Ok((CharacterClass::Category(cat), expected));
			}

			// Not a category, so first try to check for special 'X..' props
			if chars.next() == Some('X') {
				let Some(ch0) = chars.next() else { return Err(RegexError::new_str("malformed character property", self.idx, self.idx + end + 1)) };
				let Some(ch1) = chars.next() else { return Err(RegexError::new_str("malformed character property", self.idx, self.idx + end + 1)) };
				match (ch0, ch1) {
					('w', 'd') | // perl word
					('a', 'n') => {
						self.consume_count(end + 1);	
						return Ok((CharacterClass::Category(unicode::Category::Letter | unicode::Category::Number), expected))
					},
					('s', 'p') | // perl space
					('p', 's') => {
						self.consume_count(end + 1);	
						return Ok((CharacterClass::PosixSpace, expected))
					},
					('u', 'c') => {
						self.consume_count(end + 1);	
						return Ok((CharacterClass::UNC, expected))
					},
					_ => ()
				}
			}

            // Then just treat it as a unicode property, either a long or short version works
			match unicode::Script::parse(prop_str).or_else(|| unicode::Script::from_short_name(prop_str)) {
				Some(script) => {
					self.consume_count(end + 1);
					Ok((CharacterClass::Script(script), expected))
				},
				None => Err(RegexError::new(format!("Invalid character property '{prop_str}'"), self.idx, self.idx + end + 1)),
			}
        } else {
			let cat = if self.cursor.len() >= 2 { 
                if !self.cursor.chars().take(2).all(|ch| ch.is_ascii()) {
                    return Err(RegexError::new_str("Malformed unicode property, only expected unicode characters", self.idx, self.idx))
                }

				match unicode::Category::parse(&self.cursor[..2]) {
					Some(cat) => {
						self.consume_count(2);
						Some(cat)
					},
					None => match unicode::Category::parse(&self.cursor[..1]) {
						Some(cat) => {
							self.consume_count(1);
							Some(cat)
						},
						None => None,
					}
				}
			} else if self.cursor.len() == 1 {
                if !self.cursor.chars().take(1).all(|ch| ch.is_ascii()) {
                    return Err(RegexError::new_str("Malformed unicode property, only expected unicode characters", self.idx, self.idx))
                }

				let tmp = unicode::Category::parse(&self.cursor[..1]);
				self.consume_count(1);
				tmp
			} else {
                None
            };

			match cat {
				Some(cat) => Ok((CharacterClass::Category(cat), expected)),
				None      => Err(RegexError::new(format!("Invalid character property '{}'", self.cursor), self.idx, self.orig.len())),
			}
		}
    }

	fn parse_backref_g(&mut self) -> Result<RegexNode, RegexError> {
        // Consume 'g' character
		self.consume_count(1);

		let Some(ch) = self.cursor.chars().next() else {
            return Err(RegexError::new_str("Back reference does not contain an index or name", self.idx, self.idx))
        };

		match ch {
			'{' => {
				let Some(end) = self.cursor.find('}') else { return Err(RegexError::new_str("Backrefence is not closed", self.idx, self.orig.len())) };
				let inner = &self.cursor[1..end];

				let Some(ch) = inner.chars().next() else { return Err(RegexError::new_str("Empty braced back reference is empty", self.idx, self.idx)) }; 
				match ch {
					'-' => {
						let val = inner[1..].parse::<u16>().map_err(|err| RegexError::new(format!("Invalid back reference value: {err}"), self.idx + 3, self.idx + end))?;
                        
						// The parser knows which index we are at, as it generates them, so already calculate the absolute index here
                        // If we would support pre-parsed regex snippets, this needs to be resolved in a post processing pass
						if val > self.capture_idx {
                            Err(RegexError::new(format!("Relative back reference would be out of range: {val}, index: {}", self.capture_idx), self.idx, self.idx + end))
						} else {
                            self.consume_count(end + 1);
							Ok(RegexNode::AbsBackRef(self.capture_idx + 1 - val))
						}
					},
					ch if ch.is_numeric() => {
						let val = inner.parse::<u16>().map_err(|err| RegexError::new(format!("Invalid back reference value: {err}"), self.idx + 3, self.idx + end))?;
						self.consume_count(end + 1);
						Ok(RegexNode::AbsBackRef(val))
					},
					ch if ch.is_alphabetic() => {
						self.check_capture_name(inner)?;
						self.consume_count(end + 1);
						Ok(RegexNode::NamedBackRef(inner.to_string()))
					},
					_ => Err(RegexError::new_str("Invalid backreference value", self.idx + 2, self.idx + end)),
				}
			},
			'-' => {
                self.consume_count(1);
				let end = self.cursor.find(|ch: char| !ch.is_numeric()).map_or(self.cursor.len(), |val| val + 1);
				let val = self.cursor[..end].parse::<u16>().map_err(|err| RegexError::new(format!("Invalid back reference value: {err}"), self.idx + 3, self.idx + end))?;
				self.consume_count(end);

				// The parser knows which index we are at, as it generates them, so already calculate the absolute index here
                // If we would support pre-parsed regex snippets, this needs to be resolved in a post processing pass
				if val > self.capture_idx {
					Err(RegexError::new(format!("Relative back reference would be out of range: {val}, index: {}", self.capture_idx), self.idx, self.idx + end))
				} else {
					Ok(RegexNode::AbsBackRef(self.capture_idx + 1 - val))
				}
			},
			ch if ch.is_numeric() => {
				let end = self.cursor.find(|ch: char| !ch.is_numeric()).unwrap_or(self.cursor.len());
				let val = self.cursor[..end].parse::<u16>().map_err(|err| RegexError::new(format!("Invalid back reference value: {err}"), self.idx + 3, self.idx + end))?;
				self.consume_count(end);
				Ok(RegexNode::AbsBackRef(val))
			},
			_ => Err(RegexError::new_str("Invalid back reference", self.idx - 2, self.idx)),
		}
	}

    
	fn parse_backref_k(&mut self) -> Result<RegexNode, RegexError> {
        // Consume 'k' character
		self.consume_count(1);
		let Some(ch) = self.cursor.chars().next() else {
            return Err(RegexError::new_str("Back reference does not contain an index or name", self.idx, self.idx))
        };

		let end_ch = match ch {
			'<'  => '>',
			'{'  => '}',
			'\'' => '\'',
			_    => return Err(RegexError::new_str("Invalid back reference", self.idx, self.idx)),
		};

		let Some(end) = self.cursor[1..].find(end_ch) else { return Err(RegexError::new_str("Backrefence is not closed", self.idx, self.orig.len())) };
		let inner = &self.cursor[1..end + 1];

		self.check_capture_name(inner)?;
		self.consume_count(end + 2);
		Ok(RegexNode::NamedBackRef(inner.to_string()))
	}

    
	fn check_capture_name(&mut self, capture: &str) -> Result<(), RegexError> {
		if capture.len() > 32 {
			return Err(RegexError::new(format!("A capture name cannot be longer than 32 characters: '{capture}'"), self.idx, self.idx + capture.len()))
		}

		if capture.starts_with(|ch: char| ch.is_numeric()) {
			return Err(RegexError::new(format!("A capture name cannot start with a number: '{capture}'"), self.idx, self.idx + capture.len()))
		}
		if capture.chars().any(|ch| !(ch.is_alphanumeric() || ch == '_')) {
			return Err(RegexError::new(format!("A capture name can only contain alphanumeric characters or '_': '{capture}'"), self.idx, self.idx + capture.len()))
		}
		Ok(())
	}

}