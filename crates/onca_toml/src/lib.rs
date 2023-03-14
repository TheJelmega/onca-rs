#![feature(let_chains)]

use onca_core::{
	prelude::*,
	collections::HashMap,
};
use onca_parser_utils::str_parser::*;

pub struct TomlParseError(pub ParserError);

pub enum Item {
	Comment(String),
	String(String),
	Integer(i64),
	Float(f64),
	Boolean(bool),
	Array(DynArray<Item>),
	Table(Table),
}

/// Toml table that preserves comments
pub struct Table {
	/// Actual items (including comments)
	items   : DynArray<Item>,
	/// Mapping from key to an index
	mapping : HashMap<String, usize>,
}

impl Table {
	pub fn new() -> Self {
		Self { items: DynArray::new(), mapping: HashMap::new() }
	}

	/// Append an item to the toml
	pub fn push(&mut self, key: String, item: Item) -> bool {
		match self.mapping.get(&key) {
			Some(_) => false,
			None => {
				let idx = self.items.len();
				self.items.push(item);
				self.mapping.insert(key, idx);
				true
			}
		}
	}
	
	/// Append an item with multiple keys to the toml
	/// 
	/// # Error
	/// 
	/// If an item could not be added, as one of the sub-keys points to a non-table, an error with the index of the key the push failed at is returned
	pub fn push_multi_key(&mut self, keys: &[String], item: Item) -> Result<(), usize> {
		if keys.len() == 1 {
			if self.push(keys[0].clone(), item) {
				Ok(())
			} else {
				Err(0)
			}
			
		} else {
			match self.get_mut(&keys[0]) {
				Some(item) => {
					match item {
						Item::Table(table) => table,
						// Non table item, so we can't add this
						_ => return Err(0),
					}
				},
				None => {
					self.push(keys[0].clone(), Item::Table(Table::new()));
					match self.items.last_mut().unwrap() {
						Item::Table(table) => table,
						_ => unreachable!("Last item should always be a table here")
					}
				}
			}.push_multi_key(&keys[1..], item).map_err(|idx| idx + 1)
		}
	}

	/// Push a comment into the table
	pub fn push_comment(&mut self, comment: String) {
		self.items.push(Item::Comment(comment))
	}

	/// Get an element from the toml
	pub fn get(&self, key: &str) -> Option<&Item> {
		self.mapping.get(&key.to_onca_string()).map(|idx| &self.items[*idx])
	}

	/// Get a mutable element from the toml
	pub fn get_mut(&mut self, key: &str) -> Option<&mut Item> {
		self.mapping.get(&key.to_onca_string()).map(|idx| &mut self.items[*idx])
	}

	fn get_or_add_table(&mut self, keys: &[String]) -> Result<&mut Table, i32> {
		if keys.len() == 0 {
			Ok(self)
		} else {
			self.get_or_add_single_table(&keys[0])?.get_or_add_table(&keys[1..])
		}
	}

	fn add_array_table(&mut self, keys: &[String]) -> Result<&mut Table, i32> {
		if keys.len() == 1 {
			let arr = match self.mapping.get(&keys[0]) {
				Some(idx) => {
					if let Item::Array(arr) = &mut self.items[*idx] {
						arr
					} else {
						return Err(0)
					}
				},
				None => {
					self.push(keys[0].clone(), Item::Array(DynArray::new()));
					match self.items.last_mut().unwrap() {
						Item::Array(arr) => arr,
						_ => unreachable!("Last item should always be an array here")
					}
				}
			};
			arr.push(Item::Table(Table::new()));
			// SAFETY: We added a table, so the last element is always valid and a table
			match arr.last_mut().unwrap() {
				Item::Table(table) => Ok(table),
				_ => unreachable!("Last item should always be a table here")
			}
		}
		else
		{
			self.get_or_add_single_table(&keys[0])?.add_array_table(&keys[1..])
		}
	}

	fn get_or_add_single_table(&mut self, key: &String) -> Result<&mut Table, i32> {
		match self.mapping.get(&key) {
			Some(idx) => {
				match &mut self.items[*idx] {
					Item::Table(table) => Ok(table),
					// Non table item, so we can't add this
					_ => return Err(0),
				}
			},
			None => {
				self.push(key.clone(), Item::Table(Table::new()));
				match self.items.last_mut().unwrap() {
					Item::Table(table) => Ok(table),
					_ => unreachable!("Last item should always be a table here")
				}
			}
		}
	}
}

pub struct Toml {
	table : Table,
}

// TODO: Read from stream
impl Toml {
	/// Create a new toml
	pub fn new() -> Self {
		Self { table: Table::new() }
	}

	/// Parse toml from a string
	pub fn parse(source: &str) -> Result<Self, TomlParseError> {
		let mut parser = Parser::new(source);
		parser.parse()
	}

	/// Append an item to the toml
	pub fn push(&mut self, key: String, item: Item) -> bool {
		self.table.push(key, item)
	}
	
	/// Append an item with multiple keys to the toml
	/// 
	/// # Error
	/// 
	/// If an item could not be added, as one of the sub-keys points to a non-table, an error with the index of the key the push failed at is returned
	pub fn push_multi_key(&mut self, keys: &[String], item: Item) -> Result<(), usize> {
		self.table.push_multi_key(keys, item)
	}

	/// Push a comment into the toml
	pub fn push_comment(&mut self, comment: String) {
		self.table.push_comment(comment)
	}

	/// Get an element from the toml
	pub fn get(&self, key: &str) -> Option<&Item> {
		self.table.get(key)
	}
	
	/// Get a mutable element from the toml
	pub fn get_mut(&mut self, key: &str) -> Option<&mut Item> {
		self.table.get_mut(key)
	}
}


struct Parser<'a> {
	pub parser : StrParser<'a>,
}

impl<'a> Parser<'a> {
	fn new(source: &'a str) -> Self {
		Self { parser: StrParser::new(source) }
	}

	fn parse(&mut self) -> Result<Toml, TomlParseError> {
		let mut toml = Toml::new();
		let mut table = &mut toml.table;

		// Consume all whitespace so we have something to parse
		self.parser.consume_whitespace(true);

		while self.parser.can_parse() {
			// Now we should either have a comment, a table, or a key-item pair
			if self.parser.string.starts_with('#') {
				let comment = self.parse_comment();
				table.push_comment(comment);
			} else if self.parser.string.starts_with("[[") {
				_ = self.parser.consume_str("[[");
				let keys = self.parse_keys()?;
				table = match toml.table.add_array_table(&keys) {
    			    Ok(arr) => arr,
    			    Err(_) => return Err(self.error_and_skip_to_eol("Path does not point to a table")),
    			};
				if !self.parser.consume_str("]]") {
					return Err(self.error_and_skip_to_eol("Table is not closed"))
				}
			} else if self.parser.string.starts_with('[') {
				_ = self.parser.consume_char('[');
				let keys = self.parse_keys()?;
				table = match toml.table.get_or_add_table(&keys) {
    			    Ok(table) => table,
    			    Err(_) => return Err(self.error_and_skip_to_eol("Path does not point to a table")),
    			};
				if !self.parser.consume_char(']') {
					return Err(self.error_and_skip_to_eol("Table is not closed"))
				}
			} else {
				let (keys, item) = self.parse_key_item()?;
				_ = table.push_multi_key(&keys, item);
			}

			// Consume all whitespace for the next iteration
			self.parser.consume_whitespace(true);
		}
		Ok(toml)
	}

	fn parse_key_item(&mut self) -> Result<(DynArray<String>, Item), TomlParseError> {
		let keys = self.parse_keys()?;
		self.parser.consume_whitespace(false);
		if !self.parser.consume_char('=') {
			return Err(self.error_and_skip_to_eol("Key is not followed by an `=`"))
		}
		self.parser.consume_whitespace(false);
		let item = self.parse_item()?;
		Ok((keys, item))
	}

	fn parse_keys(&mut self) -> Result<DynArray<String>, TomlParseError> {	
		let mut arr = DynArray::new();
		loop {
			let key = if self.parser.string.starts_with('"') {
				match self.parser.extract_string('"', '"', false) {
					Some(s) => s.to_onca_string(),
					None => return Err(self.error_and_skip_to_eol("Invalid key")),
				}
			} else {
				let end = self.parser.string.find(|ch: char| !ch.is_alphanumeric() && ch != '-' && ch != '_').unwrap_or(self.parser.string.len());
				let key = &self.parser.string[..end];
				self.parser.consume_count(key.len());
				key.to_onca_string()
			};
			arr.push(key);

			self.parser.consume_whitespace(false);
			if !self.parser.consume_char('.') {
				return Ok(arr);
			}
		}
	}

	fn parse_item(&mut self) -> Result<Item, TomlParseError> {
		if !self.parser.can_parse() {
			return Err(self.error_and_terminate("End of file"));
		}

		// SAFETY: We only can reach here if there is still data to parse, so there is at least 1 character
		match self.parser.string.chars().nth(0).unwrap() {
			// TOML basic strings
			'"' => {
				let long_delim = "\"\"\"";
				if self.parser.string.starts_with(long_delim) {
					match self.parser.extract_string(long_delim, long_delim, true) {
						Some(string) => Ok(Item::String(string.to_onca_string())),
						None => Err(self.error_and_skip_to_eol("Invalid string"))
					}
				} else {
					match self.parser.extract_string('"', '"', false) {
						Some(string) => Ok(Item::String(string.to_onca_string())),
						None => Err(self.error_and_skip_to_eol("Invalid string"))
					}
				}
			},
			// TOML literal string
			'\'' => {
				let long_delim = "'''";
				if self.parser.string.starts_with(long_delim) {
					match self.parser.extract_string(long_delim, long_delim, true) {
						Some(string) => Ok(Item::String(string.escape_default().collect())),
						None => Err(self.error_and_skip_to_eol("Invalid string"))
					}
				} else {
					match self.parser.extract_string('\'', '\'', false) {
						Some(string) => Ok(Item::String(string.escape_default().collect())),
						None => Err(self.error_and_skip_to_eol("Invalid string"))
					}
				}
			},
			'[' => self.parse_array(),
			'{' => self.parse_inline_table(),
			// Numbers
			ch if ch.is_numeric() || ch == '-' || ch == '+' => {
				let s = self.parser.extract_until(|ch: char| ch.is_whitespace() || ch == '\n' || ch == ',');
				// remove `_`
				let mut s = s.to_onca_string();
				s.retain(|ch| ch != '_');

				if s.contains("inf") {
					if s.starts_with('-') {
						Ok(Item::Float(-f64::INFINITY))
					} else {
						Ok(Item::Float(f64::INFINITY))
					}
				} else if s.contains("nan") {
					if s.starts_with('-') {
						Ok(Item::Float(-f64::NAN))
					} else {
						Ok(Item::Float(f64::NAN))
					}
				}else if let Some(s) = s.strip_prefix("0x") {
					match i64::from_str_radix(s, 16) {
						Ok(val) => Ok(Item::Integer(val)),
						Err(_) => Err(self.error_and_skip_to_eol("Invalid hexadecimal literal"))
					}
				} else if let Some(s) = s.strip_prefix("0o") {
					match i64::from_str_radix(s, 8) {
						Ok(val) => Ok(Item::Integer(val)),
						Err(_) => Err(self.error_and_skip_to_eol("Invalid octal literal"))
					}
				} else if let Some(s) = s.strip_prefix("0b") {
					match i64::from_str_radix(s, 2) {
						Ok(val) => Ok(Item::Integer(val)),
						Err(_) => Err(self.error_and_skip_to_eol("Invalid binary literal"))
					}
				} else if s.contains(['.', 'e', 'E']) {
					match s.parse::<f64>() {
						Ok(fp) => Ok(Item::Float(fp)),
						Err(_) => Err(self.error_and_skip_to_eol("Invalid float literal"))
					}
				} else {
					match s.parse::<i64>() {
						Ok(val) => Ok(Item::Integer(val)),
						Err(_) => Err(self.error_and_skip_to_eol("Invalid integer literal"))
					}
				}
			},
			_ => Err(self.error_and_skip_to_eol("Invalid item")),
		}
	}

	fn parse_array(&mut self) -> Result<Item, TomlParseError> {
		let valid = self.parser.consume_char('[');
		debug_assert!(valid);

		self.parser.consume_whitespace(false);
		
		let mut arr = DynArray::new();
		arr.push(self.parse_item()?);
		self.parser.consume_whitespace(false);
		
		while self.parser.consume_char(',') {
			self.parser.consume_whitespace(false);
			arr.push(self.parse_item()?);
			self.parser.consume_whitespace(false);
		};
		
		self.parser.consume_whitespace(false);
		if self.parser.consume_char(']') {
			Ok(Item::Array(arr))
		} else {
			Err(self.error_and_skip_to_eol("Array was not ended correctly"))
		}
	}

	fn parse_inline_table(&mut self) -> Result<Item, TomlParseError> {
		let valid = self.parser.consume_char('{');
		debug_assert!(valid);

		self.parser.consume_whitespace(false);
		
		let mut table = Table::new();
		let (keys, item) = self.parse_key_item()?;
		table.push_multi_key(&keys, item).map_err(|_| self.error_and_skip_to_eol("Duplicate key"))?;
		
		while self.parser.consume_char(',') {
			self.parser.consume_whitespace(false);
			let (keys, item) = self.parse_key_item()?;
			table.push_multi_key(&keys, item).map_err(|_| self.error_and_skip_to_eol("Duplicate key"))?;
			self.parser.consume_whitespace(false);
		};
		
		self.parser.consume_whitespace(false);
		if self.parser.consume_char('}') {
			Ok(Item::Table(table))
		} else {
			Err(self.error_and_skip_to_eol("Array was not ended correctly"))
		}
	}

	fn parse_comment(&mut self) -> String {
		// Consume '#'
		let valid_comment = self.parser.consume_char('#');
		debug_assert!(valid_comment);

		match self.parser.string.find('\n') {
		    Some(eol_idx) => {
				// SAFETY: unwrap() will always work, as even in an empty comment, this would point to the '#' character
				let str_end = if self.parser.string.bytes().nth(eol_idx - 1).unwrap() == '\r' as u8 {
					eol_idx - 1
				} else {
					eol_idx
				};

				let comment = self.parser.string[..str_end].to_onca_string();
				self.parser.consume_to_eol();
				comment
			},
		    None => {
				let comment = self.parser.string.to_onca_string();
				self.parser.end();
				comment
			},
		}
	}

	fn error_and_skip_to_eol(&mut self, msg: &'static str) -> TomlParseError {
		let err = TomlParseError(self.parser.error(msg));
		self.parser.consume_to_eol();
		err
	}

	fn error_and_terminate(&mut self, msg: &'static str) -> TomlParseError {
		let err = TomlParseError(self.parser.error(msg));
		self.parser.end();
		err
	}
}