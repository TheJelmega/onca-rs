use crate::*;


#[test]
fn test_character_class_match() {
	// Generic character types: digit
	check_matches(r"\d", RegexFlags::None, &["0"], &["a"]);
	check_matches(r"\D", RegexFlags::None, &["a"], &["0"]);

	// Generic character types: horizontal whitespace
	check_matches(r"\h", RegexFlags::None, &[" "], &["a"]);
	check_matches(r"\H", RegexFlags::None, &["a"], &[" "]);

	// Generic character types: vertical whitesapce
	check_matches(r"\v", RegexFlags::None, &["\n"], &["a"]);
	check_matches(r"\V", RegexFlags::None, &["a"], &["\n"]);

	// Generic character types: whitespace
	check_matches(r"\s", RegexFlags::None, &["\t", "\n"], &["a"]);
	check_matches(r"\S", RegexFlags::None, &["a"], &["\t", "\n"]);

	// Generic character types: word
	check_matches(r"\w", RegexFlags::None, &["a", "9", "_"], &[" "]);
	check_matches(r"\W", RegexFlags::None, &[" "], &["a", "9", "_"]);

	// In unicode category
	check_matches(r"\p{Sm}", RegexFlags::None, &["+"], &["a"]);
	check_matches(r"\P{Sm}", RegexFlags::None, &["a"], &["+"]);

	// In unicode script
	check_matches(r"\p{Cyrillic}", RegexFlags::None, &["Щ"], &["a"]);
	check_matches(r"\p{Cyrl}"    , RegexFlags::None, &["Щ"], &["a"]);
	check_matches(r"\P{Cyrillic}", RegexFlags::None, &["a"], &["Щ"]);

	// TODO
	//check_matches(r"\R", RegexFlags::None, &["\r\n", "\n", "\r"], &["a"]);
}

#[test]
fn test_pcre_character_class_match() {
	// PCRE Xan (alphanumeric)
	check_matches(r"\p{Xan}", RegexFlags::None, &["a", "9"], &[" ", "_"]);
	check_matches(r"\P{Xan}", RegexFlags::None, &[" ", "_"], &["a", "9"]);

	// PCRE Xwd (Perl word)
	check_matches(r"\p{Xwd}", RegexFlags::None, &["a", "9"], &[" ", "_"]);
	check_matches(r"\P{Xwd}", RegexFlags::None, &[" ", "_"], &["a", "9"]);

	// PCRE Xps (POSIX space)
	check_matches(r"\p{Xps}", RegexFlags::None, &[" "], &["a"]);
	check_matches(r"\P{Xps}", RegexFlags::None, &["a"], &[" "]);

	// PCRE Xsp (Perl space)
	check_matches(r"\p{Xsp}", RegexFlags::None, &[" "], &["a"]);
	check_matches(r"\P{Xsp}", RegexFlags::None, &["a"], &[" "]);

	// PCRE Xuc (Universal Character Name)
	check_matches(r"\p{Xuc}", RegexFlags::None, &["@", "\u{A0A0}"], &["a"]);
	check_matches(r"\P{Xuc}", RegexFlags::None, &["a"], &["@", "\u{A0A0}"]);
}

#[test]
fn test_posix_character_class_match() {
	// Posix alnum
	check_matches("[:alnum:]" , RegexFlags::None, &["a", "0"], &[" "]);
	check_matches("[:^alnum:]", RegexFlags::None, &[" "], &["a", "0"]);

	// Posix alpha
	check_matches("[:alpha:]" , RegexFlags::None, &["a"], &["0", " "]);
	check_matches("[:^alpha:]", RegexFlags::None, &["0", " "], &["a"]);

	// Posix ascii
	check_matches("[:ascii:]" , RegexFlags::None, &["a", "0",  " "], &["\u{F0}"]);
	check_matches("[:^ascii:]", RegexFlags::None, &["\u{F0}"], &["a", "0",  " "]);

	// Posix blank
	check_matches("[:blank:]" , RegexFlags::None, &[" "], &["a"]);
	check_matches("[:^blank:]", RegexFlags::None, &["a"], &[" "]);

	// Posix control
	check_matches("[:cntrl:]" , RegexFlags::None, &["\x07"], &["a"]);
	check_matches("[:^cntrl:]", RegexFlags::None, &["a"], &["x07"]);

	// Posix digit
	check_matches("[:digit:]" , RegexFlags::None, &["1"], &["a"]);
	check_matches("[:^digit:]", RegexFlags::None, &["a"], &["1"]);

	// Posix graph
	check_matches("[:graph:]" , RegexFlags::None, &["a"], &[" ", "\x07", "\u{061C}", "\u{180E}", "\u{2067}"]);
	check_matches("[:^graph:]", RegexFlags::None, &[" ", "\x07", "\u{061C}", "\u{180E}", "\u{2067}"], &["a"]);

	// Posix print
	check_matches("[:print:]" , RegexFlags::None, &["a", " "], &["\x07", "\u{061C}", "\u{180E}", "\u{2067}"]);
	check_matches("[:^print:]", RegexFlags::None, &["\x07", "\u{061C}", "\u{180E}", "\u{2067}"], &["a", " "]);

	// Posix lower
	check_matches("[:lower:]" , RegexFlags::None, &["a"], &["A"]);
	check_matches("[:^lower:]", RegexFlags::None, &["A"], &["a"]);

	// Posix upper
	check_matches("[:upper:]" , RegexFlags::None, &["A"], &["a"]);
	check_matches("[:^upper:]", RegexFlags::None, &["a"], &["A"]);

	// Posix punct
	check_matches("[:punct:]" , RegexFlags::None, &["?"], &["a"]);
	check_matches("[:^punct:]", RegexFlags::None, &["a"], &["?"]);

	// Posix word
	check_matches("[:word:]" , RegexFlags::None, &["a"], &["_", " "]);
	check_matches("[:^word:]", RegexFlags::None, &["_", " "], &["a"]);
	
	// Posix xdigit
	check_matches("[:xdigit:]" , RegexFlags::None, &["8", "a", "F"], &["G", " "]);
	check_matches("[:^xdigit:]", RegexFlags::None, &["G", " "], &["8", "a", "F"]);
}

#[test]
fn test_character_class_def() {
	check_matches("[a]" , RegexFlags::None, &["a"], &["b"]);
	check_matches("[^a]", RegexFlags::None, &["b"], &["a"]);

	check_matches("[abc]" , RegexFlags::None, &["a", "b", "c"], &["d"]);
	check_matches("[^abc]", RegexFlags::None, &["d"], &["a", "b", "c"]);

	check_matches("[a-c]" , RegexFlags::None, &["a", "b", "c"], &["d"]);
	check_matches("[^a-c]", RegexFlags::None, &["d"], &["a", "b", "c"]);

	check_matches("[a-c-e]" , RegexFlags::None, &["a", "b", "c", "e", "-"], &["d"]);
	check_matches("[^a-c-e]", RegexFlags::None, &["d"], &["a", "b", "c", "e", "-"]);

	check_matches("[a^]", RegexFlags::None, &["a", "^"], &["b"]);

	check_matches("[a]b]"  , RegexFlags::None, &["ab]"], &["b", "bb]"]);
	check_matches("[W-]46]", RegexFlags::None, &["W46]", "-46]"], &["46]", "W46"]);
}

#[test]
fn test_groups() {
	// Literal
	check_matches("literal", RegexFlags::None, &["literal"], &["litera"]);

	// Group
	check_matches("(literal)", RegexFlags::None, &["literal"], &["litera"]);

	// Non-captured group
	check_matches("(?:literal)", RegexFlags::None, &["literal"], &["litera"]);
	
	// Non-multi capture alteration
	check_matches("((literal)|(other))", RegexFlags::None, &["literal", "other"], &["othr"]);
	
	// Capture alteration
	check_matches("(?|(literal)|(other))", RegexFlags::None, &["literal", "other"], &["othr"]);
	
	// Named capture (tag style)
	check_matches("(?<capture>literal)", RegexFlags::None, &["literal"], &["litera"]);
	
	// Named capture (perl style)
	check_matches("(?'capture'literal)", RegexFlags::None, &["literal"], &["litera"]);
	
	// Named capture (python style)
	check_matches("(?P<capture>literal)", RegexFlags::None, &["literal"], &["litera"]);
	
}

#[test]
fn test_match() { 
	// Literal
	check_matches("literal", RegexFlags::None, &["literal"], &["litera"]);

	// Group
	check_matches("(literal)", RegexFlags::None, &["literal"], &["litera"]);

	// Alternation
	check_matches("(literal|other)", RegexFlags::None, &["literal", "other"], &["othr"]);

	// start & end
	check_matches("^a" , RegexFlags::None, &["a"], &[""]);
	check_matches("a^a", RegexFlags::None, &[], &["", "a"]);
	check_matches("a$" , RegexFlags::None, &["a"], &[""]);
	check_matches("a$a", RegexFlags::None, &[], &["", "a"]);
}

#[test]
fn test_match_repetition() {
	check_matches("a?"     , RegexFlags::None, &["", "a"], &["aa", "s"]);
	check_matches("a+"     , RegexFlags::None, &["a", "aa"], &["", "s"]);
	check_matches("a*"     , RegexFlags::None, &["", "a", "aa"], &["s"]);
	check_matches("a{2}"   , RegexFlags::None, &["aa"], &["", "a", "aaa", "s"]);
	check_matches("a{2,}"  , RegexFlags::None, &["aa", "aaa"], &["", "a", "s"]);
	check_matches("a{2,4}" , RegexFlags::None, &["aa", "aaa", "aaaa"], &["", "a", "aaaaa", "s"]);
	check_matches("a{2,}a" , RegexFlags::None, &["aaaa"], &["aa"]);
	check_matches("a{2,}?a", RegexFlags::None, &["aaa"], &["aaaaa"]);
	check_matches("a{2,}+a", RegexFlags::None, &[], &["aaa", "aaaaa"]);
}

#[test]
fn test_backref() {
	check_matches(r"(a)\g1"   , RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(a)\g{1}" , RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(a)\g-1"  , RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(a)\g{-1}", RegexFlags::None, &["aa", ], &["", "a", "bb"]);

	check_matches(r"(?<name>a)\g{name}", RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(?<name>a)\k{name}", RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(?<name>a)\k<name>", RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(?<name>a)\k'name'", RegexFlags::None, &["aa", ], &["", "a", "bb"]);
	check_matches(r"(?<name>a)(?P=name)", RegexFlags::None, &["aa", ], &["", "a", "bb"]);
}

#[test]
fn test_lookahead_lookbehind() {
	check_matches(r"foo(?=bar)\w{3}", RegexFlags::None, &["foobar"], &["foobaz"]);
	check_matches(r"foo(?!bar)\w{3}", RegexFlags::None, &["foobaz"], &["foobar"]);
	check_matches(r"\w{3}(?<=foo)bar", RegexFlags::None, &["foobar"], &["bazbar"]);
	check_matches(r"\w{3}(?<!foo)bar", RegexFlags::None, &["bazbar"], &["foobar"]);

	// nested
	check_matches(r".{6}(?<=(?<!foo)bar)baz", RegexFlags::None, &["barbarbaz"], &["foobarbaz"])
}

#[test]
fn test_conditional() {
	check_matches(r"(a)?(?(1)b|c)", RegexFlags::None, &["ab", "c"], &["b", "ac"]);
	check_matches(r"(a)?(?(-1)b|c)", RegexFlags::None, &["ab", "c"], &["b", "ac"]);
	check_matches(r"(?<name>a)?(?(<name>)b|c)", RegexFlags::None, &["ab", "c"], &["b", "ac"]);
	check_matches(r"(?'name'a)?(?('name')b|c)", RegexFlags::None, &["ab", "c"], &["b", "ac"]);
	check_matches(r"a?(?(?<=(a))b|c)", RegexFlags::None, &["ab", "c"], &["b", "ac"]);


	check_matches(r"(?(DEFINE)b)", RegexFlags::None, &[""], &["a", "b", "c"])
}

fn check_matches(regex_s: &str, flags: RegexFlags, valid: &[&str], invalid: &[&str]) {
	let regex = Regex::new(regex_s, flags).unwrap();
	for val in valid {
		assert!(regex.is_match(*val).is_some(), "Failed to match regex '{regex_s}' with value '{val}'");
	}
	for val in invalid {
		assert!(regex.is_match(*val).is_none(), "Should not match regex '{regex_s}' with value '{val}'");
	}
}


#[test]
fn opt_test() {
	let _ = Regex::new(r"a\aa", RegexFlags::None);
}