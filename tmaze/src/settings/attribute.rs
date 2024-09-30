use std::{fmt, str::FromStr};

use crossterm::style::Attributes;
use serde::Deserialize;

macro_rules! Attribute {
    (
        $(
            $(#[$inner:ident $($args:tt)*])*
            $name:ident = $sgr:expr,
        )*
    ) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize)]
        #[non_exhaustive]
        #[serde(rename_all = "snake_case")]
        pub enum Attribute {
            $(
                $(#[$inner $($args)*])*
                $name,
            )*
        }

        impl Attribute {
            pub fn iterator() -> impl Iterator<Item = Attribute> {
                use self::Attribute::*;
                [ $($name,)* ].into_iter()
            }
        }
    }
}

Attribute! {
    /// Resets all the attributes.
    Reset = 0,
    /// Increases the text intensity.
    Bold = 1,
    /// Decreases the text intensity.
    Dim = 2,
    /// Emphasises the text.
    Italic = 3,
    /// Underlines the text.
    Underlined = 4,

    // Other types of underlining
    /// Double underlines the text.
    DoubleUnderlined = 2,
    /// Undercurls the text.
    Undercurled = 3,
    /// Underdots the text.
    Underdotted = 4,
    /// Underdashes the text.
    Underdashed = 5,

    /// Makes the text blinking (< 150 per minute).
    SlowBlink = 5,
    /// Makes the text blinking (>= 150 per minute).
    RapidBlink = 6,
    /// Swaps foreground and background colors.
    Reverse = 7,
    /// Hides the text (also known as Conceal).
    Hidden = 8,
    /// Crosses the text.
    CrossedOut = 9,
    /// Sets the [Fraktur](https://en.wikipedia.org/wiki/Fraktur) typeface.
    ///
    /// Mostly used for [mathematical alphanumeric symbols](https://en.wikipedia.org/wiki/Mathematical_Alphanumeric_Symbols).
    Fraktur = 20,
    /// Turns off the `Bold` attribute. - Inconsistent - Prefer to use NormalIntensity
    NoBold = 21,
    /// Switches the text back to normal intensity (no bold, italic).
    NormalIntensity = 22,
    /// Turns off the `Italic` attribute.
    NoItalic = 23,
    /// Turns off the `Underlined` attribute.
    NoUnderline = 24,
    /// Turns off the text blinking (`SlowBlink` or `RapidBlink`).
    NoBlink = 25,
    /// Turns off the `Reverse` attribute.
    NoReverse = 27,
    /// Turns off the `Hidden` attribute.
    NoHidden = 28,
    /// Turns off the `CrossedOut` attribute.
    NotCrossedOut = 29,
    /// Makes the text framed.
    Framed = 51,
    /// Makes the text encircled.
    Encircled = 52,
    /// Draws a line at the top of the text.
    OverLined = 53,
    /// Turns off the `Frame` and `Encircled` attributes.
    NotFramedOrEncircled = 54,
    /// Turns off the `OverLined` attribute.
    NotOverLined = 55,
}

impl From<Attribute> for crossterm::style::Attribute {
    fn from(value: Attribute) -> Self {
        use crossterm::style;
        use Attribute::*;

        match value {
            Reset => style::Attribute::Reset,
            Bold => style::Attribute::Bold,
            Dim => style::Attribute::Dim,
            Italic => style::Attribute::Italic,
            Underlined => style::Attribute::Underlined,
            DoubleUnderlined => style::Attribute::DoubleUnderlined,
            Undercurled => style::Attribute::Undercurled,
            Underdotted => style::Attribute::Underdotted,
            Underdashed => style::Attribute::Underdashed,
            SlowBlink => style::Attribute::SlowBlink,
            RapidBlink => style::Attribute::RapidBlink,
            Reverse => style::Attribute::Reverse,
            Hidden => style::Attribute::Hidden,
            CrossedOut => style::Attribute::CrossedOut,
            Fraktur => style::Attribute::Fraktur,
            NoBold => style::Attribute::NoBold,
            NormalIntensity => style::Attribute::NormalIntensity,
            NoItalic => style::Attribute::NoItalic,
            NoUnderline => style::Attribute::NoUnderline,
            NoBlink => style::Attribute::NoBlink,
            NoReverse => style::Attribute::NoReverse,
            NoHidden => style::Attribute::NoHidden,
            NotCrossedOut => style::Attribute::NotCrossedOut,
            Framed => style::Attribute::Framed,
            Encircled => style::Attribute::Encircled,
            OverLined => style::Attribute::OverLined,
            NotFramedOrEncircled => style::Attribute::NotFramedOrEncircled,
            NotOverLined => style::Attribute::NotOverLined,
        }
    }
}

impl FromStr for Attribute {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "reset" => Ok(Attribute::Reset),
            "bold" => Ok(Attribute::Bold),
            "dim" => Ok(Attribute::Dim),
            "italic" => Ok(Attribute::Italic),
            "underlined" => Ok(Attribute::Underlined),
            "double_underlined" => Ok(Attribute::DoubleUnderlined),
            "undercurled" => Ok(Attribute::Undercurled),
            "underdotted" => Ok(Attribute::Underdotted),
            "underdashed" => Ok(Attribute::Underdashed),
            "slow_blink" => Ok(Attribute::SlowBlink),
            "rapid_blink" => Ok(Attribute::RapidBlink),
            "reverse" => Ok(Attribute::Reverse),
            "hidden" => Ok(Attribute::Hidden),
            "crossed_out" => Ok(Attribute::CrossedOut),
            "fraktur" => Ok(Attribute::Fraktur),
            "no_bold" => Ok(Attribute::NoBold),
            "normal_intensity" => Ok(Attribute::NormalIntensity),
            "no_italic" => Ok(Attribute::NoItalic),
            "no_underline" => Ok(Attribute::NoUnderline),
            "no_blink" => Ok(Attribute::NoBlink),
            "no_reverse" => Ok(Attribute::NoReverse),
            "no_hidden" => Ok(Attribute::NoHidden),
            "not_crossed_out" => Ok(Attribute::NotCrossedOut),
            "framed" => Ok(Attribute::Framed),
            "encircled" => Ok(Attribute::Encircled),
            "overlined" => Ok(Attribute::OverLined),
            "not_framed_or_encircled" => Ok(Attribute::NotFramedOrEncircled),
            "not_overlined" => Ok(Attribute::NotOverLined),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Attribute::Reset => write!(f, "reset"),
            Attribute::Bold => write!(f, "bold"),
            Attribute::Dim => write!(f, "dim"),
            Attribute::Italic => write!(f, "italic"),
            Attribute::Underlined => write!(f, "underlined"),
            Attribute::DoubleUnderlined => write!(f, "double_underlined"),
            Attribute::Undercurled => write!(f, "undercurled"),
            Attribute::Underdotted => write!(f, "underdotted"),
            Attribute::Underdashed => write!(f, "underdashed"),
            Attribute::SlowBlink => write!(f, "slow_blink"),
            Attribute::RapidBlink => write!(f, "rapid_blink"),
            Attribute::Reverse => write!(f, "reverse"),
            Attribute::Hidden => write!(f, "hidden"),
            Attribute::CrossedOut => write!(f, "crossed_out"),
            Attribute::Fraktur => write!(f, "fraktur"),
            Attribute::NoBold => write!(f, "no_bold"),
            Attribute::NormalIntensity => write!(f, "normal_intensity"),
            Attribute::NoItalic => write!(f, "no_italic"),
            Attribute::NoUnderline => write!(f, "no_underline"),
            Attribute::NoBlink => write!(f, "no_blink"),
            Attribute::NoReverse => write!(f, "no_reverse"),
            Attribute::NoHidden => write!(f, "no_hidden"),
            Attribute::NotCrossedOut => write!(f, "not_crossed_out"),
            Attribute::Framed => write!(f, "framed"),
            Attribute::Encircled => write!(f, "encircled"),
            Attribute::OverLined => write!(f, "overlined"),
            Attribute::NotFramedOrEncircled => write!(f, "not_framed_or_encircled"),
            Attribute::NotOverLined => write!(f, "not_overlined"),
        }
    }
}

pub fn deserialize_attributes<'de, D>(deserializer: D) -> Result<Attributes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer).map(|vec| {
        let mut attributes = Attributes::default();
        for attr in vec {
            attributes.set(
                match Attribute::from_str(&attr) {
                    Ok(t) => t,
                    Err(_) => panic!(
                        "could not decode attribute: {}, valid attributes: {:?}",
                        attr,
                        Attribute::iterator()
                            .map(|a| a.to_string())
                            .collect::<Vec<_>>() // TODO: print similar attributes
                    ),
                }
                .into(),
            );
        }
        attributes
    })
}
