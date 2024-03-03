//! Libary to retrieve information about unicode characters
//! 
//! Official info:
//! https://www.unicode.org/reports/tr44/
//! https://www.unicode.org/reports/tr51/

use core::fmt;
use std::{
	fmt::Debug,
};

use onca_base::EnumFromNameT;
use onca_common_macros::{flags, EnumFromIndex, EnumFromName};


mod unicode;

// Unicode index into info arrays
#[derive(Clone, Copy, PartialEq, Eq)]
enum UnicodeIndex {
	Single(u32),
	Range(u32, u32),
}

impl UnicodeIndex {
	fn cmp_internal(&self, other: &Self) -> core::cmp::Ordering {
		match self {
		    UnicodeIndex::Single(s) => match other {
    		    UnicodeIndex::Single(o) => s.cmp(o),
    		    UnicodeIndex::Range(o_begin, o_end) => if s < o_begin {
					core::cmp::Ordering::Less
				} else if s > o_end {
					core::cmp::Ordering::Greater
				} else {
					core::cmp::Ordering::Equal
				},
    		},
		    UnicodeIndex::Range(s_begin, s_end) =>  match other {
    		    UnicodeIndex::Single(o) => if s_end < o {
					core::cmp::Ordering::Less
				} else if s_begin > o {
					core::cmp::Ordering::Greater
				} else {
					core::cmp::Ordering::Equal
				},
    		    UnicodeIndex::Range(o_begin, o_end) => if s_end < o_begin {
					core::cmp::Ordering::Less
				} else if s_begin > o_end {
					core::cmp::Ordering::Greater
				} else {
					core::cmp::Ordering::Equal
				},
    		},
		}
	}
}

impl PartialEq<u32> for UnicodeIndex {
    fn eq(&self, other: &u32) -> bool {
        match self {
            UnicodeIndex::Single(val) => val == other,
            UnicodeIndex::Range(begin, end) => begin <= other && end >= other,
        }
    }
}

impl PartialOrd<u32> for UnicodeIndex {
    fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
        match self {
            UnicodeIndex::Single(val) => val.partial_cmp(other),
            UnicodeIndex::Range(begin, end) => {
				if begin > other {
					Some(core::cmp::Ordering::Greater)
				} else if end < other {
					Some(core::cmp::Ordering::Less)
				} else {
					Some(core::cmp::Ordering::Equal)
				}
			},
        }
    }
}

impl PartialOrd for UnicodeIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp_internal(other))
    }
}

impl Ord for UnicodeIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cmp_internal(other)
    }
}

impl Debug for UnicodeIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(val) => write!(f, "UnicodeIndex::Single({val:#07X}         )"),
			//f.debug_tuple("Single").field(arg0).finish(),
            Self::Range(begin, end) => write!(f, "UnicodeIndex::Range ({begin:#07X}, {end:#07X})"),
			//f.debug_tuple("Range").field(arg0).field(arg1).finish(),
        }
    }
}

//==============================================================

/// Unicode category
#[flags(parse_from_name)]
pub enum Category {
	/// Lu: An upper case letter.
	#[parse_name("Lu")]
	UppercaseLetter,
	/// Ll: A lowercase letter.
	#[parse_name("Ll")]
	LowercaseLetter,
	/// Lt: A digraph encoded as a signle character, with first part uppercase.
	#[parse_name("Lt")]
	TitlecaseLetter,
	/// LC: Lu | Ll | Lt
	#[parse_name("LC")]
	CasedLetter = UppercaseLetter | LowercaseLetter | TitlecaseLetter,
	/// Lm: A modifier letter.
	#[parse_name("Lm")]
	ModifierLetter,
	/// Lo: Other Letters, including syllables and ideographs.
	#[parse_name("Lo")]
	OtherLetter,
	/// L: Lu | Ll | Lt | Lm | Lo
	#[parse_name("L")]
	Letter = CasedLetter | ModifierLetter | OtherLetter,
	/// Mn: A nonspacing combining mark (zero advance width).
	#[parse_name("Mn")]
	NonspacingMark,
	/// Mc: A spacing combining mark (positive advance width).
	#[parse_name("Mc")]
	SpacingMark,
	/// Me: An enclosing combining mark.
	#[parse_name("Me")]
	EnclosingMark,
	/// M: Mn | Mc | Me
	#[parse_name("M")]
	Mark = NonspacingMark | SpacingMark | EnclosingMark,
	/// Nd: A decimal digit.
	#[parse_name("Nd")]
	DecimalNumber,
	/// Nl: A letterlike numeric character.
	#[parse_name("Nl")]
	LetterNumber,
	/// No: A numeric character of other type.
	#[parse_name("No")]
	OtherNumber,
	/// N: Nd | Nl | No
	#[parse_name("N")]
	Number = DecimalNumber | LetterNumber | OtherNumber,
	/// Pc: A connecting punctuation mark, like a tie.
	#[parse_name("Pc")]
	ConnectorPunctuation,
	/// Pd: A dash or hyphen punctuation mark.
	#[parse_name("Pd")]
	DashPunctuation,
	/// Ps: An opening punctuation mark (of a pair).
	#[parse_name("Ps")]
	OpenPunctuation,
	/// Pe: A closing punctuation mark (of a pair).
	#[parse_name("Pe")]
	ClosePunctuation,
	/// Pi: An initial quotiation mark.
	#[parse_name("Pi")]
	InitialPunctuation,
	/// Pf: A final quotation mark.
	#[parse_name("Pf")]
	FinalPunctuation,
	/// Po: A punctuation mark of other type.
	#[parse_name("Po")]
	OtherPunctuation,
	// P: Pc | Pd | Ps | Pe | Pi | Pf | Po
	#[parse_name("P")]
	Punctuation = ConnectorPunctuation | DashPunctuation | OpenPunctuation | ClosePunctuation | InitialPunctuation | FinalPunctuation | OtherPunctuation,
	/// Sm: A symbol of mathematical use.
	#[parse_name("Sm")]
	MathSymbol,
	/// Sc: A currency sign.
	#[parse_name("Sc")]
	CurrencySymbol,
	/// Sk: Anon-letterlike modifier symbol.
	#[parse_name("Sk")]
	ModifierSymbol,
	/// So: A symbol of other type.
	#[parse_name("So")]
	OtherSymbol,
	/// S: Sm | Sc | Sk | So
	#[parse_name("S")]
	Symbol = MathSymbol | CurrencySymbol |ModifierSymbol | OtherSymbol,
	/// Zs: A space character (of various non-zero widths).
	#[parse_name("Zs")]
	SpaceSeparator,
	/// Zl: U+2028 LINE SEPARATOR only.
	#[parse_name("Zl")]
	LineSeparator,
	/// Zp: U+2029 PARAGRAPH SEPARATOR only.
	#[parse_name("Zp")]
	ParagraphSeparator,
	/// Z: Zs | Zl | Zp
	#[parse_name("Z")]
	Separator = SpaceSeparator | LineSeparator | ParagraphSeparator,
	/// Cc: A C0 or C1 control code.
	#[parse_name("Cc")]
	Control,
	/// Cf: A format control character.
	#[parse_name("Cf")]
	Format,
	/// Cs: A surrogate code point.
	#[parse_name("Cs")]
	Surrogate,
	/// Co: A private-use character.
	#[parse_name("Co")]
	PrivateUse,
	/// Cn: A reserved unassigned code point or a noncharacter.
	#[parse_name("Cn")]
	Unsassigned,
	/// C: Cc | Cf | Cs | Co | Cn
	#[parse_name("C")]
	Other = Control | Format | Surrogate | PrivateUse | Unsassigned,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromIndex)]
pub enum CanonicalCombiningClass {
	///Spacing and enclosing marks, also many vowel and consonant signs, even if nonspacing.
	NotReordered = 0,
	/// Marks which overlay a base letter or symbol
	Overlay      = 1,
	/// Diacritic reading marks for CJK unified ideographs,
	HanReading   = 6,
	/// Diacritic Nukta marks in Brahmi-derived scripts.
	Nukta        = 7,
	/// Hiragana/Katakan voicing marks.
	KanaVoicing  = 8,
	/// Viramas
	Virama       = 9,
	/// Start of fixed position classes
	Ccc10        = 10,
	Ccc11        = 11,
	Ccc12        = 12,
	Ccc13        = 13,
	Ccc14        = 14,
	Ccc15        = 15,
	Ccc16        = 16,
	Ccc17        = 17,
	Ccc18        = 18,
	Ccc19        = 19,
	Ccc20        = 20,
	Ccc21        = 21,
	Ccc22        = 22,
	Ccc23        = 23,
	Ccc24        = 24,
	Ccc25        = 25,
	Ccc26        = 26,
	Ccc27        = 27,
	Ccc28        = 28,
	Ccc29        = 29,
	Ccc30        = 30,
	Ccc31        = 31,
	Ccc32        = 32,
	Ccc33        = 33,
	Ccc34        = 34,
	Ccc35        = 35,
	Ccc36        = 36,
	Ccc84        = 84,
	Ccc91        = 91,
	Ccc103       = 103,
	Ccc107       = 107,
	Ccc118       = 118,
	Ccc122       = 122,
	Ccc129       = 129,
	Ccc130       = 130,
	Ccc132       = 132,
	Ccc133       = 133, // Reserved
	/// Marks attached at the bottom Left.
	Atbl         = 200,
	/// Marks attached at the bottom.
	Atb          = 202,
	/// Marks attached directly above.
	Ata          = 214,
	/// Marks attached at the top right.
	Atar         = 216,
	/// Distinct marks at the bottom left.
	Bl           = 218,
	/// Distinct marks directly below.
	B            = 220,
	/// Distinct marks at the bottom right.
	Br           = 222,
	/// Distinct marks to the left.
	L            = 224,
	/// Distinct marks to the right
	R            = 226,
	/// Distinct marks at the top left.
	Al           = 228,
	/// Distinct marks directly above.
	A            = 230,
	/// Distinct marks at the top right.
	Ar           = 232,
	/// Distinct marks subtending two bases.
	Db           = 233,
	/// Distinct marks extending above two bases.
	Da           = 234,
	/// Greek iota subscript only
	Is           = 240,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum BidirectionalClass {
	/// Any strong left-to-right characters (Strong type).
	#[parse_name("L")]
	LeftToRight,
	/// Any strong right-to-left (non-Arabic type) character (Strong type).
	#[parse_name("R")]
	RightToLeft,
	/// Any strong right-to-left (Arabic type) character (Strong type).
	#[parse_name("AL")]
	ArabicLetter,
	/// Any ASCII digit or Eastern Arabic-Indic digit (Weak type).
	#[parse_name("EN")]
	EuropeanNumber,
	/// Plus and minus signs (Weak type).
	#[parse_name("ES")]
	EuropeanSeparator,
	/// A terminator in a numeric format context, includes currency signs (Weak type).
	#[parse_name("ET")]
	EuropeanTerminator,
	/// Any Arabic-Indic digit (Weak type).
	#[parse_name("AN")]
	ArabicNumber,
	/// Commas, colons, and slashes (Weak type).
	#[parse_name("CS")]
	CommonSeparator,
	/// Any nonspacing mark (Weak type).
	#[parse_name("NSM")]
	NonspacingMark,
	/// Most format charactes, control codes, or noncharacters (Weak type).
	#[parse_name("BN")]
	BoundaryNeutral,
	/// Various newline characters (Neutral type).
	#[parse_name("B")]
	ParagraphSeparator,
	/// Various segment-related (Neutral type).
	#[parse_name("S")]
	SegmentSeparator,
	/// Spaces (Neutral type).
	#[parse_name("WS")]
	WhiteSpace,
	/// Most other symbols and punctuation marks (Neutral type).
	#[parse_name("ON")]
	OtherNeutral,
	/// U+202A: the LR embedding control (Explicit formatting types).
	#[parse_name("LRE")]
	LeftToRightEmbedding,
	/// U+202D: The LR override control (Explicit formatting types).
	#[parse_name("LRO")]
	LeftToRightOverride,
	/// U+202B: The RL embedding control (Explicit formatting types).
	#[parse_name("RLE")]
	RightToLeftEmbedding,
	/// U+202E: The RL override control (Explicit formatting types).
	#[parse_name("RLO")]
	RightToLeftOverride,
	/// U+202C: Terminates an embedding or override control (Explicit formatting types).
	#[parse_name("PDF")]
	PopDirectionalFormat,
	/// U+2066: The LR isolate control (Explicit formatting types).
	#[parse_name("LRI")]
	LeftToRightIsolate,
	/// U+2067: The RL isolate control (Explicit formatting types).
	#[parse_name("RLI")]
	RightToLeftIsolate,
	/// U+2068: The first strong isolate control (Explicit formatting types).
	#[parse_name("FSI")]
	FirstStrongIsolate,
	/// U+2069: Terminates an isolate control (Explicit formatting types).
	#[parse_name("PDI")]
	PopDirectionalIsolate,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rational {
	numerator:   i64,
	denominator: u64,
}

#[flags(parse_from_name)]
pub enum UnicodeFlags {
	/// The character is a "mirrored" character in bidirectional text.
	BidiMirrored,
	/// The character is an emoji.
	Emoji,
	/// The character should be rendered as an emoji, instead of text.
	#[parse_name("Emoji_Presentation")]
	EmojiPresentation,
	/// The character is an emoji modifier, e.g. skin tone modifier.
	#[parse_name("Emoji_Modifier")]
	EmojiModifier,
	/// The character is a base for emoji modifier.
	#[parse_name("Emoji_Modifier_Base")]
	EmojiModifierBase,
	/// The character is used in emoji sequences that normally do not appear on emoji keyboards as separate choises, such as base characters for emoji keycaps.
	/// Also includes are REgionalIndicator charactes and U+FEoF Variation Selector-16.
	///
	/// # Note
	///
	/// All characters in emoji sequences are either Emoji or EmojiComponent.
	/// However, implementations must not assume that all EmojiComponent characters are also Emoji.
	///There are some non-emoji characters that are used inv arious emoji sequences, such as tag characters and ZWJ.
	#[parse_name("Emoji_Component")]
	EmojiComponent,
	/// The character is a pictographic symbol, or in a reserved ranges in blocks largly associated with emoji characters.
	/// This enables segmentation rulses invloving emoji to be specfied stably, even in cases where an excisting non-emoji pictographics sybol later comes to be treated as an emoji.
	///
	/// # Note
	///
	/// This property is used in the regex definitions for the Default Grapheme Cluster Boundary Specification.
	#[parse_name("Extended_Pictographic")]
	ExtendedPictographic,
	/// The character is excluded from composition.
	CompositionExclusion,
	/// An ASCII character commonly used for the representation fo hexadecimal character.
	#[parse_name("ASCII_Hex_Digit")]
	AsciiHexDigit,
	/// Format control characters which have specific functions in the Unicode Bidirectional Algorithm.
	#[parse_name("Bidi_Control")]
	BidiControl,
	/// Punctuation characters explicitly called out as dashes in the Unicode Standard, plus their compatibility equivalents.
	/// Most of these have the GeneralCategory value `Pd`, but some have the GeneralCategory value `SM` because of their use in mathematics.
	Dash,
	/// For a machine-readable list of deprecated charactes.
	/// No characters will ever be removed from the standard, but the usage of deprecated characters is strongly discourages.
	Deprecated,
	/// Characters that linguistically modify the meaning of another characterss to which they apply.
	/// Some diacritics are not combining characters, and some combining characters are not diacritics.
	Diacritic,
	/// Characters whose principal function is to extend the value of a preceding alphabetic character or to extend the shape of adjacent characters.
	/// Typical of these are length marks, iteration marks, and the Arabic tatweel.
	Extender,
	/// Characters commonly used for the representation of hexadecimal numbers, plus their compatiblity equivalents.
	#[parse_name("Hex_Digit")]
	HexDigit,
	/// Characters considered CJKV (Chinenese, Japanese, Korean and Vietnamese) or other sinoform (Chinese writing-related) ideographs.
	/// This property roughly defines the class of "Chinese characters" and does not include characters of other logographic scripts such as Cuneiform or Eqyptian Hieroglyphs.
	/// The Ideographic property is used in the definition of Ideographic Description Sequences.
	Ideographic,
	#[parse_name("ID_Compat_Math_Start")]
	IdCompatMathStart,
	#[parse_name("ID_Compat_Math_Continue")]
	IdCompatMathContinue,
	#[parse_name("IDS_Unary_Operator")]
	IdsUnaryOperator,
	#[parse_name("IDS_Binary_Operator")]
	IdsBinaryOperator,
	#[parse_name("IDS_Trinary_Operator")]
	IdsTrinaryOperator,
	/// Format control characters which have specific functions for control of cursive joining and ligation.
	#[parse_name("Join_Control")]
	JoinControl,
	/// A small number of spacing vowel letters occuring in certain Southeast Asian scripts such as Thai and Loa, which use a visual order display model.
	/// These letters are stored in text ahead of syllable-initial consonants, and require special handling for processes such as searching and sorting.
	#[parse_name("Logical_Order_Exception")]
	LogicalOrderException,
	/// Code points permanently reserved for internal use.
	#[parse_name("Noncharacter_Code_Point")]
	NoncharacterCodePoint,
	/// Used in deriving `Alphabetic` property.
	#[parse_name("Other_Alphabetic")]
	OtherAlphabetic,
	/// Used in dreiving the `DeafultIgnorableCodePoint` property.
	#[parse_name("Other_Default_Ignorable_Code_Point")]
	OtherDefaultIgnorableCodePoint,
	/// Used in dreiving the `GraphemeExtend` property.
	#[parse_name("Other_Grapheme_Extend")]
	OtherGraphemeExtend,
	/// Used to maintin backward compatibility of `IDContinue`.
	#[parse_name("Other_ID_Continue")]
	OtherIdContinue,
	/// Used to maintin backward compatibility of `IDStart`.
	#[parse_name("Other_ID_Start")]
	OtherIdStart,
	/// Used in dreiving the `Lowercase` property.
	#[parse_name("Other_Lowercase")]
	OtherLowercase,
	/// Used in dreiving the `Math` property.
	#[parse_name("Other_Math")]
	OtherMath,
	/// Used in dreiving the `Uppercase` property.
	#[parse_name("Other_Uppercase")]
	OtherUppercase,
	/// Used for pattern syntax.
	#[parse_name("Pattern_Syntax")]
	PatternSyntax,
	/// Used for pattern syntax.
	#[parse_name("Pattern_White_Space")]
	PatternWhiteSpace,
	/// A small class of visible format controls, which precede and then span a sequence of other characters, usually digits.
	/// These have also been known as "subtending marks", because most of them take a form which visually extends underneath the sequence of following digits.
	#[parse_name("Prepended_Concatenation_Mark")]
	PrependedConcatenationMark,
	/// Punctuation characters that function as quotation marks.
	#[parse_name("Quotation_Mark")]
	QuotationMark,
	/// Used in the definition of Ideographic Description Sequences.
	Radical,
	/// Property of the regional indicator characters, U+1F1E6..=U+1F1FF.
	/// This property is referenced in various segmentation algorithms, to assist in correct breaking around emoji flag sequences.
	#[parse_name("Regional_Indicator")]
	RegionalIndicator,
	/// Punctuation characters that generally mark the end of sentences.
	#[parse_name("Sentence_Terminal")]
	SentenceTerminal,
	/// Charactes with a "soft dot", like `i` or `j`. An accent placed on these characters causes the dot to disappear.
	/// An explicit `dot` above can be added where required, such as in lithuanian.
	#[parse_name("Soft_Dotted")]
	SoftDotted, 
	/// Punctuation characters that generally mark the end of textual units.
	#[parse_name("Terminal_Punctuation")]
	TerminalPunctuation,
	/// A property which specified the exact set of Unified CJK Ideographs in the standard.
	/// This set excludes CJK Compatibility ideographs (which have canonical decompositions of Unified CJK Ideographs), as well as characters from the CJK Symbols and Puncuation block.
	/// The calss of UnifiedIdeograph charactes is a proper subset of the calss of Ideographic characters.
	#[parse_name("Unified_Ideograph")]
	UnifiedIdeograph,
	/// Indicates characters that are Variation Selectors.
	#[parse_name("Variation_Selector")]
	VariationSelector,
	/// Spaces, separator charactes and other control characters which should be treated by programming langauges as "white space" for the purpose of parsing elements.
	/// See also lineBreak, GraphemeCulsterBreak, SentenceBreak, and WordBreak, which classify space charactes and related contorls somewhet differently for particular text segmentation contexts.
	#[parse_name("White_Space")]
	WhiteSpace,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum Age {
	Unknown,
	#[parse_name("1.1")]
	Version1_1,
	#[parse_name("2.0")]
	Version2_0,
	#[parse_name("2.1")]
	Version2_1,
	#[parse_name("3.0")]
	Version3_0,
	#[parse_name("3.1")]
	Version3_1,
	#[parse_name("3.2")]
	Version3_2,
	#[parse_name("4.0")]
	Version4_0,
	#[parse_name("4.1")]
	Version4_1,
	#[parse_name("5.0")]
	Version5_0,
	#[parse_name("5.1")]
	Version5_1,
	#[parse_name("5.2")]
	Version5_2,
	#[parse_name("6.0")]
	Version6_0,
	#[parse_name("6.1")]
	Version6_1,
	#[parse_name("6.2")]
	Version6_2,
	#[parse_name("6.3s")]
	Version6_3,
	#[parse_name("7.0")]
	Version7_0,
	#[parse_name("8.0")]
	Version8_0,
	#[parse_name("9.0")]
	Version9_0,
	#[parse_name("10.0")]
	Version10_0,
	#[parse_name("11.0")]
	Version11_0,
	#[parse_name("12.0")]
	Version12_0,
	#[parse_name("12.1")]
	Version12_1,
	#[parse_name("13.0")]
	Version13_0,
	#[parse_name("14.0")]
	Version14_0,
	#[parse_name("15.0")]
	Version15_0,
	#[parse_name("15.1")]
	Version15_1,
}

/// East Asian Width
/// 
/// https://www.unicode.org/reports/tr11/
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum EastAsianWidth {
	/// Full width.
	#[parse_name("F")]
	FullWidth,
	/// Half width.
	#[parse_name("H")]
	HalfWidth,
	/// Half width.
	#[parse_name("Na")]
	Narrow,
	/// Wide.
	#[parse_name("W")]
	Wide,
	/// Ambiguous.
	#[parse_name("A")]
	Ambiguous,
	/// Neutral.
	#[parse_name("N")]
	Neutral,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
/// Dual Joining Group
pub enum DualJoiningGroup {
	/// No joining group.
	#[parse_name("No_Joining_Group")]
	NoGroup,
	// Arabic
	/// Beh joining group.
	/// 
	/// Includes Teh and Theh.
	#[parse_name("BEH")]
	Beh,
	/// Noon Joining Group.
	/// 
	/// Includes Noon Ghunna.
	#[parse_name("NOON")]
	Noon,
	/// African Noon joinng group.
	#[parse_name("AFRICAN NOON")]
	AfricanNoon,
	/// Nya joining group.
	/// 
	/// Includes Jawi Nya,
	#[parse_name("NYA")]
	Nya,
	/// Yeh joining group.
	/// 
	/// Includes Alef Maksura.
	#[parse_name("YEH")]
	Yeh,
	/// Farsi Yeh joining group.
	#[parse_name("FARSI YEH")]
	FarsiYeh,
	/// This Yah joining group.
	/// 
	/// Final and isolated forms are not attested.
	#[parse_name("THIN YEH")]
	ThinYeh,
	/// Barushaski Yeh Barree joining group.
	/// 
	/// Dual joining, as opposed to Yeh Barree.
	#[parse_name("BURUSHASKI YEH BARREE")]
	BurushaskiYehBarree,
	/// Hah joining group.
	/// 
	/// Includes Khah and Jeem.
	#[parse_name("HAH")]
	Hah,
	/// Seen joining group.
	/// 
	/// Includes Sheen.
	#[parse_name("SEEN")]
	Seen,
	/// Sad joining group.
	/// 
	/// Includes Dad.
	#[parse_name("SAD")]
	Sad,
	/// Tah joining group.
	/// 
	/// Includes Zah.
	#[parse_name("TAH")]
	Tah,
	/// Ain joining group.
	/// 
	/// Includes Ghain.
	#[parse_name("AIN")]
	Ain,
	/// Feh joining group.
	#[parse_name("FEH")]
	Feh,
	/// African Feh joining group.
	#[parse_name("AFRICAN FEH")]
	AfricanFeh,
	/// Qaf joining group.
	#[parse_name("QAF")]
	Qaf,
	/// African Qaf joining group.
	#[parse_name("AFRICAN QAF")]
	AfricanQaf,
	/// Meem joining group.
	#[parse_name("MEEM")]
	Meem,
	/// Heh joining group.
	#[parse_name("HEH")]
	Heh,
	/// Knotted Heh joining group.
	/// 
	/// This is a regional variation
	#[parse_name("KNOTTED HEH")]
	KnottedHeh,
	/// Heh Goal joining group.
	/// 
	/// Includes Hamza On Heh Goal.
	#[parse_name("HEH GOAL")]
	HehGoal,
	/// Kaf joining group.
	#[parse_name("KAF")]
	Kaf,
	/// Swash Kaf joining group.
	#[parse_name("SWASH KAF")]
	SwashKaf,
	/// Gaf joining group.
	/// 
	/// Includes Keheh.
	#[parse_name("GAF")]
	Gaf,
	/// Lam joining group.
	#[parse_name("LAM")]
	Lam,
	
	// Syriac
	/// Beth joining group.
	/// 
	/// Includes Persian Bheth.
	#[parse_name("BETH")]
	Beth,
	/// Gamal joining group.
	/// 
	/// Includes Gamal Garshuni and Persian Ghamal.
	#[parse_name("GAMAL")]
	Gamal,
	/// Heth joining group.
	#[parse_name("HETH")]
	Heth,
	/// Teth Garshuni joining group.
	#[parse_name("TETH")]
	Teth,
	/// Yudh joining group.
	#[parse_name("YUDH")]
	Yudh,
	/// Kaph joining group.
	#[parse_name("KAPH")]
	Kaph,
	/// Khaph joining group.
	#[parse_name("KHAPH")]
	Khaph,
	/// Lamadh joining group.
	#[parse_name("LAMADH")]
	Lamadh,
	/// Mim joining group.
	#[parse_name("MIM")]
	Mim,
	/// Nun joining group.
	#[parse_name("NUN")]
	Nun,
	/// Semkath joining group.
	#[parse_name("SEMKATH")]
	Semkath,
	/// Final Semkath joining group.
	#[parse_name("FINAL SEMKATH")]
	FinalSemkath,
	/// E joining group.
	#[parse_name("E")]
	E,
	/// Pe joining group.
	#[parse_name("PE")]
	Pe,
	/// Reversed Pe joining group.
	#[parse_name("REVERSED PE")]
	ReversedPe,
	/// Fe joining group.
	/// 
	/// Sogdian.
	#[parse_name("FE")]
	Fe,
	/// Qaph joining group.
	#[parse_name("QAPH")]
	Qaph,
	/// Shin joining group.
	#[parse_name("SHIN")]
	Shin,
	/// Malayalam Nga joining group.
	#[parse_name("MALAYALAM NGA")]
	MalayalamNga,
	/// Malayalam Nya joining group.
	#[parse_name("MALAYALAM NYA")]
	MalayalamNya,
	/// Malayalam Tta joining group.
	#[parse_name("MALAYALAM TTA")]
	MalayalamTta,
	/// Malayalam Nna joining group.
	#[parse_name("MALAYALAM NNA")]
	MalayalamNna,
	/// Malayalam Nnna joining group.
	#[parse_name("MALAYALAM NNNA")]
	MalayalamNnna,
	/// Malayalam Lla joining group.
	#[parse_name("MALAYALAM LLA")]
	MalayalamLla,
	
	// Manicheaen
	/// Manicheaen Aleph joining group.
	#[parse_name("MANICHAEAN ALEPH")]
	ManicheaenAleph,
	/// Manicheaen Beth joining group.
	#[parse_name("MANICHAEAN BETH")]
	ManicheaenBeth,
	/// Manicheaen Gimel joining group.
	/// 
	/// Includes `Ghimel` mentioned in unicode standard.
	#[parse_name("MANICHAEAN GIMEL")]
	ManicheaenGimel,
	/// Manicheaen Lamedh joining group.
	#[parse_name("MANICHAEAN LAMEDH")]
	ManicheaenLamedh,
	/// Manicheaen Dhamedh joining group.
	#[parse_name("MANICHAEAN DHAMEDH")]
	ManicheaenDhamedh,
	/// Manicheaen Themedh joining group.
	#[parse_name("MANICHAEAN THAMEDH")]
	ManicheaenThamedh,
	/// Manicheaen Mem joining group.
	#[parse_name("MANICHAEAN MEM")]
	ManicheaenMem,
	/// Manicheaen Samekh joining group.
	#[parse_name("MANICHAEAN SAMEKH")]
	ManicheaenSamekh,
	/// Manicheaen Ayin joining group.
	#[parse_name("MANICHAEAN AYIN")]
	ManicheaenAyin,
	/// Manicheaen Pe joining group.
	#[parse_name("MANICHAEAN PE")]
	ManicheaenPe,
	/// Manicheaen Qoph joining group.
	#[parse_name("MANICHAEAN QOPH")]
	ManicheaenQoph,
	/// Manicheaen One joining group.
	#[parse_name("MANICHAEAN ONE")]
	ManicheaenOne,
	/// Manicheaen Five joining group.
	#[parse_name("MANICHAEAN FIVE")]
	ManicheaenFive,
	/// Manicheaen Ten joining group.
	#[parse_name("MANICHAEAN TEN")]
	ManicheaenTen,
	/// Manicheaen Twenty joining group.
	#[parse_name("MANICHAEAN TWENTY")]
	ManicheaenTwenty,
	
	// Hanifi Rhoingya
	
	/// Hanifi Rohingya Pa joining group.
	#[parse_name("HANIFI ROHINGYA PA")]
	HanifiRohingyaPa,
	/// Hanifi Rohingya Kinna Ya joining group.
	#[parse_name("HANIFI ROHINGYA KINNA YA")]
	HanifiRohingyaKinnaYa,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum RightJoiningGroup {
	/// No joining group
	#[parse_name("No_Joining_Group")]
	NoGroup,
	// Arabic
	/// Alef joining group.
	#[parse_name("ALEF")]
	Alef,
	/// Waw joining group.
	#[parse_name("WAW")]
	Waw,
	/// Straight Waw joining group.
	/// 
	/// Tatar Straight Waw.
	#[parse_name("STRAIGHT WAW")]
	StraightWaw,
	/// Dal joining group.
	/// 
	/// Includes Thal.
	#[parse_name("DAL")]
	Dal,
	/// Reh joining group.
	/// 
	/// Includes Zain.
	#[parse_name("REH")]
	Reh,
	/// Teh Marbuta joining group.
	/// 
	/// Includes Hamza On Heh.
	#[parse_name("TEH MARBUTA")]
	TehMarbuta,
	/// Teh Marbuta Goal joining group.
	#[parse_name("TEH MARBUTA GOAL")]
	TehMarbutaGoal,
	/// Yeh With Tail joining group.
	#[parse_name("YEH WITH TAIL")]
	YehWithTail,
	/// Yeh Barree joining group.
	#[parse_name("YEH BARREE")]
	YehBarree,
	/// Rohinga Yeh joining group.
	#[parse_name("ROHINGYA YEH")]
	RohingyaYeh,
	
	// Syriac
	/// Alaph joining group.
	#[parse_name("ALAPH")]
	Alaph,
	/// Dalath Rish joining group.
	/// 
	/// Includes Rish, Dotless Dalath Rish, and Persian Dhalath.
	#[parse_name("DALATH RISH")]
	DalathRish,
	/// He joining group.
	#[parse_name("HE")]
	He,
	/// Syriac waw joining group.
	#[parse_name("SYRIAC WAW")]
	SyriacWaw,
	/// Zain joining group.
	#[parse_name("ZAIN")]
	Zain,
	/// Zhain joining group.
	/// 
	/// Sogdian.
	#[parse_name("ZHAIN")]
	Zhain,
	/// Yudh He joining group.
	#[parse_name("YUDH HE")]
	YudhHe,
	/// Sadhe joining group.
	#[parse_name("SADHE")]
	Sadhe,
	/// Taw joining group.
	#[parse_name("TAW")]
	Taw,
	/// Suriyani Malayalam Ra joining group.
	#[parse_name("MALAYALAM RA")]
	MalayalamRa,
	/// Suriyani Malayalam Llla joining group.
	#[parse_name("MALAYALAM LLLA")]
	MalayalamLlla,
	/// Suriyani Malayalam Ssa joining group.
	#[parse_name("MALAYALAM SSA")]
	MalayalamSsa,
	
	// Manicheaen
	/// Manicheaen Daleth joining group.
	#[parse_name("MANICHAEAN DALETH")]
	ManicheaenDaleth,
	/// Manicheaen Waw joining group.
	#[parse_name("MANICHAEAN WAW")]
	ManicheaenWaw,
	/// Manicheaen Zayin joining group.
	#[parse_name("MANICHAEAN ZAYIN")]
	ManicheaenZayin,
	/// Manicheaen Teth joining group.
	#[parse_name("MANICHAEAN TETH")]
	ManicheaenTeth,
	/// Manicheaen Yodh joining group.
	#[parse_name("MANICHAEAN YODH")]
	ManicheaenYodh,
	/// Manicheaen Kaph joining group.
	#[parse_name("MANICHAEAN KAPH")]
	ManicheaenKaph,
	/// Manicheaen Sadhe joining group.
	#[parse_name("MANICHAEAN SADHE")]
	ManicheaenSadhe,
	/// Manicheaen Resh joining group.
	#[parse_name("MANICHAEAN RESH")]
	ManicheaenResh,
	/// Manicheaen Taw joining group.
	#[parse_name("MANICHAEAN TAW")]
	ManicheaenTaw,
	/// Manicheaen Hundred joining group.
	#[parse_name("MANICHAEAN HUNDRED")]
	ManicheaenHundred,
	
	// Other
	/// Vertical Tail joining group.
	#[parse_name("VERTICAL TAIL")]
	VerticalTail,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum LeftJoiningGroup {
	/// No joining group.
	#[parse_name("No_Joining_Group")]
	None,
	/// Manichaean Heth joining group.
	#[parse_name("MANICHAEAN HETH")]
	ManichaeanHeth,
	/// Manichaean Nun joining group.
	#[parse_name("MANICHAEAN NUN")]
	ManichaeanNun,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum NonJoiningGroup {
	/// No joining group.
	#[parse_name("No_Joining_Group")]
	None,
	/// Malayalam Ja joining group.
	#[parse_name("MALAYALAM JA")]
	MalayalamJa,
	/// Malayalam Bha joining group.
	#[parse_name("MALAYALAM BHA")]
	MalayalamBha,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum JoiningType {
	RightJoining(RightJoiningGroup),
	LeftJoining(LeftJoiningGroup),
	DualJoining(DualJoiningGroup),
	JoinCausing,
	NonJoining(NonJoiningGroup),
	Transparent,
}

impl JoiningType {
	pub fn parse(ty: &str, group: &str) -> Self {
		match ty {
			"R" => Self::RightJoining(RightJoiningGroup::parse(group).unwrap()),
			"L" => Self::LeftJoining(LeftJoiningGroup::parse(group).unwrap()),
			"D" => Self::DualJoining(DualJoiningGroup::parse(group).unwrap()),
			"C" => { assert!(group == "No_Joining_Group"); Self::JoinCausing },
			"U" => Self::NonJoining(NonJoiningGroup::parse(group).unwrap()),
			"T" => { assert!(group == "No_Joining_Group"); Self::Transparent },
			_ => unreachable!()
		}
	}
}

///  Bidirectional bracket orientation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BidiBracketOrientation {
	Open,
	Close
}

#[derive(Clone, Copy, Debug)]
pub struct BidiBracket {
	pub orientation: BidiBracketOrientation,
	pub opposite:    char,
}

#[derive(Clone, Copy, Debug)]
pub struct EmojiSource {
	/// Charcter code.
	pub code:     (u32, u32),
	/// DoCoMo Shift-JIS code.
	pub docomo:   Option<u32>,
	/// KDDI Shift-JIS code.
	pub kddi:     Option<u32>,
	///SoftBank Shift-JIS code.
	pub softbank: Option<u32>,
}


/// Syllable type of a hangul character
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum HangulSyllableType {
	///  Leading Jamo.
	#[parse_name("L")]
	Leading,
	// Vowel Jamo.
	#[parse_name("V")]
	Vowel,
	/// Trailing Jamo.
	#[parse_name("T")]
	Trailing,
	/// <L, V> sequence (a Leading Jamo followed by a Vowel Jamo)
	#[parse_name("LV")]
	LvSyllable,
	/// <LV, T> sequence (LV Syllable. followed by a Trailing Jamo)
	#[parse_name("LVT")]
	LvtSyllable,
}

/// Indic positional category
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum IndicPositionalCategory {
	/// Right positional.
	Right,
	/// Left positional
	Left,
	/// Dependent vowels that occur to th left of a consonant.
	#[parse_name("Visual_Order_Left")]
	VisualOrderLeft,
	/// Left and right positionals.
	#[parse_name("Left_And_Right")]
	LeftAndRight,
	/// Top positional.
	Top,
	/// Bottom positional.
	Bottom,
	/// Top and Bottom positionals.
	#[parse_name("Top_And_Bottom")]
	TopAndBottom,
	/// Top and Right positionals.
	#[parse_name("Top_And_Right")]
	TopAndRight,
	/// Top and Left positionals.
	#[parse_name("Top_And_Left")]
	TopAndLeft,
	/// Top, Left, and Right positionals.
	#[parse_name("Top_And_Left_And_Right")]
	TopAndLeftAndRight,
	/// Bottom and Right positionals.
	#[parse_name("Bottom_And_Right")]
	BottomAndRight,
	/// Bottom and Left positionals.
	#[parse_name("Bottom_And_Left")]
	BottomAndLeft,
	/// Top, Bottom, and Left positionals.
	#[parse_name("Top_And_Bottom_And_Right")]
	TopAndBottomAndLeft,
	/// Top, Bottom, and Right positionals.
	#[parse_name("Top_And_Bottom_And_Left")]
	TopAndBottomAndRight,
	/// Overstuck
	Overstruck,
}

/// Indic Syllabic Category
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum IndicSyllabicCategory {
	/// Bindu/Anusvara (nasalization or -n)
	Bindu,
	/// Visarga (-h).
	/// 
	/// Excludes letters for Jihvamuliya and Upadhmaniya, which are related, but structured somewhat differently.
	Visarga,
	/// Avagraha (elision of initial a- in sandhi).
	Avagraha,
	/// Nukta (diacritic for borrowed consonants or other consonant modfications).
	/// 
	/// Note that while the resulting sound is typically a consonant, the base letter a nukta follows may be an independent vowel.
	/// For example <U+0A85 GUJARATI LETTER A, U+0AFD GUJARATI SIGN TRHEE-DOT NUKTA ABOVE> is used to transcribe ARABIC LETTER AIN.
	Nukta,
	/// Virama (killing of inherent vowel in consonant sequence or consonant stacker).
	/// 
	/// Only includes characters that can act both as visible killer viramas and socnosnat stackers.
	/// Separate property values exist for characters that can only act as pure killers or only as consontant stackers.
	Virama,
	/// Pure killer (killing of inherent vowel in consonant sequence, with no consonant stacking behavior).
	#[parse_name("Pure_Killer")]
	PureKiller,
	/// Invisible stacker (invisible consonant stacker virama).
	/// 
	/// Note that in some scripts, such as Kharoshthi and Masaram Gondi, an invisible stacker may have a second function, 
	/// changing the shape and/or location of hte consonant preceding it, even where there is no consonant following the invisible stacker.
	#[parse_name("Invisible_Stacker")]
	InvisibleStacker,
	/// Independent Vowels (contrasted with matras).
	#[parse_name("Vowel_Independent")]
	VowelIndependent,
	/// Fependent Vowels (constrasted with independing vowelsand/or with complex placement).
	/// 
	/// Known as matras in Indix scripts.
	/// Also inclused vowel modifiers thta follow dependent (and sometimes independent) vowels.
	#[parse_name("Vowel_Dependent")]
	VowelDependent,
	/// (Other) Vowels (reanalyzed as ordinary alphabetic letters or marks).
	Vowel,
	/// Consonant Placeholder.
	/// 
	/// This includes generic placeholders used for Incid script layout (NBSP and dotted circle), 
	/// as well as a few script-specific vowel-holder characters which are not technically consonants, 
	/// but serve instead as bases for placement of vowel marks.
	#[parse_name("Consonant_Placeholder")]
	ConsonantPlaceholder,
	/// Consonant (ordinary abugida consonants, with inherent vowels).
	Consonant,
	/// Dead Consonant (special consonant with killed vowel).
	#[parse_name("Consonant_Dead")]
	ConsonantDead,
	/// Consonant that may make stacked ligatures with the next dconsonant with the use of a virama.
	#[parse_name("Consonant_With_Stacker")]
	ConsonantWithStacker,
	/// Clusre-initial consonants.
	#[parse_name("Consonant_Prefixed")]
	ConsonantPrefixed,
	/// Repha form of RA (reasnalyzed in some scripts), when preceding the main consonant.
	#[parse_name("Consonant_Preceding_Repha")]
	ConsonantPrecedingRepha,
	/// Consonatns taht succeed the main consonant in characte sequences, but are pronounced before it.
	#[parse_name("Consonant_Initial_Postfixed")]
	ConsonantInitialPostfixed,
	/// Repha form of RA (reanalyzed in some scrips), when succeeding the main consonant.
	#[parse_name("Consonant_Succeeding_Repha")]
	ConsonantSucceedingRepha,
	/// Subjoined Consonant (C2 form subtending a base consonant in Tibetan, etc).
	#[parse_name("Consonant_Subjoined")]
	ConsonantSubjoined,
	/// Medial Consonant (medial liquid, occuring in clusters).
	#[parse_name("Consonant_Medial")]
	ConsonantMedial,
	/// Final Consonant (special final forms which do not take vowels).
	#[parse_name("Consonant_Final")]
	ConsonantFinal,
	/// Head letter (Tibetan).
	#[parse_name("Consonant_Head_Letter")]
	ConsonantHeadLetter,
	/// Reanalyzed not participating in the abugida structre, but serving to modify the sound of an adjacent vowel or consonant.
	/// 
	/// Note that this is no the same as General Category == ModifierLetter.
	#[parse_name("Modifying_Letter")]
	ModifyingLetter,
	/// Tone letter (spacing lexical tone mark with status as a letter).
	#[parse_name("Tone_Letter")]
	ToneLetter,
	/// Tone MArk (nonspacing or spacing lecixal tone mark).
	#[parse_name("Tone_Mark")]
	ToneMark,
	/// Gemination Mark (doubling of the preceidng or following consonant).
	/// 
	/// U+0A61 GURMUKHI ADDAAK precedes the consonant if geminates, while the others follow the consonant the geminate.
	#[parse_name("Gemination_Mark")]
	GeminationMark,
	/// Cantillation Mark (recitation arks, such as svara markers from the Samaveda).
	#[parse_name("Cantillation_Mark")]
	CantillationMark,
	/// Register Shifer (shifts register for consonants, akin to a tone marker).
	#[parse_name("Register_Shifter")]
	RegisterShifter,
	/// Syllable Modifier (miscellaneous combining character that modify something in the orthographic syllable they secceed or appear in).
	#[parse_name("Syllable_Modifier")]
	SyllableModifier,
	/// Consonant Killer (signifies that the previous consonant or consonants are not pronounced).
	#[parse_name("Consonant_Killer")]
	ConsonantKiller,
	/// Non-joiner (zero width non-joiner).
	#[parse_name("Non_Joiner")]
	NonJoiner,
	/// Joiner (zero width jointer).
	Joiner,
	/// Number jointer (froms ligarutes between numbers for multiplication).
	#[parse_name("Number_Joiner")]
	NumberJoiner,
	/// Number (can be used as vowel-holders like consonant placeholders).
	/// 
	/// Note: A number may even hold subjoined consonants which may in turn have been formed using a virama or a stacker,
	/// e.g. the sequence <U+1A93, U+1A60, U+1A34> where THAI THAM LETTER LOW TA is subjoined to TAI THAM THAM DIGIT THREE using an invisible stacker.
	Number,
	/// Brahmi Joining NUmber (may be joined by a Number_Joiner of the same scripts, e.g. in Brahmi).
	/// 
	/// Note: These are different from Numbers, in the way tha tthere is no known evidence of Brahmi Joining Numbers taking vowels or subjoined constants.
	/// Until such evidence is found, implementations may assume that Brahmi Joining NUmber only participate in shaping with other Brahmi Joining Numbers.
	#[parse_name("Brahmi_Joining_Number")]
	BrahmiJoiningNumber,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum JamoShortName {
	/// Hangul Choseong Ieung.
	#[parse_name("")]
	Blank,
	/// Hangul Choseong/Jungseong Kiyeok.
	G,
	/// Hangul Choseong/Jungseong Ssangkiyeok.
	GG,
	/// Hangul Choseong/Jungseong Nieun.
	N,
	/// Hangul Choseong/Jungseong Tikeut.
	D,
	/// Hangul Choseong Ssangtikeut.
	DD,
	/// Hangul Choseong Rieul.
	R,
	/// Hangul Choseong/Jungseong Mieum.
	M,
	/// Hangul Choseong/Jungseong Pieup.
	B,
	/// Hangul Choseong Ssangpieup.
	BB,
	/// Hangul Choseong/Jungseong Sios.
	S,
	/// Hangul Choseong/Jungseong Ssangsios.
	SS,
	/// Hangul Choseong/Jungseong Cieuc.
	J,
	/// Hangul Choseong Ssangcieuc.
	JJ,
	/// Hangul Choseong/Jungseong Chieuch.
	C,
	/// Hangul Choseong/Jungseong Khieukh.
	K,
	/// Hangul Choseong/Jungseong Thieuth.
	T,
	/// Hangul Choseong/Jungseong Phieuph
	P,
	/// Hangul Choseong/Jungseong Hieuh.
	H,
	/// Hangul Jungseong A.
	A,
	/// Hangul Jungseong Ae.
	AE,
	/// Hangul Jungseong Ya.
	YA,
	/// Hangul Jungseong Yae.
	YAE,
	/// Hangul Jungseong Eo.
	EO,
	/// Hangul Jungseong E.
	E,
	/// Hangul Jungseong Yeo.
	YEO,
	/// Hangul Jungseong Ye.
	YE,
	/// Hangul Jungseong O.
	O,
	/// Hangul Jungseong Wa.
	WA,
	/// Hangul Jungseong Wae.
	WAE,
	/// Hangul Jungseong Oe.
	OE,
	/// Hangul Jungseong Yo.
	YO,
	/// Hangul Jungseong U.
	U,
	/// Hangul Jungseong Weo.
	WEO,
	/// Hangul Jungseong We.
	WE,
	/// Hangul Jungseong Wi.
	WI,
	/// Hangul Jungseong Yu.
	YU,
	/// Hangul Jungseong Eu.
	EU,
	/// Hangul Jungseong Yi.
	YI,
	/// Hangul Jungseong I.
	I,
	/// Hangul Jungseong Kiyeok-Sios.
	GS,
	/// Hangul Jungseong Nieun-Cieuc.
	NJ,
	/// Hangul Jungseong Nieun-Hieuh.
	NH,
	/// Hangul Jungseong Rieul.
	L,
	/// Hangul Jungseong Rieul-Kiyeok
	LG,
	/// Hangul Jungseong Rieul-Mieum.
	LM,
	/// Hangul Jungseong Rieul-Pieup.
	LB,
	/// Hangul Jungseong Rieul-Sios.
	LS,
	/// Hangul Jungseong Rieul-Thieuth.
	LT,
	/// Hangul Jungseong Rieul-Phieuph.
	LP,
	/// Hangul Jungseong Rieul-Hieuh.
	LH,
	/// Hangul Jungseong Pieup-Sios.
	BS,
	/// Hangul Jungseong Ieung.
	NG,
}

/// Line break.
/// 
/// THe maing of each class (annotation between parenthises) can be one of the following values:
/// - (A): Allow a break oppertunity after in specified contexts
/// - (XA): Prevents a break oppertunity after in specified contexts
/// - (B): Allow a break oppertunity before in specified contexts
/// - (XB): Prevents a break oppertunity before in specified contexts
/// - (P): Allow a break oppertunity for a pair in specified contexts
/// - (XP): Prevents a break oppertunity for a pair in specified contexts
/// 
/// For additional info, see: https://www.unicode.org/reports/tr14/
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum LineBreak {
	/// Amibguous (Alphebetic or Ideograph).
	AI,
	/// Aksara (XB/XA).
	AK,
	/// Ordinary alphabetic and symbolic characters (XP).
	AL,
	/// Aksara pre-base (B/XA).
	AP,
	/// Aksara start (XB/XA).
	AS,
	/// Break after (A).
	BA,
	/// Break before (B).
	BB,
	/// Break oppertunity before and after (B/A/XP).
	B2,
	/// Mnadatory break (A) (Non-tailorable).
	BK,
	/// Contingent break opportunity (B/A).
	CB,
	/// Conditional Japanses starter.
	CJ,
	/// Close Punctuation (XB).
	CL,
	/// Combining Mark (XB) (Non-tailorable).
	CM,
	/// Closing Parenthesis (XB).
	CP,
	/// Carriage return (A) (Non-tailorable).
	CR,
	/// Emoji base (B/A).
	EB,
	/// Emoji Modifier (A).
	EM,
	/// Exclamation/Interrogation (XB).
	EX,
	/// Non-breaking ("Glue") (XB/XA) (Non-tailorable).
	GL,
	/// Hangul LV syllable (B/A).
	H2,
	/// Hangul LVT syllable (B/A).
	H3,
	/// Hyphen (XA).
	HY,
	/// Ideographic (B/A).
	ID,
	/// Hebrew Letter (XB).
	HL,
	/// Inseparable characters (XP).
	IN,
	/// Infux numeric separator (XB.
	IS,
	/// Hangul L Jamo (B).
	JL,
	/// Hangul T Jamo (A).
	JT,
	/// Hangul V Jamo (XA/XB).
	JV,
	/// Life feed (A) (Non-tailorable).
	LF,
	/// Next line (A) (Non-tailorable).
	NL,
	/// Nonstarters (XB).
	NS,
	/// Numeric (XP).
	NU,
	/// Open Punctuation (XA).
	OP,
	/// Postfix numeric (XB).
	PO,
	/// Prefix numeric (CA).
	PR,
	/// Quotation (XB/XA).
	QU,
	/// Regional indicator (B/A/XP).
	RI,
	/// Complex-context dependent (South east asian) (P).
	SA,
	/// Surrogate (XP) (Non-tailorable).
	SG,
	/// Space (A) (Non-tailorable).
	SP,
	/// Symbols allowing break after (A).
	SY,
	/// Virama final(XB/A).
	VF,
	/// Virama (XB/XA).
	VI,
	/// Word jointer (XB/XA) (Non-tailorable)
	WJ,
	/// Unknown (XP).
	XX,
	/// Zero width space (A) (Non-tailorable).
	ZW,
	/// Zero width jointer (XA/XB) (Non-tailorable).
	ZWJ,
}

/// Grapheme cluster break.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum GraphemeClusterBreak {
	/// Carriage return
	CR,
	/// Line feed
	LF,
	Control,
	Extend,
	/// Zero width jointer
	ZWJ,
	#[parse_name("Regional_Indicator")]
	RegionalIndicator,
	Prepend,
	SpacingMark,
	/// Hangul L syllable.
	L,
	/// Hangul V syllable.
	V,
	/// Hangul T syllable.
	T,
	/// Hangul LV syllable.
	LV,
	/// Hangul LVT syllable.
	LVT,
	/// This is not a property value.
	/// 
	/// It is used in the reule to represnet any code point.
	Any,
}

/// Sentence break.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum SentenceBreak {
	/// Carriage return.
	CR,
	/// Line feed.
	LF,
	Extend,
	Sep,
	Format,
	Sp,
	Lower,
	Upper,
	OLetter,
	Numeric,
	ATerm,
	SContinue,
	STerm,
	Close,
	/// This is not a property value.
	/// 
	/// It is used in the reule to represnet any code point.
	Any,
}

/// Word break.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum WordBreak {
	/// Carriage return
	CR,
	/// Line feed
	LF,
	Newline,
	Extend,
	/// Zero width joiner.
	ZWJ,
	#[parse_name("Regional_Indicator")]
	RegionalIndicator,
	Format,
	Katakana,
	#[parse_name("Hebrew_Letter")]
	HebrewLetter,
	ALetter,
	#[parse_name("Single_Quote")]
	SingleQuote,
	#[parse_name("Double_Quote")]
	DoubleQuote,
	MidNumLet,
	MidLetter,
	MidNum,
	Numeric,
	ExtendNumLet,
	WSegSpace,
	/// This is not a property value.
	/// 
	/// It is used in the reule to represnet any code point.
	Any
}

/// Scripts
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum Script {
	Unknown,
	/// To see what scripts contain this character, check the `script_extension` property of the unicode character.
	Common,
	Latin,
	Greek,
	Cyrillic,
	Armenian,
	Hebrew,
	Arabic,
	Syriac,
	Thaana,
	Devanagari,
	Bengali,
	Gurmukhi,
	Gujarati,
	Oriya,
	Tamil,
	Telugu,
	Kannada,
	Malayalam,
	Sinhala,
	Thai,
	Lao,
	Tibetan,
	Myanmar,
	Georgian,
	Hangul,
	Ethiopic,
	Cherokee,
	#[parse_name("Canadian_Aboriginal")]
	CanadianAboriginal,
	Ogham,
	Runic,
	Khmer,
	Mongolian,
	Hiragana,
	Katakana,
	Bopomofo,
	Han,
	Yi,
	#[parse_name("Old_Italic")]
	OldItalic,
	Gothic,
	Deseret,
	/// To see what scripts contain this character, check the `script_extension` property of the unicode character.
	Inherited,
	Tagalog,
	Hanunoo,
	Buhid,
	Tagbanwa,
	Limbu,
	#[parse_name("Tai_Le")]
	TaiLe,
	#[parse_name("Linear_B")]
	LinearB,
	Ugaritic,
	Shavian,
	Osmanya,
	Cypriot,
	Braille,
	Buginese,
	Coptic,
	#[parse_name("New_Tai_Lue")]
	NewTaiLue,
	Glagolitic,
	Tifinagh,
	#[parse_name("Syloti_Nagri")]
	SylotiNagri,
	#[parse_name("Old_Persian")]
	OldPersian,
	Kharoshthi,
	Balinese,
	Cuneiform,
	Phoenician,
	#[parse_name("Phags_Pa")]
	PhagsPa,
	Nko,
	Sundanese,
	Lepcha,
	#[parse_name("Ol_Chiki")]
	OlChiki,
	Vai,
	Saurashtra,
	#[parse_name("Kayah_Li")]
	KayahLi,
	Rejang,
	Lycian,
	Carian,
	Lydian,
	Cham,
	#[parse_name("Tai_Tham")]
	TaiTham,
	#[parse_name("Tai_Viet")]
	TaiViet,
	Avestan,
	#[parse_name("Egyptian_Hieroglyphs")]
	EgyptianHieroglyphs,
	Samaritan,
	Lisu,
	Bamum,
	Javanese,
	#[parse_name("Meetei_Mayek")]
	MeeteiMayek,
	#[parse_name("Imperial_Aramaic")]
	ImperialAramaic,
	#[parse_name("Old_South_Arabian")]
	OldSouthArabian,
	#[parse_name("Inscriptional_Parthian")]
	InscriptionalParthian,
	#[parse_name("Inscriptional_Pahlavi")]
	InscriptionalPahlavi,
	#[parse_name("Old_Turkic")]
	OldTurkic,
	Kaithi,
	Batak,
	Brahmi,
	Mandaic,
	Chakma,
	#[parse_name("Meroitic_Cursive")]
	MeroiticCursive,
	#[parse_name("Meroitic_Hieroglyphs")]
	MeroiticHieroglyphs,
	Miao,
	Sharada,
	#[parse_name("Sora_Sompeng")]
	SoraSompeng,
	Takri,
	#[parse_name("Caucasian_Albanian")]
	CaucasianAlbanian,
	#[parse_name("Bassa_Vah")]
	BassaVah,
	Duployan,
	Elbasan,
	Grantha,
	#[parse_name("Pahawh_Hmong")]
	PahawhHmong,
	Khojki,
	#[parse_name("Linear_A")]
	LinearA,
	Mahajani,
	Manichaean,
	#[parse_name("Mende_Kikakui")]
	MendeKikakui,
	Modi,
	Mro,
	#[parse_name("Old_North_Arabian")]
	OldNorthArabian,
	Nabataean,
	Palmyrene,
	#[parse_name("Pau_Cin_Hau")]
	PauCinHau,
	#[parse_name("Old_Permic")]
	OldPermic,
	#[parse_name("Psalter_Pahlavi")]
	PsalterPahlavi,
	Siddham,
	Khudawadi,
	Tirhuta,
	#[parse_name("Warang_Citi")]
	WarangCiti,
	Ahom,
	#[parse_name("Anatolian_Hieroglyphs")]
	AnatolianHieroglyphs,
	Hatran,
	Multani,
	#[parse_name("Old_Hungarian")]
	OldHungarian,
	SignWriting,
	Adlam,
	Bhaiksuki,
	Marchen,
	Newa,
	Osage,
	Tangut,
	#[parse_name("Masaram_Gondi")]
	MasaramGondi,
	Nushu,
	Soyombo,
	#[parse_name("Zanabazar_Square")]
	ZanabazarSquare,
	Dogra,
	#[parse_name("Gunjala_Gondi")]
	GunjalaGondi,
	Makasar,
	Medefaidrin,
	#[parse_name("Hanifi_Rohingya")]
	HanifiRohingya,
	Sogdian,
	#[parse_name("Old_Sogdian")]
	OldSogdian,
	Elymaic,
	Nandinagari,
	#[parse_name("Nyiakeng_Puachue_Hmong")]
	NyiakengPuachueHmong,
	Wancho,
	Chorasmian,
	#[parse_name("Dives_Akuru")]
	DivesAkuru,
	#[parse_name("Khitan_Small_Script")]
	KhitanSmallScript,
	Yezidi,
	#[parse_name("Cypro_Minoan")]
	CyproMinoan,
	#[parse_name("Old_Uyghur")]
	OldUyghur,
	Tangsa,
	Toto,
	Vithkuqi,
	Kawi,
	#[parse_name("Nag_Mundari")]
	NagMundari,
	Sindhi,
}

impl Script {
	pub fn from_short_name(s: &str) -> Option<Self> {
		match s {
			"Zinh" => Some(Self::Inherited),
			"Latn" => Some(Self::Latin),
			"Grek" => Some(Self::Greek),
			"Cyrl" => Some(Self::Cyrillic),
			"Armn" => Some(Self::Armenian),
			"Hebr" => Some(Self::Hebrew),
			"Arab" => Some(Self::Arabic),
			"Syrc" => Some(Self::Syriac),
			"Thaa" => Some(Self::Thaana),
			"Deva" => Some(Self::Devanagari),
			"Beng" => Some(Self::Bengali),
			"Guru" => Some(Self::Gurmukhi),
			"Gujr" => Some(Self::Gujarati),
			"Orya" => Some(Self::Oriya),
			"Taml" => Some(Self::Tamil),
			"Telu" => Some(Self::Telugu),
			"Knda" => Some(Self::Kannada),
			"Mlym" => Some(Self::Malayalam),
			"Sinh" => Some(Self::Sinhala),
			"Thai" => Some(Self::Thai),
			"Laoo" => Some(Self::Lao),
			"Tibt" => Some(Self::Tibetan),
			"Mymr" => Some(Self::Myanmar),
			"Geor" => Some(Self::Georgian),
			"Hang" => Some(Self::Hangul),
			"Ethi" => Some(Self::Ethiopic),
			"Cher" => Some(Self::Cherokee),
			"Cans" => Some(Self::CanadianAboriginal),
			"Ogam" => Some(Self::Ogham),
			"Runr" => Some(Self::Runic),
			"Khmr" => Some(Self::Khmer),
			"Mong" => Some(Self::Mongolian),
			"Hira" => Some(Self::Hiragana),
			"Kana" => Some(Self::Katakana),
			"Bopo" => Some(Self::Bopomofo),
			"Hani" => Some(Self::Han),
			"Yiii" => Some(Self::Yi),
			"Ital" => Some(Self::OldItalic),
			"Goth" => Some(Self::Gothic),
			"Dsrt" => Some(Self::Deseret),
			"Tglg" => Some(Self::Tagalog),
			"Hano" => Some(Self::Hanunoo),
			"Buhd" => Some(Self::Buhid),
			"Tagb" => Some(Self::Tagbanwa),
			"Limb" => Some(Self::Limbu),
			"Tale" => Some(Self::TaiLe),
			"Linb" => Some(Self::LinearB),
			"Ugar" => Some(Self::Ugaritic),
			"Shaw" => Some(Self::Shavian),
			"Osma" => Some(Self::Osmanya),
			"Cprt" => Some(Self::Cypriot),
			"Brai" => Some(Self::Braille),
			"Budi" => Some(Self::Buginese),
			"Copt" => Some(Self::Coptic),
			"Talu" => Some(Self::NewTaiLue),
			"Glag" => Some(Self::Glagolitic),
			"Tgnh" => Some(Self::Tifinagh),
			"Sylo" => Some(Self::SylotiNagri),
			"Xpeo" => Some(Self::OldPersian),
			"Khar" => Some(Self::Kharoshthi),
			"Bali" => Some(Self::Balinese),
			"Xsux" => Some(Self::Cuneiform),
			"Phnx" => Some(Self::Phoenician),
			"Phag" => Some(Self::PhagsPa),
			"Nkoo" => Some(Self::Nko),
			"Sunb" => Some(Self::Sundanese),
			"Lepc" => Some(Self::Lepcha),
			"Olch" => Some(Self::OlChiki),
			"Vaii" => Some(Self::Vai),
			"Saur" => Some(Self::Saurashtra),
			"Kali" => Some(Self::KayahLi),
			"Rjng" => Some(Self::Rejang),
			"Lyci" => Some(Self::Lycian),
			"Cari" => Some(Self::Carian),
			"Lydi" => Some(Self::Lydian),
			"Cham" => Some(Self::Cham),
			"Lana" => Some(Self::TaiTham),
			"Tavt" => Some(Self::TaiViet),
			"Avst" => Some(Self::Avestan),
			"Egyp" => Some(Self::EgyptianHieroglyphs),
			"Samr" => Some(Self::Samaritan),
			"Lisu" => Some(Self::Lisu),
			"Bamu" => Some(Self::Bamum),
			"Java" => Some(Self::Javanese),
			"Mtei" => Some(Self::MeeteiMayek),
			"Armi" => Some(Self::ImperialAramaic),
			"Sarb" => Some(Self::OldSouthArabian),
			"Prti" => Some(Self::InscriptionalParthian),
			"Phli" => Some(Self::InscriptionalPahlavi),
			"Orkh" => Some(Self::OldTurkic),
			"Kthi" => Some(Self::Kaithi),
			"Batk" => Some(Self::Batak),
			"Brah" => Some(Self::Brahmi),
			"Mand" => Some(Self::Mandaic),
			"Cakm" => Some(Self::Chakma),
			"Merc" => Some(Self::MeroiticCursive),
			"Mero" => Some(Self::MeroiticHieroglyphs),
			"Plrd" => Some(Self::Miao),
			"Shrd" => Some(Self::Sharada),
			"Sora" => Some(Self::SoraSompeng),
			"Takr" => Some(Self::Takri),
			"Aghb" => Some(Self::CaucasianAlbanian),
			"Bass" => Some(Self::BassaVah),
			"Dupl" => Some(Self::Duployan),
			"Elba" => Some(Self::Elbasan),
			"Gran" => Some(Self::Grantha),
			"Hmng" => Some(Self::PahawhHmong),
			"Khoj" => Some(Self::Khojki),
			"Lina" => Some(Self::LinearA),
			"Mahj" => Some(Self::Mahajani),
			"Mani" => Some(Self::Manichaean),
			"Mend" => Some(Self::MendeKikakui),
			"Modi" => Some(Self::Modi),
			"Mroo" => Some(Self::Mro),
			"Narb" => Some(Self::OldNorthArabian),
			"Nbat" => Some(Self::Nabataean),
			"Palm" => Some(Self::Palmyrene),
			"Pauc" => Some(Self::PauCinHau),
			"Perm" => Some(Self::OldPermic),
			"Phlp" => Some(Self::PsalterPahlavi),
			"Sidd" => Some(Self::Siddham),
			"Sind" => Some(Self::Khudawadi),
			"Tirh" => Some(Self::Tirhuta),
			"Wara" => Some(Self::WarangCiti),
			"Ahom" => Some(Self::Ahom),
			"Hluw" => Some(Self::AnatolianHieroglyphs),
			"Hatr" => Some(Self::Hatran),
			"Mult" => Some(Self::Multani),
			"Hung" => Some(Self::OldHungarian),
			"Sgnw" => Some(Self::SignWriting),
			"Adlm" => Some(Self::Adlam),
			"Bhks" => Some(Self::Bhaiksuki),
			"Marc" => Some(Self::Marchen),
			"Newa" => Some(Self::Newa),
			"Osge" => Some(Self::Osage),
			"Tang" => Some(Self::Tangut),
			"Gonm" => Some(Self::MasaramGondi),
			"Nshu" => Some(Self::Nushu),
			"Soyo" => Some(Self::Soyombo),
			"Zanb" => Some(Self::ZanabazarSquare),
			"Dogr" => Some(Self::Dogra),
			"Gong" => Some(Self::GunjalaGondi),
			"Maka" => Some(Self::Makasar),
			"Medf" => Some(Self::Medefaidrin),
			"Hmnp" => Some(Self::NyiakengPuachueHmong),
			"Wcho" => Some(Self::Wancho),
			"Kits" => Some(Self::KhitanSmallScript),
			"Cpmn" => Some(Self::CyproMinoan),
			_      => None
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum VerticalOrientation {
	/// Characters are upright, same orienctation as in the code pages.
	#[parse_name("U")]
	Upright,
	/// Rotated 90 degrees clockwise compared to the code chart.
	#[parse_name("R")]
	Rotated,
	/// Characters aer not just upright or sideways, but generally require a different glyph that in the code chart when used in vertical texts.
	/// In additiona, as a fallback, the caharacter can be dispalyed with th code chart glyph upright.
	#[parse_name("Tu")]
	TransformedUpright,
	/// Characters aer not just upright or sideways, but generally require a different glyph that in the code chart when used in vertical texts.
	/// In additiona, as a fallback, the caharacter can be dispalyed with th code chart glyph 90 degrees rotated.
	#[parse_name("Tr")]
	TransformedRotated,
}


/// Character decomposition mapping
#[derive(Clone, Copy, Debug)]
pub enum CharacterDecomposition {
	None,
	Normal(&'static [u32]),
	Font(u32),
	NoBreak(u32),
	Initial(&'static [u32]),
	Medial(&'static [u32]),
	Final(&'static [u32]),
	Isolated(&'static [u32]),
	Circle(&'static [u32]),
	Super(u32),
	Sub(u32),
	Vertical(&'static [u32]),
	Wide(u32),
	Narrow(u32),
	Small(u32),
	Square(&'static [u32]),
	Fraction(&'static [u32]),
	Compat(&'static [u32]),
}

/// Joining info.
#[derive(Clone, Copy, Debug)]
pub struct JoiningInfo {
	/// Schematic name
	pub name:      &'static str,
	/// Joinig type and group
	pub join_type: JoiningType,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Casing {
	Simple(char),
	Complex(&'static [char]),
	Conditional(&'static [(&'static str, &'static [char])]),
}

#[flags(parse_from_name)]
pub enum DerivedCoreProperty {
	/// Characters with a lowercase property.
	///
	/// Generated from: `category=Ll + flag=Other_Lowercase`
	Lowercase,
	/// Charcters with an uppercase property.
	///
	/// Generated from: `category=Ll + flag=Other_Uppercase`
	Uppercase,
	/// Characters which are considered to be either uppercase, lowercase, or titlecase characters.
	///
	/// Generated from: `Lowercase + Uppercase + category=Lt`
	Cased,
	/// Characters which are ignored for casing pruposes.
	/// 
	/// Generated from `category=[Mn + Me + Cf + Lm + Sk] + word_break=[MidLetter + MinNumLet + SignleQuote]`
	#[parse_name("Case_Ignorable")]
	CaseIgnorable,
	/// Characters whose normalized forms are not stabel under a to_lower mapping.
	///
	/// Generated from: `toLowercase(toNFD(X)) != toNFD(X)`
	#[parse_name("Changes_When_Lowercased")]
	ChangesWhenLowercased,
	/// Characters whose normalized forms are not stable under a to_upper mapping.
	///
	/// Generated from: `toUppercase(toNFD(X)) != toNFD(X)`
	#[parse_name("Changes_When_Uppercased")]
	ChangesWhenUppercased,
	/// Characters whose normalized forms are not stable under a to_title mapping.
	///
	/// Generated from: `toTitlcase(toNFD(X)) != toNFD(X)`
	#[parse_name("Changes_When_Titlecased")]
	ChangesWhenTitlecased,
	/// Characters whose normalized forms are not stable under a to_title mapping.
	///
	/// Generated from: `toCasefold(toNFD(X)) != toNFD(X)`
	#[parse_name("Changes_When_Casefolded")]
	ChangesWhenCasefolded,
	/// Characters which may chagne when they undergo case mapping
	///
	/// Generated from: `ChagnesWhenLowercased + ChangesWhenUppercased + ChangesWhenTitlecased`
	#[parse_name("Changes_When_Casemapped")]
	ChangesWhenCasemapped = ChangesWhenLowercased | ChangesWhenUppercased | ChangesWhenTitlecased,
	/// Characters with the Alphabetic property.
	///
	/// Generated from: `Lowercase + Uppercase + Category=[Lt + Lm + Lo + Nl] + flag=Other_Alphabetic`
	Alphabetic,
	/// For programmatic determination of default ignorable code points.
	/// New characters that should be ignored in rendering (unless explicitly supported) will be assigned in these ranges,
	/// permitting programs to correctly handle the default rendering of such characters when not otherwise supported.
	///
	/// Generated from:
	/// ```ignored
	/// 	flags=[
	/// 		Other_Default_Ignorable_Code_Point
	/// 		+ Variation_Selector
	/// 		- White_Space
	/// 		- Prepended_Concatenation_Mark (Exceptional format characters that should be visible)
	/// 	]
	/// 	+ category=Cf
	/// 	- FFF9..=FFB (Interlinear annotation format characters)
	/// 	- 13430..=1343F (Egyptian hieroglyph format characters)
	/// ```
	#[parse_name("Default_Ignorable_Code_Point")]
	DefaultIgnorableCodePoint,
	/// Property used together with the definition of Standard Korean Syllabled Block to ddefine "Grapheme base".
	///
	/// Generated from: `0..=10FFFF - Category=[Cc + Cf  k Cs + Co + Cn + Zl + Zp] - GraphemeExtend`
	///
	/// # Note
	///
	/// GraphemeBase is a propery of individual characters.
	/// The usage contrasts with "grapheme base", which is an attribute of Unicode strings;
	/// a grapheme base may consist of a Korean syllable which is itself represented by a sequence of conjoining jamos.
	#[parse_name("Grapheme_Base")]
	GraphemeBase,
	/// Property used to define "Grapheme extender".
	///
	/// Generated from `category=[Me + Mn] + flag=Other_Grapheme_Extend`
	///
	/// # Note
	///
	/// The set of character for which GraphemeExend=Yes is used in the derivation of the property value Grapheme_Cluser_Break=Extend.
	/// Grapheme_Cluser_Break consists of the set of charactes for which Grapheme_Extend=Yes or Emoji_Modifier=Yes.
	#[parse_name("Grapheme_Extend")]
	GraphemeExtend,
	/// Characters with the Math property.
	///
	/// Generated from: Category=Sm + flag=OtherMath
	Math,
	#[parse_name("ID_Start")]
	IdStart,
	#[parse_name("ID_Continue")]
	IdContinue,
	#[parse_name("XID_Start")]
	XidStart,
	#[parse_name("XID_Continue")]
	XidContinue,
}

/// Indic conjunction break.
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum IndicConjunctBreak {
	Linker,
	Consonant,
	Extend,
	None
}

//==============================================================
// Direct getters
//==============================================================


/// Get the name of a unicode codepoint, or `None`` when the codepoint is not valid or is part of the private use space.
pub fn get_name(codepoint: u32) -> Option<&'static str> {
	from_key(codepoint, &unicode::NAMES)
}

/// Get flags for the unicode codepoint.
pub fn get_flags(codepoint: u32) -> UnicodeFlags {
	from_index_or(codepoint, &unicode::FLAGS, UnicodeFlags::None)
}

/// Get the category for the unicode codepoint, or `None`` when the codepoint is not valid or is part of the private use space
pub fn get_category(codepoint: u32) -> Option<Category> {
	from_index(codepoint, &unicode::CATEGORIES)
}

/// Get the canonical combining class for the unicode codepoint.
///
/// The value returned by this function is not valid when an invalid codepoint is passed and a default value will be returned.
pub fn get_canonical_combining_class(codepoint: u32) -> CanonicalCombiningClass {
	from_index_or(codepoint, &unicode::CANONICAL_COMBINE_CLASSES, CanonicalCombiningClass::NotReordered)
}

/// Get the bidirectional class for the unicode codepoint.
///
/// The value returned by this function is not valid when an invalid codepoint is passed and a default value will be returned.
pub fn get_bidirectional_class(codepoint: u32) -> BidirectionalClass {
	from_index_or(codepoint, &unicode::BIDIRECTIONAL_CLASSES, BidirectionalClass::LeftToRight)
}

/// Get the decomposition mapping for a character.
pub fn get_character_decomposition(ch: char) -> Option<CharacterDecomposition> {
	from_index(ch as u32, &unicode::DECOMPOSITIONS)
}

/// Get the numeric value that is represented by the given character.
pub fn get_numeric_value(ch: char) -> Option<u8> {
	from_index(ch as u32, &unicode::NUMERIC_VALUES)
}

/// Get the digit value that is represented by the given character.
pub fn get_digit_value(ch: char) -> Option<u8> {
	from_index(ch as u32, &unicode::DIGIT_VALUES)
}

/// Get the rational value that is represented by the given character.
pub fn get_rational_value(ch: char) -> Option<Rational> {
	from_index(ch as u32, &unicode::RATIONAL_VALUES)
}

/// Get the character's bidirectionally mirrored representation, or `None` if there is no mirrored version.
pub fn get_bidirectional_mirrored(ch: char) -> Option<char> {
	from_key(ch, &unicode::BIDI_MIRRORING_GLYPH)
}

/// Get the lowercase representation of a character, if no lowercase representation exists, the character wil be returned.
pub fn to_lower(ch: char) -> Casing {
	from_key_or(ch, &unicode::TO_LOWER, Casing::Simple(ch))
}

/// Get the uppercase representation of a character, if no lowercase representation exists, the character wil be returned.
pub fn to_upper(ch: char) -> Casing {
	from_key_or(ch, &unicode::TO_UPPER, Casing::Simple(ch))
}

/// Get the titlecase representation of a character, if no lowercase representation exists, the character wil be returned.
pub fn to_title(ch: char) -> Casing {
	from_key_or(ch, &unicode::TO_TITLE, Casing::Simple(ch))
}

/// Get the joining info for a codepoint, or `None` if there is no joining info for the codepoint.
pub fn get_joining_info(codepoint: u32) -> Option<JoiningInfo> {
	from_index(codepoint, &unicode::JOINING_INFO)
}

/// Get the orientation of a bidirectional bracket.
pub fn get_bracket_orientation(ch: char) -> Option<BidiBracketOrientation> {
	from_key(ch, &unicode::BIDI_PAIRED_BRACKETS).map(|val| val.orientation)
}

/// Get the paired bracket
pub fn get_paired_bracket(ch: char) -> Option<char> {
	from_key(ch, &unicode::BIDI_PAIRED_BRACKETS).map(|val| val.opposite)
}

/// Get the name of the block containing the unicode codepoint.
pub fn get_block(codepoint: u32) -> &'static str {
	from_index_or(codepoint, &unicode::UNI_BLOCKS, "<invalid>")
}

/// Get the age (unicode version in which the codepoint was added) for a unicode codepoint.
pub fn get_age(codepoint: u32) -> Age {
	from_index_or(codepoint, &unicode::DERIVED_AGE, Age::Unknown)
}

/// Get the east-asian width of the unicode codepoint, or `None` if there is no east-asian width for the codepoint.
pub fn get_east_asian_width(codepoint: u32) -> Option<EastAsianWidth> {
	from_index(codepoint, &unicode::EAST_ASIAN_WIDTHS)
}

/// Get the equivalent unified ideograph for a character.
pub fn get_equivalent_unified_ideograph(ch: char) -> Option<char> {
	from_key(ch, &unicode::EQUVALENT_UNIFIED_IDEOGRAPHS)
}

/// Get the hangul syllable type of the unicode codepoint, or `None` if there is no type for the codepoint.
pub fn get_hangul_syllable_type(ch: char) -> Option<HangulSyllableType> {
	from_index(ch as u32, &unicode::HANGUL_SYLLABLE_TYPE)
}

/// Get the indic positional type of the unicode codepoint, or `None` if there is no type for the codepoint.
pub fn get_indic_positional_type(ch: char) -> Option<IndicPositionalCategory> {
	from_index(ch as u32, &unicode::INDIC_POSITIONAL_CATEGORIES)
}

/// Get the indic syllable type of the unicode codepoint, or `None` if there is no type for the codepoint.
pub fn get_indic_syllabic_type(ch: char) -> Option<IndicSyllabicCategory> {
	from_index(ch as u32, &unicode::INDIC_SYLLABIC_CATEGORIES)
}

/// Get the jamo short name for a Hangul syllable.
pub fn get_jamo_short_name(ch: char) -> Option<JamoShortName> {
	from_key(ch, &unicode::JAMO_SHORT_NAMES)
}

/// Get the line break of the unicode codepoint, or `None` if there is no break info for the codepoint.
pub fn get_line_break(ch: char) -> Option<LineBreak> {
	from_index(ch as u32, &unicode::LINE_BREAKS)
}

/// Get the grapheme cluster break of the unicode codepoint, or `None` if there is no break info for the codepoint.
pub fn get_grapheme_break(ch: char) -> Option<GraphemeClusterBreak> {
	from_index(ch as u32, &unicode::GRAPHEME_BREAKS)
}

/// Get the sentence break of the unicode codepoint, or `None` if there is no break info for the codepoint.
pub fn get_sentence_break(ch: char) -> Option<SentenceBreak> {
	from_index(ch as u32, &unicode::SENTENCE_BREAKS)
}

/// Get the word break of the unicode codepoint, or `None` if there is no break info for the codepoint.
pub fn get_word_break(ch: char) -> Option<WordBreak> {
	from_index(ch as u32, &unicode::WORD_BREAKS)
}

/// Get the script of the unicode codepoint, or `None` if there is no script info for the codepoint.
pub fn get_script(ch: char) -> Option<Script> {
	from_index(ch as u32, &unicode::SCRIPTS)
}

/// Get the script of the unicode codepoint, or `None` if there is no script info for the codepoint.
pub fn get_script_extensions(ch: char) -> Option<&'static [Script]> {
	from_index(ch as u32, &unicode::SCRIPT_EXTENSIONS)
}

/// Get the vertical orientation of the unicode codepoint.
///
/// The value returned by this function is not valid when an invalid codepoint is passed and a default value will be returned.
pub fn get_vertical_orientation(ch: char) -> VerticalOrientation {
	from_index_or(ch as u32, &unicode::VERTICAL_ORIENTATIONS, VerticalOrientation::Upright)
}

/// Get the Shift-JIS codes for emojis
pub fn get_emoji_sources(codepoint0: u32, codepoint1: u32) -> Option<EmojiSource> {
	match unicode::EMOJI_SOURCES.binary_search_by_key(&codepoint0, |val| val.code.0) {
	    Ok(idx) => for i in idx..unicode::EMOJI_SOURCES.len() {
			let source = &unicode::EMOJI_SOURCES[i];
			if source.code.0 != codepoint0 {
				return None;
			}

			if source.code.1 == codepoint1 {
				return Some(*source);
			}
		},
	    Err(_) => {},
	}
	None
}

//==============================================================
// Flags
//==============================================================

/// Is the character a "mirrored" in bidirectional text?
pub fn is_bidi_mirrored(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::BidiMirrored)
}

/// Is the character an emoji?
pub fn is_emoji(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::Emoji)
}

/// Is the character represented as an emoji by default?
pub fn is_emoji_presentation(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::EmojiPresentation)
}

/// Is the character an emoji modifier (e.g. skin tone modifier)?
pub fn is_emoji_modifier(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::EmojiModifier)
}

/// Is the character a base for emojis?
pub fn is_emoji_modifier_base(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::EmojiModifierBase)
}

/// Is the character an pictographic symbol?
pub fn is_extended_pictographic(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::ExtendedPictographic)
}

/// Is the character an ASCII character used to represent a hex digit?
pub fn is_ascii_hex_digit(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::AsciiHexDigit)
}

/// Does the character represent a specific function in the Unicode Bidrivetional Algorithm?
pub fn is_bidi_control(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::BidiControl)
}

/// Does the character represent a punctuation character explicitly called out as dashes in the Unicode Standard, plus their compatibility equivalents?
pub fn is_dash(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::Dash)
}

/// Is the unicode codepoint deprecated?
pub fn is_deprecated(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::Deprecated)
}

/// Is the unicode codepoint a diacritic?
/// i.e. a character that linguistically modify the meaning of another characterss to which they apply..
pub fn is_diacritic(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::Diacritic)
}

/// Is the unicode codepoint an extender?
/// i.e. a character whose principal function is to extend the value of a preceding alphabetic character or to extend the shape of adjacent characters.
pub fn is_extender(codepoint: u32) -> bool {	
	get_flags(codepoint).contains(UnicodeFlags::Extender)
}

/// Does the unicode codepoint repesent a hex digit, or their compatiblity equivalents?
pub fn is_hex_digit(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::HexDigit)
}

/// Is the character a CJKV (Chinenese, Japanese, Korean and Vietnamese) or other sinoform (Chinese writing-related) ideograph?
pub fn is_ideographic(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::HexDigit)
}

/// Does the unicode codepoint have a specific function for control of cursive joining and ligation?
pub fn is_join_control(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::JoinControl)
}

/// Does the unicode codepoint require special handling for processes like searching and sorting?
pub fn is_logical_order_exception(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::LogicalOrderException)
}

/// Is the unicode codepoint a visible format control, which precedes and then spans a sequence of other characters, usually digits?
pub fn is_prepended_concatination_mark(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::PrependedConcatenationMark)
}

/// Does the unicode codepoint function as a quotation mark?
pub fn is_quotation_mark(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::QuotationMark)
}

/// Does the unicode codepoint function a regional indicator?
pub fn is_regional_indicator(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::QuotationMark)
}

/// Does the unicode codepoint generally mark the end of a sentence?
pub fn is_sentence_terminal(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::SentenceTerminal)
}

/// Is the character a soft-dotted character?
pub fn is_soft_dotted(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::SoftDotted)
}

/// Does the unicode codepoint generally mark the end of a textual unit?
pub fn is_terminal_punctuation(codepoint: u32) -> bool {
	get_flags(codepoint).contains(UnicodeFlags::TerminalPunctuation)
}

/// Is the character a unified CJK ideograph?
pub fn is_unified_ideograph(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::UnifiedIdeograph)
}

/// Is the character treated as a whitespace in a programming alngauge for the purpose of parsing elements?
pub fn is_white_space(ch: char) -> bool {
	get_flags(ch as u32).contains(UnicodeFlags::WhiteSpace)
}

//==============================================================
// Derived properties
//==============================================================

/// Get the indic conjunt break.
pub fn get_indic_conjunct_break(codepoint: u32) -> IndicConjunctBreak {
	from_index_or(codepoint, &unicode::INDIC_CONJUNCT_BREAK, IndicConjunctBreak::None)
}

/// Get the derive property flags
pub fn get_derived_core_properties(codepoint: u32) -> DerivedCoreProperty {
	from_index_or(codepoint, &unicode::DERIVED_PROPS, DerivedCoreProperty::None)
}

/// Is the character lowercase?
pub fn is_lowercase(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::Lowercase)
}

/// Is the character uppercase?
pub fn is_uppercase(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::Uppercase)
}

/// Is the character cased? (lowercase, uppercase, or titlecase).
pub fn is_cased(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::Cased)
}

/// Is the character ignored for casing purposes
pub fn is_case_ignorable(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::CaseIgnorable)
}

/// Does the character change when lowercased, i.e. are the character's normalized forms not stable under a `to_lower` mapping?
pub fn changes_when_lowercased(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::ChangesWhenLowercased)
}

/// Does the character change when uppercased, i.e. are the character's normalized forms not stable under a `to_upper` mapping?
pub fn changes_when_uppercased(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::ChangesWhenUppercased)
}

/// Does the character change when titlecased, i.e. are the character's normalized forms not stable under a `to_title` mapping?
pub fn changes_when_titlecased(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::ChangesWhenTitlecased)
}

/// Does the character change when case folded, i.e. are the character's normalized forms not stable under case folding?
pub fn changes_when_casefolded(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::ChangesWhenCasefolded)
}

/// Does the character change when case mapped, i.e. are the character's normalized forms not stable under case mapping (using either 'to_lower', 'to_upper', or 'to_title')?
pub fn changes_when_casemapped(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::ChangesWhenCasefolded)
}

/// Is the character alphabetical.
pub fn is_alphabetic(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::CaseIgnorable)
}

/// For programmatic determination of default ignorable code points.
/// New characters that should be ignored in rendering (unless explicitly supported) will be assigned in these ranges,
/// permitting programs to correctly handle the default rendering of such characters when not otherwise supported.
pub fn is_default_ignorable_codepoint(codepoint: u32) -> bool {
	get_derived_core_properties(codepoint).contains(DerivedCoreProperty::DefaultIgnorableCodePoint)
}

/// Is the character a grapheme base?
pub fn is_grapheme_base(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::GraphemeBase)
}

/// Is the character a grapheme extender?
pub fn is_grapheme_extender(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::GraphemeExtend)
}

/// Is the character a math symbol?
pub fn is_math_symbol(ch: char) -> bool {
	get_derived_core_properties(ch as u32).contains(DerivedCoreProperty::Math)
}

//==============================================================
// Helpers
//==============================================================


fn from_index<T: Copy>(codepoint: u32, arr: &[(UnicodeIndex, T)]) -> Option<T> {
	match arr.binary_search_by(|val| val.0.partial_cmp(&codepoint).unwrap()) {
	    Ok(idx) => Some(arr[idx].1),
	    Err(_) => None,
	}
}

fn from_index_or<T: Copy>(codepoint: u32, arr: &[(UnicodeIndex, T)], default: T) -> T {
	match arr.binary_search_by(|val| val.0.partial_cmp(&codepoint).unwrap()) {
	    Ok(idx) => arr[idx].1,
	    Err(_) => default,
	}
}

fn from_key<K: Copy + Ord, T: Copy>(key: K, arr: &[(K, T)]) -> Option<T> {
	match arr.binary_search_by_key(&key, |val| val.0) {
	    Ok(idx) => Some(arr[idx].1),
	    Err(_) => None,
	}
}

fn from_key_or<K: Copy + Ord, T: Copy>(key: K, arr: &[(K, T)], default: T) -> T {
	match arr.binary_search_by_key(&key, |val| val.0) {
	    Ok(idx) => arr[idx].1,
	    Err(_) => default,
	}
}