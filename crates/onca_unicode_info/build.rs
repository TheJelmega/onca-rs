use core::fmt;
use std::{
	fmt::Debug, fs::File, io::{self, BufRead, BufReader, LineWriter, Write}, path::Path
};

use onca_base::{EnumFromIndexT, EnumFromNameT};
use onca_common_macros::{flags, EnumFromIndex, EnumFromName};

// Unicode index into info arrays
#[derive(Clone, Copy, PartialEq, Eq)]
enum UnicodeIndex {
	Single(u32),
	Range(u32, u32),
}

impl UnicodeIndex {
	fn merge(self, other: Self) -> Option<Self> {
		assert!(self < other);

		match self {
		    UnicodeIndex::Single(s) => match other {
    		    UnicodeIndex::Single(o) => if s + 1 == o {
					Some(Self::Range(s, o))
				} else {
					None
				},
    		    UnicodeIndex::Range(o_begin, o_end) => if s + 1 == o_begin {
					Some(Self::Range(s, o_end))
				} else {
					None
				}
    		},
		    UnicodeIndex::Range(s_begin, s_end) =>  match other {
    		    UnicodeIndex::Single(o) => if s_end + 1 == o {
					Some(Self::Range(s_begin, o))
				} else {
					None
				}
    		    UnicodeIndex::Range(o_begin, o_end) => if s_end + 1 == o_begin {
					Some(Self::Range(s_begin, o_end))
				} else {
					None
				},
    		},
		}
	}

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

	fn parse(s: &str) -> Self {
		if s.contains("..") {
			let mut elems = s.split("..");
			let begin = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			let end = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			Self::Range(begin, end)
		} else {
			let code = u32::from_str_radix(s, 16).unwrap();
			Self::Single(code)	
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
				if other < begin {
					Some(core::cmp::Ordering::Less)
				} else if other > end {
					Some(core::cmp::Ordering::Greater)
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
pub enum UnicodeCategory {
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
	Punctuation = ConnectorPunctuation | DashPunctuation | OpenPunctuation | ClosePunctuation | InitialPunctuation | FinalPunctuation | OpenPunctuation,
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
	/// The character is a mirrored character 
	BidiMirrored,
	/// The character is an emoji
	Emoji,
	/// The character should be rendered as an emoji, instead of text
	#[parse_name("Emoji_Presentation")]
	EmojiPresentation,
	/// 
	#[parse_name("Emoji_Modifier")]
	EmojiModifier,
	#[parse_name("Emoji_Modifier_Base")]
	EmojiModifierBase,
	#[parse_name("Emoji_Component")]
	EmojiComponent,
	#[parse_name("Extended_Pictographic")]
	ExtendedPictographic,
	/// The character is excluded from composition.
	CompositionExclusion,
	#[parse_name("ASCII_Hex_Digit")]
	AsciiHexDigit,
	#[parse_name("Bidi_Control")]
	BidiControl,
	Dash,
	Deprecated,
	Diacritic,
	Extender,
	#[parse_name("Hex_Digit")]
	HexDigit,
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
	#[parse_name("Join_Control")]
	JoinControl,
	#[parse_name("Logical_Order_Exception")]
	LogicalOrderException,
	#[parse_name("Noncharacter_Code_Point")]
	NoncharacterCodePoint,

	#[parse_name("Other_Alphabetic")]
	OtherAlphabetic,
	#[parse_name("Other_Default_Ignorable_Code_Point")]
	OtherDefaultIgnorableCodePoint,
	#[parse_name("Other_Grapheme_Extend")]
	OtherGraphemeExtend,
	#[parse_name("Other_ID_Continue")]
	OtherIdContinue,
	#[parse_name("Other_ID_Start")]
	OtherIdStart,
	#[parse_name("Other_Lowercase")]
	OtherLowercase,
	#[parse_name("Other_Math")]
	OtherMath,
	#[parse_name("Other_Uppercase")]
	OtherUppercase,
	#[parse_name("Pattern_Syntax")]
	PatternSyntax,
	#[parse_name("Pattern_White_Space")]
	PatternWhiteSpace,
	#[parse_name("Prepended_Concatenation_Mark")]
	PrependedConcatenationMark,
	#[parse_name("Quotation_Mark")]
	QuotationMark,
	Radical,
	#[parse_name("Regional_Indicator")]
	RegionalIndicator,
	#[parse_name("Sentence_Terminal")]
	SentenceTerminal,
	#[parse_name("Soft_Dotted")]
	SoftDotted, 
	#[parse_name("Terminal_Punctuation")]
	TerminalPunctuation,
	#[parse_name("Unified_Ideograph")]
	UnifiedIdeograph,
	#[parse_name("Variation_Selector")]
	VariationSelector,
	#[parse_name("White_Space")]
	WhiteSpace,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum UnicodeAge {
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

#[derive(Clone, Copy)]
pub struct BidiBracket {
	pub orientation: BidiBracketOrientation,
	pub opposite:    char,
}

/// Unicode block
#[derive(Clone, Copy, Debug)]
pub struct UnicodeBlock {
	/// Start of unicode block.
	pub start: u32,
	/// End of unicode block.
	pub end:   u32,
	/// Unicode block name.
	pub name:  &'static str,
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
	fn from_short_name(s: &str) -> Self {
		match s {
			"Zinh" => Self::Inherited,
			"Latn" => Self::Latin,
			"Grek" => Self::Greek,
			"Cyrl" => Self::Cyrillic,
			"Armn" => Self::Armenian,
			"Hebr" => Self::Hebrew,
			"Arab" => Self::Arabic,
			"Syrc" => Self::Syriac,
			"Thaa" => Self::Thaana,
			"Deva" => Self::Devanagari,
			"Beng" => Self::Bengali,
			"Guru" => Self::Gurmukhi,
			"Gujr" => Self::Gujarati,
			"Orya" => Self::Oriya,
			"Taml" => Self::Tamil,
			"Telu" => Self::Telugu,
			"Knda" => Self::Kannada,
			"Mlym" => Self::Malayalam,
			"Sinh" => Self::Sinhala,
			"Thai" => Self::Thai,
			"Laoo" => Self::Lao,
			"Tibt" => Self::Tibetan,
			"Mymr" => Self::Myanmar,
			"Geor" => Self::Georgian,
			"Hang" => Self::Hangul,
			"Ethi" => Self::Ethiopic,
			"Cher" => Self::Cherokee,
			"Cans" => Self::CanadianAboriginal,
			"Ogam" => Self::Ogham,
			"Runnr" => Self::Runic,
			"Khmr" => Self::Khmer,
			"Mong" => Self::Mongolian,
			"Hira" => Self::Hiragana,
			"Kana" => Self::Katakana,
			"Bopo" => Self::Bopomofo,
			"Hani" => Self::Han,
			"Yiii" => Self::Yi,
			"Ital" => Self::OldItalic,
			"Goth" => Self::Gothic,
			"Dsrt" => Self::Deseret,
			"Tglg" => Self::Tagalog,
			"Hano" => Self::Hanunoo,
			"Buhd" => Self::Buhid,
			"Tagb" => Self::Tagbanwa,
			"Limb" => Self::Limbu,
			"Tale" => Self::TaiLe,
			"Linb" => Self::LinearB,
			"Ugar" => Self::Ugaritic,
			"Shaw" => Self::Shavian,
			"Osma" => Self::Osmanya,
			"Cprt" => Self::Cypriot,
			"Brai" => Self::Braille,
			"Budi" => Self::Buginese,
			"Copt" => Self::Coptic,
			"Talu" => Self::NewTaiLue,
			"Glag" => Self::Glagolitic,
			"Tgnh" => Self::Tifinagh,
			"Sylo" => Self::SylotiNagri,
			"Xpeo" => Self::OldPersian,
			"Khar" => Self::Kharoshthi,
			"Bali" => Self::Balinese,
			"Xsux" => Self::Cuneiform,
			"Phnx" => Self::Phoenician,
			"Phag" => Self::PhagsPa,
			"Nkoo" => Self::Nko,
			"Sunb" => Self::Sundanese,
			"Lepc" => Self::Lepcha,
			"Olch" => Self::OlChiki,
			"Vaii" => Self::Vai,
			"Saur" => Self::Saurashtra,
			"Kali" => Self::KayahLi,
			"Rjng" => Self::Rejang,
			"Lyci" => Self::Lycian,
			"Cari" => Self::Carian,
			"Lydi" => Self::Lydian,
			"Cham" => Self::Cham,
			"Lana" => Self::TaiTham,
			"Tavt" => Self::TaiViet,
			"Avst" => Self::Avestan,
			"Egyp" => Self::EgyptianHieroglyphs,
			"Samr" => Self::Samaritan,
			"Lisu" => Self::Lisu,
			"Bamu" => Self::Bamum,
			"Java" => Self::Javanese,
			"Mtei" => Self::MeeteiMayek,
			"Armi" => Self::ImperialAramaic,
			"Sarb" => Self::OldSouthArabian,
			"Prti" => Self::InscriptionalParthian,
			"Phli" => Self::InscriptionalPahlavi,
			"Orkh" => Self::OldTurkic,
			"Kthi" => Self::Kaithi,
			"Batk" => Self::Batak,
			"Brah" => Self::Brahmi,
			"Mand" => Self::Mandaic,
			"Cakm" => Self::Chakma,
			"Merc" => Self::MeroiticCursive,
			"Mero" => Self::MeroiticHieroglyphs,
			"Plrd" => Self::Miao,
			"Shrd" => Self::Sharada,
			"Sora" => Self::SoraSompeng,
			"Takr" => Self::Takri,
			"Aghb" => Self::CaucasianAlbanian,
			"Bass" => Self::BassaVah,
			"Dupl" => Self::Duployan,
			"Elba" => Self::Elbasan,
			"Gran" => Self::Grantha,
			"Hmng" => Self::PahawhHmong,
			"Khoj" => Self::Khojki,
			"Lina" => Self::LinearA,
			"Mahj" => Self::Mahajani,
			"Mani" => Self::Manichaean,
			"Mend" => Self::MendeKikakui,
			"Modi" => Self::Modi,
			"Mroo" => Self::Mro,
			"Narb" => Self::OldNorthArabian,
			"Nbat" => Self::Nabataean,
			"Palm" => Self::Palmyrene,
			"Pauc" => Self::PauCinHau,
			"Perm" => Self::OldPermic,
			"Phlp" => Self::PsalterPahlavi,
			"Sidd" => Self::Siddham,
			"Sind" => Self::Khudawadi,
			"Tirh" => Self::Tirhuta,
			"Wara" => Self::WarangCiti,
			"Ahom" => Self::Ahom,
			"Hluw" => Self::AnatolianHieroglyphs,
			"Hatr" => Self::Hatran,
			"Mult" => Self::Multani,
			"Hung" => Self::OldHungarian,
			"Sgnw" => Self::SignWriting,
			"Adlm" => Self::Adlam,
			"Bhks" => Self::Bhaiksuki,
			"Marc" => Self::Marchen,
			"Newa" => Self::Newa,
			"Osge" => Self::Osage,
			"Tang" => Self::Tangut,
			"Gonm" => Self::MasaramGondi,
			"Nshu" => Self::Nushu,
			"Soyo" => Self::Soyombo,
			"Zanb" => Self::ZanabazarSquare,
			"Dogr" => Self::Dogra,
			"Gong" => Self::GunjalaGondi,
			"Maka" => Self::Makasar,
			"Medf" => Self::Medefaidrin,
			"Hmnp" => Self::NyiakengPuachueHmong,
			"Wcho" => Self::Wancho,
			"Kits" => Self::KhitanSmallScript,
			"Cpmn" => Self::CyproMinoan,
			_ => Self::Unknown
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
	/// For programmatic determination of default ifnorable code points.
	/// New characters taht should be ifnored in redering (unless explicitly supported) will be assigned in these ranges,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumFromName)]
pub enum IndicConjunctBreak {
	Linker,
	Consonant,
	Extend,
	None
}

//==============================================================================================================================
// Unicode parser build code
//==============================================================================================================================

//==============================================================
// Parser specific implementation
//==============================================================

impl Rational {
	pub fn from_str(s: &str) -> Option<Self> {
		if s.is_empty() {
			return None;
		}

		if s.contains('/')  {	
			let mut elems = s.split('/');
			Some(Self {
				numerator: i64::from_str_radix(elems.next().unwrap(), 10).unwrap(),
				denominator: u64::from_str_radix(elems.next().unwrap(), 10).unwrap(),
			})
		} else {
			Some(Self {
				numerator: i64::from_str_radix(s, 10).unwrap(),
				denominator: 1,
			})
		}
	}
}

//==============================================================
// Parser specific structs/enums
//==============================================================

/// Character decomposition mapping
#[derive(Clone, PartialEq, Eq)]
pub enum BuildCharacterDecomposition {
	None,
	Normal(Vec<u32>),
	Font(u32),
	NoBreak(u32),
	Initial(Vec<u32>),
	Medial(Vec<u32>),
	Final(Vec<u32>),
	Isolated(Vec<u32>),
	Circle(Vec<u32>),
	Super(u32),
	Sub(u32),
	Vertical(Vec<u32>),
	Wide(u32),
	Narrow(u32),
	Small(u32),
	Square(Vec<u32>),
	Fraction(Vec<u32>),
	Compat(Vec<u32>),
}

impl BuildCharacterDecomposition {
	pub fn from_str(s: &str) -> Self {
		if s.is_empty() {
			return BuildCharacterDecomposition::None;
		}

		let mut elems = s.split(' ');
		let name = elems.next().unwrap();
		let mut vals = Vec::new();
		if !name.starts_with('<') {
			vals.push(u32::from_str_radix(name, 16).unwrap());
		}

		for s_val in elems {
			vals.push(u32::from_str_radix(s_val, 16).unwrap());
		}

		match name {
			"<font>"     => BuildCharacterDecomposition::Font(vals[0]),
			"<noBreak>"  => BuildCharacterDecomposition::NoBreak(vals[0]),
			"<initial>"  => BuildCharacterDecomposition::Initial(vals),
			"<medial>"   => BuildCharacterDecomposition::Medial(vals),
			"<final>"    => BuildCharacterDecomposition::Medial(vals),
			"<isolated>" => BuildCharacterDecomposition::Medial(vals),
			"<circle>"   => BuildCharacterDecomposition::Circle(vals),
			"<super>"    => BuildCharacterDecomposition::Super(vals[0]),
			"<sub>"      => BuildCharacterDecomposition::Sub(vals[0]),
			"<vertical>" => BuildCharacterDecomposition::Vertical(vals),
			"<wide>"     => BuildCharacterDecomposition::Wide(vals[0]),
			"<narrow>"   => BuildCharacterDecomposition::Narrow(vals[0]),
			"<small>"    => BuildCharacterDecomposition::Small(vals[0]),
			"<square>"   => BuildCharacterDecomposition::Square(vals),
			"<fraction>" => BuildCharacterDecomposition::Fraction(vals),
			"<compat>"   => BuildCharacterDecomposition::Compat(vals),
			_            => BuildCharacterDecomposition::Normal(vals)
		}
	}
}

impl fmt::Display for BuildCharacterDecomposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildCharacterDecomposition::None           => write!(f, "CharacterDecomposition::None"),
            BuildCharacterDecomposition::Normal(vals)   => write!(f, "CharacterDecomposition::Normal(&{vals:?})"),
            BuildCharacterDecomposition::Font(val)      => write!(f, "CharacterDecomposition::Font({val})"),
            BuildCharacterDecomposition::NoBreak(val)   => write!(f, "CharacterDecomposition::NoBreak({val})"),
            BuildCharacterDecomposition::Initial(vals)  => write!(f, "CharacterDecomposition::Initial(&{vals:?})"),
            BuildCharacterDecomposition::Medial(vals)   => write!(f, "CharacterDecomposition::Medial(&{vals:?})"),
            BuildCharacterDecomposition::Final(vals)    => write!(f, "CharacterDecomposition::Final(&{vals:?})"),
            BuildCharacterDecomposition::Isolated(vals) => write!(f, "CharacterDecomposition::Isolated(&{vals:?})"),
            BuildCharacterDecomposition::Circle(vals)   => write!(f, "CharacterDecomposition::Circle(&{vals:?})"),
            BuildCharacterDecomposition::Super(val)     => write!(f, "CharacterDecomposition::Super({val})"),
            BuildCharacterDecomposition::Sub(val)       => write!(f, "CharacterDecomposition::Sub({val})"),
            BuildCharacterDecomposition::Vertical(vals) => write!(f, "CharacterDecomposition::Vertical(&{vals:?})"),
            BuildCharacterDecomposition::Wide(val)      => write!(f, "CharacterDecomposition::Wide({val})"),
            BuildCharacterDecomposition::Narrow(val)    => write!(f, "CharacterDecomposition::Narrow({val})"),
            BuildCharacterDecomposition::Small(val)     => write!(f, "CharacterDecomposition::Small({val})"),
            BuildCharacterDecomposition::Square(vals)   => write!(f, "CharacterDecomposition::Square(&{vals:?})"),
            BuildCharacterDecomposition::Fraction(vals) => write!(f, "CharacterDecomposition::Fraction(&{vals:?})"),
            BuildCharacterDecomposition::Compat(vals)   => write!(f, "CharacterDecomposition::Compat(&{vals:?})"),
        }
    }
}

pub struct BuildJoiningInfo {
	name:      String,
	join_type: JoiningType,
}

impl core::fmt::Debug for BuildJoiningInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JoiningInfo {{ name: \"{}\", join_type: JoiningType::", &self.name)?;
		match self.join_type { 
    	    JoiningType::RightJoining(group) => write!(f, "RightJoining(RightJoiningGroup::{group:?})")?,
    	    JoiningType::LeftJoining(group)  => write!(f, "LeftJoining(LeftJoiningGroup::{group:?})")?,
    	    JoiningType::DualJoining(group)  => write!(f, "DualJoining(DualJoiningGroup::{group:?})")?,
    	    JoiningType::JoinCausing         => write!(f, "JoinCausing")?,
    	    JoiningType::NonJoining(group)   => write!(f, "NonJoining(NonJoiningGroup::{group:?})")?,
    	    JoiningType::Transparent         => write!(f, "Transparent")?,
    	}
		write!(f, " }}")
    }
}


#[derive(Clone)]
pub enum BuildCasing {
	Simple(char),
	Complex(Vec<char>),
	Conditional(Vec<(String, Vec<char>)>),
}

impl core::fmt::Debug for BuildCasing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildCasing::Simple(ch) => write!(f, "Casing::Simple('{ch}')"),
            BuildCasing::Complex(chs) => write!(f, "Casing::Complex(&{chs:?})"),
            BuildCasing::Conditional(conditions) => {
				write!(f, "Casing::Conditional(&[")?;
				for (idx, (condition, chs)) in conditions.iter().enumerate() {
					if idx != 0 {
						write!(f, ", ")?;
					}
					write!(f, "(\"{condition}\", &{chs:?})")?;
				}
				write!(f, "])")
			},
        }
    }
}

impl core::fmt::Debug for BidiBracket {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "BidiBracket {{ orientation: BidiBracketOrientation::{:?}, opposite: '{}' }}", self.orientation, self.opposite)
	}
}

// TODO: remove 'None' equivalent values
pub fn generate_unicode_data() {
	let mut names = Vec::new();
	let mut flags = Vec::new();
	let mut categories = Vec::new();
	let mut canon_combine_classes = Vec::new();
	let mut bidi_classes = Vec::new();
	let mut decomps = Vec::new();
	let mut to_lower = Vec::new();
	let mut to_upper = Vec::new();
	let mut to_title = Vec::new();
	let mut numeric_values = Vec::new();
	let mut digit_values = Vec::new();
	let mut rational_values = Vec::new();

	parse_file("UnicodeData.txt", |line| {
		let elems = line.trim_end().split(';').collect::<Vec<_>>();
		assert!(elems.len() == 15, "Malformed UnicodeData.txt line");

		let id = u32::from_str_radix(elems[0], 16).unwrap();

		// Skip end of CJK and hangul values
		if id == 0x9FFF ||id == 0xD7A3 {
			return;
		}
		
		let category = UnicodeCategory::parse(elems[2]).unwrap();
		let canon_combine = usize::from_str_radix(elems[3], 10).unwrap();
		let canon_combine = CanonicalCombiningClass::from_idx(canon_combine).unwrap();
		let bidi = BidirectionalClass::parse(elems[4]).unwrap();
		let decomp = BuildCharacterDecomposition::from_str(elems[5]);
		let bidi_mirrored = elems[9] == "Y";
		let flag = if bidi_mirrored { UnicodeFlags::BidiMirrored } else { UnicodeFlags::None };

		// Special case to add all CJK Unified Ideograph
		if id == 0x4E00 {
			for id in 0x4E00..=0x9FFF {
				// We also base the data on the struct on some know info for Unicode 15.1
				names.push((id, format!("CJK Unified Ideograph-{id:04X}")));
				let id = UnicodeIndex::Single(id);
				flags.push((id, flag));
				categories.push((id, category));
				canon_combine_classes.push((id, canon_combine));
				bidi_classes.push((id, bidi));
				decomps.push((id, decomp.clone()));
			}
		} else if id == 0xAC00 { // Special case for Hangul
			for id in 0xAC00..=0xD7A3 {
				names.push((id, format!("Hangul Syllable {id:04X}")));
				let id = UnicodeIndex::Single(id);
				categories.push((id, category));
				canon_combine_classes.push((id, canon_combine));
				bidi_classes.push((id, bidi));
				decomps.push((id, decomp.clone()));
				flags.push((id, flag))
			}
		} else {
			if let Some(ch) = char::from_u32(id) {
				if let Ok(val) = u32::from_str_radix(elems[12], 16) {
					let to = char::from_u32(val).unwrap();
					to_upper.push((ch, BuildCasing::Simple(to)));
				} 
				if let Ok(val) = u32::from_str_radix(elems[13], 16) {
					let to = char::from_u32(val).unwrap();
					to_lower.push((ch, BuildCasing::Simple(to)));
				} 
				if let Ok(val) = u32::from_str_radix(elems[14], 16) {
					let to = char::from_u32(val).unwrap();
					to_title.push((ch, BuildCasing::Simple(to)));
				} 
			}

			let numeric_value = u8::from_str_radix(elems[6], 10).map_or(None, |val| Some(val));
			let digit_value = u8::from_str_radix(elems[7], 10).map_or(None, |val| Some(val));
			let rational_value = Rational::from_str(elems[8]);

			names.push((id, elems[1].to_string()));
			let id = UnicodeIndex::Single(id);
			categories.push((id, category));
			canon_combine_classes.push((id, canon_combine));
			bidi_classes.push((id, bidi));
			decomps.push((id, decomp));

			if let Some(numeric_value) = numeric_value {
				numeric_values.push((id, numeric_value));
			}
			if let Some(digit_value) = digit_value {
				digit_values.push((id, digit_value));
			}
			if let Some(rational_value) = rational_value {
				rational_values.push((id, rational_value));
			}
			flags.push((id, flag))
		}
	});
	sort_and_compact(&mut categories, None);
	sort_and_compact(&mut canon_combine_classes, Some(CanonicalCombiningClass::NotReordered));
	sort_and_compact(&mut bidi_classes, Some(BidirectionalClass::LeftToRight));
	sort_and_compact(&mut decomps, Some(BuildCharacterDecomposition::None));
	sort_and_compact(&mut numeric_values, None);
	sort_and_compact(&mut digit_values, None);
	sort_and_compact(&mut rational_values, None);

	parse_file("emoji/emoji-data.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|val| val.trim());
		let code_s = elems.next().unwrap();
		let property = elems.next().unwrap();

		let mut set_emoji_data = |code: u32| {
			let idx = match flags.binary_search_by_key(&UnicodeIndex::Single(code), |elem| elem.0) {
				Ok(idx) => idx,
				Err(_) => return,
			};
			flags[idx].1 |= UnicodeFlags::parse(property).unwrap();
		};

		if code_s.contains("..") {
			let mut elems = code_s.split("..");
			let start = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			let end = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();

			for code in start..=end {
				set_emoji_data(code);
			}
		} else {
			let code = u32::from_str_radix(code_s, 16).unwrap();
			set_emoji_data(code);
		}
	});


	parse_file("CompositionExclusions.txt", |line| {
		let mut elems = line.split('#').map(|s| s.trim());
		let code = u32::from_str_radix(elems.next().unwrap().trim(), 16).unwrap();
		let idx = flags.binary_search_by_key(&UnicodeIndex::Single(code), |val| val.0).unwrap();
		flags[idx].1 |= UnicodeFlags::CompositionExclusion;
	});

	let mut uni_age = Vec::new();
	parse_file("DerivedAge.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let age = UnicodeAge::parse(elems.next().unwrap()).unwrap_or(UnicodeAge::Unknown);
		uni_age.push((index, age));
	});
	sort_and_compact(&mut uni_age, None);

	let mut east_asian_widths = Vec::new();
	parse_file("EastAsianWidth.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let width = EastAsianWidth::parse(elems.next().unwrap()).unwrap();
		east_asian_widths.push((index, width));
	});
	sort_and_compact(&mut east_asian_widths, None);

	let mut joining_info = Vec::new();
	parse_file("ArabicShaping.txt", |line| {
		let mut elems = line.split(';').map(|val| val.trim());

		let ch = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
		let name = elems.next().unwrap().to_string();
		let join_type = JoiningType::parse(elems.next().unwrap(), elems.next().unwrap());

		joining_info.push((UnicodeIndex::Single(ch), BuildJoiningInfo {
		    name,
		    join_type,
		}));
	});

	let mut bidi_paired_brackets = Vec::new();
	parse_file("BidiBrackets.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());

		let ch = char::from_u32(u32::from_str_radix(elems.next().unwrap(), 16).unwrap()).unwrap();
		let opposite = char::from_u32(u32::from_str_radix(elems.next().unwrap(), 16).unwrap()).unwrap();
		let orientation = match elems.next().unwrap().trim() {
			"o" => BidiBracketOrientation::Open,
			"c" => BidiBracketOrientation::Close,
			_   => unreachable!(),
		};

		bidi_paired_brackets.push((ch, BidiBracket {
			opposite,
			orientation
		}));
	});
	bidi_paired_brackets.sort_unstable_by_key(|val| val.0);

	let mut mirroring_glyphs = Vec::new();
	parse_file("BidiMirroring.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());

		let code = char::from_u32(u32::from_str_radix(elems.next().unwrap(), 16).unwrap()).unwrap();
		let mirror = char::from_u32(u32::from_str_radix(elems.next().unwrap(), 16).unwrap()).unwrap();
		mirroring_glyphs.push((code, mirror));
	});
	mirroring_glyphs.sort_unstable_by_key(|val| val.0);

	let mut uni_blocks = Vec::new();
	parse_file("Blocks.txt", |line| {
		let mut elems = line.split(';').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		uni_blocks.push((index, elems.next().unwrap().to_string()));
	});

	let mut emoji_sources = Vec::new();
	parse_file("EmojiSources.txt", |line| {
		let mut elems = line.split(';');

		let elem0 = elems.next().unwrap();
		let code = if elem0.contains(' ') {
			let mut codes = elem0.split(' ');
			(
				u32::from_str_radix(codes.next().unwrap(), 16).unwrap(),
				u32::from_str_radix(codes.next().unwrap(), 16).unwrap(),
			)
		} else {
			(
				u32::from_str_radix(elem0, 16).unwrap(),
				0
			)
		};

		emoji_sources.push(EmojiSource {
			code,
			docomo: u32::from_str_radix(elems.next().unwrap(), 16).map_or(None, |val| Some(val)),
			kddi: u32::from_str_radix(elems.next().unwrap(), 16).map_or(None, |val| Some(val)),
			softbank: u32::from_str_radix(elems.next().unwrap(), 16).map_or(None, |val| Some(val)),
		});
	});
	emoji_sources.sort_unstable_by_key(|val| val.code.1);
	emoji_sources.sort_by_key(|val| val.code.0);

	let mut equivalent_unified_ideographs = Vec::new();
	parse_file("EquivalentUnifiedIdeograph.txt", |line| {
		let elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim()).collect::<Vec<_>>();
		let unified = char::from_u32(u32::from_str_radix(elems[1], 16).unwrap()).unwrap();

		if elems[0].contains("..") {
			let mut elems = elems[0].split("..");
			let start = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			let end = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();

			for code in start..=end {
				let ch = char::from_u32(code).unwrap();
				equivalent_unified_ideographs.push((ch, unified));
			}
		} else {
			let code = u32::from_str_radix(elems[0], 16).unwrap();
			let ch = char::from_u32(code).unwrap();
			equivalent_unified_ideographs.push((ch, unified));
		}
	});
	equivalent_unified_ideographs.sort_unstable_by_key(|val| val.0);

	let mut hangul_syllable_types = Vec::new();
	parse_file("HangulSyllableType.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let syllable_type = HangulSyllableType::parse(elems.next().unwrap()).unwrap();
		hangul_syllable_types.push((index, syllable_type));
	});
	sort_and_compact(&mut hangul_syllable_types, None);

	let mut indic_positional_categories = Vec::new();
	parse_file("IndicPositionalCategory.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let indic_positional = IndicPositionalCategory::parse(elems.next().unwrap()).unwrap();
		indic_positional_categories.push((index, indic_positional));
	});
	sort_and_compact(&mut indic_positional_categories, None);

	let mut indic_syllabic_categories = Vec::new();
	parse_file("IndicSyllabicCategory.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let indic_syllabic = IndicSyllabicCategory::parse(elems.next().unwrap()).unwrap();
		indic_syllabic_categories.push((index, indic_syllabic));
	});
	sort_and_compact(&mut indic_syllabic_categories, None);

	let mut jamo_short_names = Vec::new();
	parse_file("Jamo.txt", |line| {
		let mut elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim());
		let code = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
		let ch = char::from_u32(code).unwrap();
		let jamo = JamoShortName::parse(elems.next().unwrap()).unwrap();
		jamo_short_names.push((ch, jamo));
	});
	jamo_short_names.sort_unstable_by_key(|val| val.0);

	let mut line_breaks = Vec::new();
	parse_file("LineBreak.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let line_break = LineBreak::parse(elems.next().unwrap()).unwrap();
		line_breaks.push((index, line_break));
	});
	sort_and_compact(&mut line_breaks, None);
	
	let mut grapheme_breaks = Vec::new();
	parse_file("GraphemeBreakProperty.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let grapheme_break = GraphemeClusterBreak::parse(elems.next().unwrap()).unwrap();
		grapheme_breaks.push((index, grapheme_break));
	});
	sort_and_compact(&mut grapheme_breaks, None);
	
	let mut sentence_breaks = Vec::new();
	parse_file("SentenceBreakProperty.txt", |line| {
		let   mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let sentence_break = SentenceBreak::parse(elems.next().unwrap()).unwrap();
		sentence_breaks.push((index, sentence_break));
	});
	sort_and_compact(&mut sentence_breaks, None);
	
	let mut word_breaks = Vec::new();
	parse_file("WordBreakProperty.txt", |line| {
		let elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim()).collect::<Vec<_>>();
		let word_break = WordBreak::parse(elems[1]).unwrap();

		if elems[0].contains("..") {
			let mut elems = elems[0].split("..");
			let begin = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			let end = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();

			word_breaks.push((UnicodeIndex::Range(begin, end), word_break));
		} else {
			let code = u32::from_str_radix(elems[0], 16).unwrap();
			word_breaks.push((UnicodeIndex::Single(code), word_break));
		}
	});
	sort_and_compact(&mut word_breaks, None);

	let mut scripts = Vec::new();
	parse_file("Scripts.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());
		let index = UnicodeIndex::parse(elems.next().unwrap());
		let script = Script::parse(elems.next().unwrap()).unwrap();
		scripts.push((index, script));
	});
	sort_and_compact(&mut scripts, None);

	let mut script_extensions = Vec::new();
	parse_file("ScriptExtensions.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());

		let index = UnicodeIndex::parse(elems.next().unwrap());

		let mut sub_scripts = Vec::new();
		for name in elems.next().unwrap().split(' ') {
			sub_scripts.push(Script::from_short_name(name));
		}

		script_extensions.push((index, sub_scripts));
	});
	sort_and_compact(&mut script_extensions, None);

	parse_file("SpecialCasing.txt", |line| {
		let elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim()).collect::<Vec<_>>();

		let id = u32::from_str_radix(elems[0], 16).unwrap();
		let id = char::from_u32(id).unwrap();
		
		let mut lower = Vec::new();
		if !elems[1].is_empty() {
			for val in elems[1].split(' ') {
				lower.push(char::from_u32(u32::from_str_radix(val, 16).unwrap()).unwrap());
			}
		}
		
		let mut upper = Vec::new();
		if !elems[2].is_empty() {
			for val in elems[2].split(' ') {
				upper.push(char::from_u32(u32::from_str_radix(val, 16).unwrap()).unwrap());
			}
		}
		
		let mut title = Vec::new();
		if !elems[3].is_empty() {
			for val in elems[3].split(' ') {
				title.push(char::from_u32(u32::from_str_radix(val, 16).unwrap()).unwrap());
			}
		}

		let condition = elems[4];

		let add_cond = |old, cond, chs| -> BuildCasing {
			match old {
				BuildCasing::Simple(ch) => BuildCasing::Conditional(vec![(String::new(), vec![ch]), (cond, chs)]),
				BuildCasing::Complex(old_chs) => BuildCasing::Conditional(vec![(String::new(), old_chs), (cond, chs)]),
				BuildCasing::Conditional(mut old_conditions) => {
					old_conditions.push((cond, chs));
					BuildCasing::Conditional(old_conditions)
				},
			}
		};

		if !lower.is_empty() && (lower.len() != 1 || lower[0] != id) {
			if !condition.is_empty() {
				let condition = condition.to_string();
				if let Ok(idx) = to_lower.binary_search_by_key(&id, |val| val.0) {
					to_lower[idx].1 = add_cond(to_lower[idx].1.clone(), condition, lower);
				} else {
					to_lower.push((id, BuildCasing::Conditional(vec![(condition, lower)])));
				}
			} else if lower.len() > 0 {
				to_lower.push((id, BuildCasing::Complex(lower)))
			} else {
				to_lower.push((id, BuildCasing::Simple(lower[0])))				
			}
		}

		if !upper.is_empty() && (upper.len() != 1 || upper[0] != id) {
			if !condition.is_empty() {
				let condition = condition.to_string();
				if let Ok(idx) = to_upper.binary_search_by_key(&id, |val| val.0) {
					to_lower[idx].1 = add_cond(to_upper[idx].1.clone(), condition, upper);
				} else {
					to_upper.push((id, BuildCasing::Conditional(vec![(condition, upper)])));
				}
			} else if upper.len() > 0 {
				to_upper.push((id, BuildCasing::Complex(upper)))
			} else {
				to_upper.push((id, BuildCasing::Simple(upper[0])))				
			}
			
		}

		if !title.is_empty() && (title.len() != 1 || title[0] != id) {
			if !condition.is_empty() {
				let condition = condition.to_string();
				if let Ok(idx) = to_title.binary_search_by_key(&id, |val| val.0) {
					to_title[idx].1 = add_cond(to_title[idx].1.clone(), condition, title);
				} else {
					to_title.push((id, BuildCasing::Conditional(vec![(condition, title)])));
				}
			} else if title.len() > 0 {
				to_title.push((id, BuildCasing::Complex(title)))
			} else {
				to_title.push((id, BuildCasing::Simple(title[0])))				
			}
			
		}
	});
	to_lower.sort_unstable_by_key(|val| val.0);
	to_upper.sort_unstable_by_key(|val| val.0);
	to_title.sort_unstable_by_key(|val| val.0);

	parse_file("NameAliases.txt", |line| {
		let elems = line.split(';').map(|s| s.trim()).collect::<Vec<_>>();
		if elems[2] == "correction" || elems[2] == "control" {
			let code = u32::from_str_radix(elems[0].trim(), 16).unwrap();
			let idx = names.binary_search_by_key(&code, |val| val.0).unwrap();
			names[idx].1 = elems[1].to_string();
		}
	});

	let mut vertical_orientations = Vec::new();
	parse_file("VerticalOrientation.txt", |line| {
		let mut elems = line.split(|ch| ch ==';' || ch == '#').map(|s| s.trim());

		let index = UnicodeIndex::parse(elems.next().unwrap());
		let vert_orient = VerticalOrientation::parse(elems.next().unwrap()).unwrap();
		vertical_orientations.push((index, vert_orient));
	});
	sort_and_compact(&mut vertical_orientations, None);
	// sort_and_compact(&mut vertical_orientations, Some(VerticalOrientation::Upright));

	parse_file("PropList.txt", |line| {
		let elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim()).collect::<Vec<_>>();

		// `Hyphen` is deprecated since 6.0.0, so ignore it
		if elems[1] == "Hyphen" {
			return;
		}

		let prop = UnicodeFlags::parse(elems[1]).unwrap();

		if elems[0].contains("..") {
			let mut elems = elems[0].split("..");
			let begin = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();
			let end = u32::from_str_radix(elems.next().unwrap(), 16).unwrap();

			for id in begin..=end {
				if let Ok(idx) = flags.binary_search_by_key(&UnicodeIndex::Single(id), |val| val.0) {
					flags[idx].1 |= prop;
				}
			}
		} else {
			let code = u32::from_str_radix(elems[0], 16).unwrap();
			if let Ok(idx) = flags.binary_search_by_key(&UnicodeIndex::Single(code), |val| val.0) {
				flags[idx].1 |= prop;
			} else {
				assert!(line.contains("reserved"));
			}
		}
	});
	sort_and_compact(&mut flags, Some(UnicodeFlags::None));

	let mut derived_props = Vec::<(UnicodeIndex, DerivedCoreProperty)>::new();
	let mut indic_conjunction_breaks = Vec::new();
	parse_file("DerivedCoreProperties.txt", |line| {
		let elems = line.split(|ch| ch == ';' || ch == '#').map(|s| s.trim()).collect::<Vec<_>>();

		// Deprecated property since Unicode 5.0.0, so ignore it
		if elems[1] == "Grapheme_Link" {
			return;
		}

		let index = UnicodeIndex::parse(elems[0]);
		if elems[1] == "InCB" {
			let indic_conjunct_break = IndicConjunctBreak::parse(elems[2]).unwrap();
			indic_conjunction_breaks.push((index, indic_conjunct_break));
		} else {
			let prop = DerivedCoreProperty::parse(elems[1]).expect(&elems[1]);
			if let Some(cur_prop) = derived_props.iter_mut().find(|val| index >= val.0 && index <= val.0) {
				cur_prop.1 |= prop;
			} else {
				derived_props.push((index, prop));
			}
		}
	});
	sort_and_compact(&mut derived_props, None);
	sort_and_compact(&mut indic_conjunction_breaks, None);

	// Generate source file
	let out_code = File::create("src/unicode.rs").unwrap();

	let mut writer = LineWriter::new(out_code);
	write!(writer, r"// This file is generated by the build script, and should not be edited manually.

use crate::*;

").unwrap();
	
	write!(writer, "pub(crate) const NAMES: [(u32, &'static str); {}] = [\n", names.len()).unwrap();
	for (id, name) in names {
		write!(writer, "\t({id:#08X}, \"{name}\"),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();

	write!(writer, "pub(crate) const FLAGS: [(UnicodeIndex, UnicodeFlags); {}] = [\n", flags.len()).unwrap();
	for (idx, val) in flags {
		let mut formatted_flags = format!("{val:#}");
		if formatted_flags.contains('|') {
			formatted_flags = formatted_flags.replace("|", ").bitor(");
			formatted_flags.insert(0, '(');
			formatted_flags.push(')');
		}

		write!(writer, "\t({idx:?}, {formatted_flags}),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();

	write_arr_index(&mut writer, "CATEGORIES", "Category", categories, true);
	write_arr_index(&mut writer, "CANONICAL_COMBINE_CLASSES", "CanonicalCombiningClass", canon_combine_classes, true);
	write_arr_index(&mut writer, "BIDIRECTIONAL_CLASSES", "BidirectionalClass", bidi_classes, true);
	
	write!(writer, "pub(crate) const DECOMPOSITIONS: [(UnicodeIndex, CharacterDecomposition); {}] = [\n", decomps.len()).unwrap();
	for (id, decomp) in decomps {
		write!(writer, "\t({id:?}, {decomp}),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();
	
	write_arr_char(&mut writer, "TO_LOWER", "Casing", to_lower, false);
	write_arr_char(&mut writer, "TO_UPPER", "Casing", to_upper, false);
	write_arr_char(&mut writer, "TO_TITLE", "Casing", to_title, false);
	write_arr_index(&mut writer, "DERIVED_AGE", "Age", uni_age, true);
	write_arr_index(&mut writer, "NUMERIC_VALUES", "u8", numeric_values, false);
	write_arr_index(&mut writer, "DIGIT_VALUES", "u8", digit_values, false);
	write_arr_index(&mut writer, "RATIONAL_VALUES", "Rational", rational_values, false);
	write_arr_index(&mut writer, "EAST_ASIAN_WIDTHS", "EastAsianWidth", east_asian_widths, true);
	write_arr_index(&mut writer, "JOINING_INFO", "JoiningInfo", joining_info, false);
	write_arr_char(&mut writer, "BIDI_PAIRED_BRACKETS", "BidiBracket", bidi_paired_brackets, false);

	write_arr_char(&mut writer, "BIDI_MIRRORING_GLYPH", "char", mirroring_glyphs, false);

	write!(writer, "pub(crate) const UNI_BLOCKS: [(UnicodeIndex, &'static str); {}] = [\n", uni_blocks.len()).unwrap();
	for (idx, block) in uni_blocks {
		write!(writer, "\t({idx:?}, \"{block}\"),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();

	write!(writer, "pub(crate) const EMOJI_SOURCES: [EmojiSource; {}] = [\n", emoji_sources.len()).unwrap();
	for source in emoji_sources {
		writer.write_fmt(format_args!("\tEmojiSource {{ code: ({:#06X}, {:#06X}), docomo: {}, kddi: {}, softbank: {} }},\n",
			source.code.0, source.code.1,
			source.docomo.map_or_else(|| "None       ".to_string(), |val| format!("Some({val})")),
			source.kddi.map_or_else(|| "None       ".to_string(), |val| format!("Some({val})")),
			source.softbank.map_or_else(|| "None       ".to_string(), |val| format!("Some({val})")),
		)).unwrap();
	}
	write!(writer, "];\n\n").unwrap();
	
	write_arr_char(&mut writer, "EQUVALENT_UNIFIED_IDEOGRAPHS", "char", equivalent_unified_ideographs, false);
	write_arr_index(&mut writer, "HANGUL_SYLLABLE_TYPE", "HangulSyllableType", hangul_syllable_types, true);
	write_arr_index(&mut writer, "INDIC_POSITIONAL_CATEGORIES", "IndicPositionalCategory", indic_positional_categories, true);
	write_arr_index(&mut writer, "INDIC_SYLLABIC_CATEGORIES", "IndicSyllabicCategory", indic_syllabic_categories, true);
	write_arr_char(&mut writer, "JAMO_SHORT_NAMES", "JamoShortName", jamo_short_names, true);
	
	write_arr_index(&mut writer, "LINE_BREAKS", "LineBreak", line_breaks, true);
	write_arr_index(&mut writer, "GRAPHEME_BREAKS", "GraphemeClusterBreak", grapheme_breaks, true);
	write_arr_index(&mut writer, "SENTENCE_BREAKS", "SentenceBreak", sentence_breaks, true);
	write_arr_index(&mut writer, "WORD_BREAKS", "WordBreak", word_breaks, true);
	write_arr_index(&mut writer, "SCRIPTS", "Script", scripts, true);
	
	write!(writer, "pub(crate) const SCRIPT_EXTENSIONS: [(UnicodeIndex, &'static [Script]); {}] = [\n", script_extensions.len()).unwrap();
	for (idx, sub_scripts) in script_extensions {
		write!(writer, "\t({idx:?}, &[").unwrap();
		for script in sub_scripts {	
			write!(writer, " Script::{script:?},").unwrap();
		}
		write!(writer, " ]),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();

	write_arr_index(&mut writer, "VERTICAL_ORIENTATIONS", "VerticalOrientation", vertical_orientations, true);

	write!(writer, "pub(crate) const DERIVED_PROPS: [(UnicodeIndex, DerivedCoreProperty); {}] = [\n", derived_props.len()).unwrap();
	for (idx, val) in derived_props {
		let mut formatted_flags = format!("{val:#}");
		if formatted_flags.contains('|') {
			formatted_flags = formatted_flags.replace("|", ").bitor(");
			formatted_flags.insert(0, '(');
			formatted_flags.push(')');
		}

		write!(writer, "\t({idx:?}, {formatted_flags}),\n").unwrap();
	}
	write!(writer, "];\n\n").unwrap();

	write_arr_index(&mut writer, "INDIC_CONJUNCT_BREAK", "IndicConjunctBreak", indic_conjunction_breaks, true);
	
}

fn write_arr_char<T: Debug>(writer: &mut dyn io::Write, name: &str, ty: &str, arr: Vec<(char, T)>, prepend_ty: bool) {
	write!(writer, "pub(crate) const {name}: [(char, {ty}); {}] = [\n", arr.len()).unwrap();
	for (idx, val) in arr {
		if prepend_ty {
			write!(writer, "\t({idx:?}, {ty}::{val:?}),\n").unwrap();
		} else {
			write!(writer, "\t({idx:?}, {val:?}),\n").unwrap();
		}
	}
	write!(writer, "];\n\n").unwrap();
}

fn write_arr_index<T: Debug>(writer: &mut dyn io::Write, name: &str, ty: &str, arr: Vec<(UnicodeIndex, T)>, prepend_ty: bool) {
	write!(writer, "pub(crate) const {name}: [(UnicodeIndex, {ty}); {}] = [\n", arr.len()).unwrap();
	for (idx, val) in arr {
		if prepend_ty {
			write!(writer, "\t({idx:?}, {ty}::{val:?}),\n").unwrap();
		} else {
			write!(writer, "\t({idx:?}, {val:?}),\n").unwrap();
		}
	}
	write!(writer, "];\n\n").unwrap();
}

fn parse_file<F: FnMut(&str)>(sub_path: &str, mut f: F) {
	let data_src_path: &Path = Path::new("unicode");

	let mut data_path = data_src_path.to_path_buf();
	data_path.push(sub_path);
	let file = File::open(data_path).unwrap();

	let mut data_reader = BufReader::new(file);
	let mut line = String::new();
	while data_reader.read_line(&mut line).unwrap() != 0 {
		let trimmed = line.trim();
		if trimmed.is_empty() || trimmed.starts_with('#') {
			line.clear();
			continue;
		}

		f(&line);
		line.clear();
	}
}

fn sort_and_compact<T: Eq + Clone>(arr: &mut Vec<(UnicodeIndex, T)>, exclude_filter: Option<T>) {
	arr.sort_unstable_by_key(|val| val.0);

	let mut compact_idx = 0;
	let mut first = true;
	for idx in 0..arr.len() {
		if let Some(exclude) = &exclude_filter {
			if arr[idx].1 == *exclude {
				continue;
			}
		}
		if first {
			arr[compact_idx] = arr[idx].clone();
			first = false;
		} else if idx != compact_idx {
			if arr[compact_idx].1 == arr[idx].1 {
				if let Some(merged) = arr[compact_idx].0.merge(arr[idx].0) {
					arr[compact_idx].0 = merged;
				} else {
					compact_idx += 1;
					arr[compact_idx] = arr[idx].clone();
				}
			} else {
				compact_idx += 1;
				arr[compact_idx] = arr[idx].clone();
			}
		}
	}

	// compact_idx should never be larger than the current size of the array
	arr.resize_with(compact_idx + 1, || unreachable!());
}

fn main() {
    generate_unicode_data();
    println!("cargo:rerun-if-changes=build.rs");
    println!("cargo:rerun-if-changes=unicode/*");
}