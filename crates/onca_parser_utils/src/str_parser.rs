use core::str::pattern::{Pattern, Searcher};

/// Parser error
pub struct ParserError {
    line   : usize,
    column : usize,
    msg    : &'static str,
}

/// Parser that can parse a `&str`
pub struct StrParser<'a> {
    pub line   : usize,
    pub column : usize,
    pub string : &'a str
}

impl<'a> StrParser<'a> {
    /// Create a new parser
    pub fn new(string: &'a str) -> Self {
        Self { line: 0, column: 0, string }
    }

    /// Try to consume a given character
    pub fn consume_char(&mut self, ch: char) -> bool {
        if self.string.starts_with(ch) {
            self.string = &self.string[1..];
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
            true
        } else {
            false
        }
    }

    /// Try ot consume a given string
    pub fn consume_str(&mut self, s: &str) -> bool {
        if self.string.starts_with(s) {
            self.consume_count(s.len());
            true
        } else {
            false
        }
    }

    /// Consume `count` characters
    pub fn consume_count(&mut self, count: usize) {
        let s = &self.string[..count];
        let num_lines = s.matches('\n').count();
        let num_colums = s.rfind('\n').map_or(0, |idx| count - idx);
        self.line += num_colums;
        self.column = if num_lines > 0 {
            num_colums
        } else {
            self.column + num_colums
        };
        self.string = &self.string[count..];
    }

    /// Skip to the next end-of-line
    pub fn consume_to_eol(&mut self) {
        let idx = self.string.find('\n').unwrap_or(self.string.len());
        self.string = &self.string[idx..];
        self.line += 1;
        self.column = 0;
    }

    pub fn consume_whitespace(&mut self, include_newline: bool) {
        let idx = self.string.find(|ch: char| !ch.is_whitespace() || (include_newline && ch != '\n')).unwrap_or(self.string.len());
        self.consume_count(idx);
    }
    
    /// Move the parser to the end (finish parsing)
    pub fn end(&mut self) {
        self.string = &self.string[self.string.len()..];
    }

    /// Check if there is still data to parse
    pub fn can_parse(&self) -> bool {
        self.string.len() != 0
    }

    /// Create an error at the current line and column
    pub fn error(&self, msg: &'static str) -> ParserError {
        ParserError { line: self.line, column: self.column, msg }
    }

    /// Find the first occurance of a non-escaped delimiter
    /// 
    /// The result contains a tuple, with the index of the match, and the index after the end of the match
    pub fn find_non_escaped_delimiter<'b, P: Pattern<'b> + Copy>(string: &'b str, delimiter: P) -> Option<(usize, usize)> {
        let mut searcher = delimiter.into_searcher(string);
		loop {
            match searcher.next_match() {
                Some(tup) => {
                    if tup.0 == 0 || string.as_bytes()[tup.0 - 1] != '\\' as u8 {
						return Some(tup);
					}
                },
                None => return None,
            }
		}
	}
     
    /// Extract a substring which is between a starting and ending patters, and may optionally span multiple lines
    /// 
    /// `len_end` is the length of the end delimiter, as we can't infer this from pattern
    pub fn extract_string<P0: Pattern<'a> + Copy, P1: Pattern<'a> + Copy>(&mut self, start_delimiter: P0, end_delimiter: P1, multi_line: bool) -> Option<&str> {
        let start = match StrParser::find_non_escaped_delimiter(self.string, start_delimiter) {
			Some(start) => (start.0, start.1),
			None => return None,
		};

		// Find the index of the last unescaped quote
		let end = match StrParser::find_non_escaped_delimiter(&self.string[start.1..], end_delimiter) {
			Some(end) => (end.0 + start.1, end.1 + start.1),
			None => return None,
		};
		
		if !multi_line && let Some(eol) = self.string.find("\n") && end.0 > eol {
			None
		} else {
			let res = &self.string[start.1..end.0];
			self.consume_count(end.1);
			Some(res)
		}
	}

    /// Extract until a certain patterns is reached
    pub fn extract_until<P: Pattern<'a>>(&mut self, pattern: P) -> &str {
        let idx = self.string.find(pattern).unwrap_or(self.string.len());
        let res = &self.string[..idx];
        self.consume_count(idx);
        res
    }
}