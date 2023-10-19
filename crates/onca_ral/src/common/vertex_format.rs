use onca_core::prelude::*;
use onca_core_macros::{EnumCount, EnumFromIndex, EnumDisplay};



/// Vertex format components
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum VertexComponents {
    X32Y32Z32W32,
    X32Y32Z32,
    X32Y32,
    X32,
    X16Y16Z16W16,
    X16Y16,
    X16,
    X8Y8Z8W8,
    X8Y8,
    X8,
    X10Y10Z10W2,
    X11Y11Z10,
}

/// Vertex format data type
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount)]
pub enum VertexDataType {
    SFloat,
    UFloat,
    SInt,
    Uint,
    SNorm,
    UNorm,
}

/// Vertex format
#[derive(Clone, Copy, PartialEq, Eq, Debug, EnumCount, EnumFromIndex, EnumDisplay)]
pub enum VertexFormat {
    X32Y32Z32W32SFloat,
    X32Y32Z32W32SInt,
    X32Y32Z32W32UInt,
    X32Y32Z32SFloat,
    X32Y32Z32SInt,
    X32Y32Z32UInt,
    X32Y32SFloat,
    X32Y32SInt,
    X32Y32UInt,
    X32SFloat,
    X32SInt,
    X32UInt,
    X16Y16Z16W16SFloat,
    X16Y16Z16W16SInt,
    X16Y16Z16W16UInt,
    X16Y16Z16W16SNorm,
    X16Y16Z16W16UNorm,
    X16Y16SFloat,
    X16Y16SInt,
    X16Y16UInt,
    X16Y16SNorm,
    X16Y16UNorm,
    X16SFloat,
    X16SInt,
    X16UInt,
    X16SNorm,
    X16UNorm,
    X8Y8Z8W8SInt,
    X8Y8Z8W8UInt,
    X8Y8Z8W8SNorm,
    X8Y8Z8W8UNorm,
    X8Y8SInt,
    X8Y8UInt,
    X8Y8SNorm,
    X8Y8UNorm,
    X8SInt,
    X8UInt,
    X8SNorm,
    X8UNorm,
    X10Y10Z10W2UInt,
    X10Y10Z10W2UNorm,
    X11Y11Z10UFloat,
    
}

impl VertexFormat {
    /// Try to get the vertex format from its components and data type
    pub fn from_components_and_data_type(component: VertexComponents, data_type: VertexDataType) -> Option<Self> {
        COMPONENTS_AND_DATA_TYPE_TO_FORMAT[component as usize][data_type as usize]
    }

    /// Get the vertex format components and data type
    pub fn to_components_and_data_type(self) -> (VertexComponents, VertexDataType) {
        let info = FORMAT_INFO[self as usize];
        (info.components, info.data_type)
    }

    /// Get the size of an element using the format
    pub fn byte_size(self) -> u8 {
        FORMAT_INFO[self as usize].byte_size
    }

    pub fn supoorts_acceleration_structure(self) -> bool {
        FORMAT_INFO[self as usize].acc_struct
    }

    /// Call a closure for each vertex format.
    pub fn for_each<F>(mut f: F)
    where
        F : FnMut(VertexFormat)
    {
        for i in 0..VertexFormat::COUNT {
            f(unsafe { Self::from_idx_unchecked(i) })
        }
    }
}

impl From<VertexFormat> for (VertexComponents, VertexDataType) {
    fn from(value: VertexFormat) -> Self {
        value.to_components_and_data_type()
    }
}

impl TryFrom<(VertexComponents, VertexDataType)> for VertexFormat {
    type Error = ();

    fn try_from(value: (VertexComponents, VertexDataType)) -> Result<Self, Self::Error> {
        Self::from_components_and_data_type(value.0, value.1).map_or(Err(()), |format| Ok(format))
    }
}

#[derive(Clone, Copy)]
struct FormatInfo {
    components: VertexComponents,
    data_type:  VertexDataType,
    byte_size:  u8,
    acc_struct: bool,
}

impl FormatInfo {
    const fn new(components: VertexComponents, data_type: VertexDataType, byte_size: u8, acc_struct: bool) -> Self {
        Self { components, data_type, byte_size, acc_struct }
    }
}

//==============================================================================================================================
// LUTS
//==============================================================================================================================

const FORMAT_INFO : [FormatInfo; VertexFormat::COUNT] = [
    /* X32Y32Z32W32SFloat  */ FormatInfo::new(VertexComponents::X32Y32Z32W32, VertexDataType::SFloat , 16, false),
    /* X32Y32Z32W32SInt    */ FormatInfo::new(VertexComponents::X32Y32Z32W32, VertexDataType::SInt   , 16, false),
    /* X32Y32Z32W32UInt    */ FormatInfo::new(VertexComponents::X32Y32Z32W32, VertexDataType::Uint   , 16, false),
    /* X32Y32Z32SFloat     */ FormatInfo::new(VertexComponents::X32Y32Z32   , VertexDataType::SFloat , 12, true ),
    /* X32Y32Z32SInt       */ FormatInfo::new(VertexComponents::X32Y32Z32   , VertexDataType::SInt   , 12, false),
    /* X32Y32Z32UInt       */ FormatInfo::new(VertexComponents::X32Y32Z32   , VertexDataType::Uint   , 12, false),
    /* X32Y32SFloat        */ FormatInfo::new(VertexComponents::X32Y32      , VertexDataType::SFloat , 8 , true ),
    /* X32Y32SInt          */ FormatInfo::new(VertexComponents::X32Y32      , VertexDataType::SInt   , 8 , false),
    /* X32Y32UInt          */ FormatInfo::new(VertexComponents::X32Y32      , VertexDataType::Uint   , 8 , false),
    /* X32SFloat           */ FormatInfo::new(VertexComponents::X32         , VertexDataType::SFloat , 4 , false),
    /* X32SInt             */ FormatInfo::new(VertexComponents::X32         , VertexDataType::SInt   , 4 , false),
    /* X32UInt             */ FormatInfo::new(VertexComponents::X32         , VertexDataType::Uint   , 4 , false),
    /* X16Y16Z16W16SFloat  */ FormatInfo::new(VertexComponents::X16Y16Z16W16, VertexDataType::SFloat , 8 , true ),
    /* X16Y16Z16W16SInt    */ FormatInfo::new(VertexComponents::X16Y16Z16W16, VertexDataType::SInt   , 8 , false),
    /* X16Y16Z16W16UInt    */ FormatInfo::new(VertexComponents::X16Y16Z16W16, VertexDataType::Uint   , 8 , false),
    /* X16Y16Z16W16SNorm   */ FormatInfo::new(VertexComponents::X16Y16Z16W16, VertexDataType::SNorm  , 8 , true ),
    /* X16Y16Z16W16UNorm   */ FormatInfo::new(VertexComponents::X16Y16Z16W16, VertexDataType::UNorm  , 8 , true ),
    /* X16Y16SFloat        */ FormatInfo::new(VertexComponents::X16Y16      , VertexDataType::SFloat , 4 , true ),
    /* X16Y16SInt          */ FormatInfo::new(VertexComponents::X16Y16      , VertexDataType::SInt   , 4 , false),
    /* X16Y16UInt          */ FormatInfo::new(VertexComponents::X16Y16      , VertexDataType::Uint   , 4 , false),
    /* X16Y16SNorm         */ FormatInfo::new(VertexComponents::X16Y16      , VertexDataType::SNorm  , 4 , true ),
    /* X16Y16UNorm         */ FormatInfo::new(VertexComponents::X16Y16      , VertexDataType::UNorm  , 4 , true ),
    /* X16SFloat           */ FormatInfo::new(VertexComponents::X16         , VertexDataType::SFloat , 2 , false),
    /* X16SInt             */ FormatInfo::new(VertexComponents::X16         , VertexDataType::SInt   , 2 , false),
    /* X16UInt             */ FormatInfo::new(VertexComponents::X16         , VertexDataType::Uint   , 2 , false),
    /* X16SNorm            */ FormatInfo::new(VertexComponents::X16         , VertexDataType::SNorm  , 2 , false),
    /* X16UNorm            */ FormatInfo::new(VertexComponents::X16         , VertexDataType::UNorm  , 2 , false),
    /* X8Y8Z8W8SInt        */ FormatInfo::new(VertexComponents::X8Y8Z8W8    , VertexDataType::SInt   , 4 , false),
    /* X8Y8Z8W8UInt        */ FormatInfo::new(VertexComponents::X8Y8Z8W8    , VertexDataType::Uint   , 4 , false),
    /* X8Y8Z8W8SNorm       */ FormatInfo::new(VertexComponents::X8Y8Z8W8    , VertexDataType::SNorm  , 4 , true ),
    /* X8Y8Z8W8UNorm       */ FormatInfo::new(VertexComponents::X8Y8Z8W8    , VertexDataType::UNorm  , 4 , true ),
    /* X8Y8SInt            */ FormatInfo::new(VertexComponents::X8Y8        , VertexDataType::SInt   , 2 , false),
    /* X8Y8UInt            */ FormatInfo::new(VertexComponents::X8Y8        , VertexDataType::Uint   , 2 , false),
    /* X8Y8SNorm           */ FormatInfo::new(VertexComponents::X8Y8        , VertexDataType::SNorm  , 2 , true ),
    /* X8Y8UNorm           */ FormatInfo::new(VertexComponents::X8Y8        , VertexDataType::UNorm  , 2 , true ),
    /* X8SInt              */ FormatInfo::new(VertexComponents::X8          , VertexDataType::SInt   , 1 , false),
    /* X8UInt              */ FormatInfo::new(VertexComponents::X8          , VertexDataType::Uint   , 1 , false),
    /* X8SNorm             */ FormatInfo::new(VertexComponents::X8          , VertexDataType::SNorm  , 1 , false),
    /* X8UNorm             */ FormatInfo::new(VertexComponents::X8          , VertexDataType::UNorm  , 1 , false),
    /* X10Y10Z10W2UInt     */ FormatInfo::new(VertexComponents::X10Y10Z10W2 , VertexDataType::Uint   , 4 , false),
    /* X10Y10Z10W2UNorm    */ FormatInfo::new(VertexComponents::X10Y10Z10W2 , VertexDataType::UNorm  , 4 , true ),
    /* X11Y11Z10UFloat,    */ FormatInfo::new(VertexComponents::X11Y11Z10   , VertexDataType::UFloat , 4 , false), 
];

const COMPONENTS_AND_DATA_TYPE_TO_FORMAT : [[Option<VertexFormat>; VertexDataType::COUNT]; VertexComponents::COUNT] = [
    //                  SFloat                                , UFloat                             , SInt                                , Uint                                , SNorm                                , UNorm
    /* X32Y32Z32W32 */ [Some(VertexFormat::X32Y32Z32W32SFloat), None                               , Some(VertexFormat::X32Y32Z32W32SInt), Some(VertexFormat::X32Y32Z32W32UInt), None                                 , None                                 ],
    /* X32Y32Z32    */ [Some(VertexFormat::X32Y32Z32SFloat   ), None                               , Some(VertexFormat::X32Y32Z32SInt   ), Some(VertexFormat::X32Y32Z32UInt   ), None                                 , None                                 ],
    /* X32Y32       */ [Some(VertexFormat::X32Y32SFloat      ), None                               , Some(VertexFormat::X32Y32SInt      ), Some(VertexFormat::X32Y32UInt      ), None                                 , None                                 ],
    /* X32          */ [Some(VertexFormat::X32SFloat         ), None                               , Some(VertexFormat::X32SInt         ), Some(VertexFormat::X32UInt         ), None                                 , None                                 ],
    /* X16Y16Z16W16 */ [None                                  , None                               , Some(VertexFormat::X16Y16Z16W16SInt), Some(VertexFormat::X16Y16Z16W16UInt), Some(VertexFormat::X16Y16Z16W16SNorm), Some(VertexFormat::X16Y16Z16W16UNorm)],
    /* X16Y16       */ [None                                  , None                               , Some(VertexFormat::X16Y16SInt      ), Some(VertexFormat::X16Y16UInt      ), Some(VertexFormat::X16Y16SNorm      ), Some(VertexFormat::X16Y16UNorm      )],
    /* X16          */ [None                                  , None                               , Some(VertexFormat::X16SInt         ), Some(VertexFormat::X16UInt         ), Some(VertexFormat::X16SNorm         ), Some(VertexFormat::X16UNorm         )],
    /* X8Y8Z8W8     */ [None                                  , None                               , Some(VertexFormat::X8Y8Z8W8SInt    ), Some(VertexFormat::X8Y8Z8W8UInt    ), Some(VertexFormat::X8Y8Z8W8SNorm    ), Some(VertexFormat::X8Y8Z8W8UNorm    )],
    /* X8Y8         */ [None                                  , None                               , Some(VertexFormat::X8Y8SInt        ), Some(VertexFormat::X8Y8UInt        ), Some(VertexFormat::X8Y8SNorm        ), Some(VertexFormat::X8Y8UNorm        )],
    /* X8           */ [None                                  , None                               , Some(VertexFormat::X8SInt          ), Some(VertexFormat::X8UInt          ), Some(VertexFormat::X8SNorm          ), Some(VertexFormat::X8UNorm          )],
    /* X10Y10Z10W2  */ [None                                  , None                               , None                                , Some(VertexFormat::X10Y10Z10W2UInt ), None                                 , Some(VertexFormat::X10Y10Z10W2UNorm )],
    /* X11Y11Z10    */ [None                                  , Some(VertexFormat::X11Y11Z10UFloat), None                                , None                                , None                                 , None                                 ],
];