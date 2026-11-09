//! Plotter fonts bundled with Plottery; data only, `plottery_lib` loads them with `Font::bundled`.
//!
//! Licences are in `LICENSES/`, credits per font via [`BundledFont::credit`].

/// How the font data has to be loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontKind {
    /// SVG 1.1 font with `<glyph d="...">` strokes.
    Stroke,
    /// TrueType whose strokes are stored as contours that must stay open.
    SingleLine,
    /// Regular TTF/OTF, possibly with the strokes in an OpenType `SVG ` table.
    Outline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Licence {
    /// SIL Open Font License 1.1.
    Ofl,
    /// Hershey fonts: the acknowledgement must ship with the data.
    Hershey,
}

impl Licence {
    pub fn name(self) -> &'static str {
        match self {
            Licence::Ofl => "SIL Open Font License 1.1",
            Licence::Hershey => "Hershey font licence",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Licence::Ofl => include_str!("../LICENSES/OFL.txt"),
            Licence::Hershey => include_str!("../LICENSES/HERSHEY.txt"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Credit {
    pub author: &'static str,
    pub source: &'static str,
    pub licence: Licence,
}

const A_OSKAY_HERSHEY: &str =
    "Dr. A. V. Hershey (U.S. National Bureau of Standards), SVG conversion by Windell H. Oskay";
const A_SHRI_HERSHEY: &str =
    "Dr. A. V. Hershey (U.S. National Bureau of Standards), SVG conversion by Shriinivas";
const A_EMS: &str = "Sheldon B. Michaels and Windell H. Oskay, Evil Mad Scientist Laboratories";
const A_WASBO: &str = "Ellen Wasbo";
const A_SHRI: &str = "Shriinivas, derived from Square Grotesk and Pinyon Script (fontlibrary.org)";
const A_RELIEF: &str = "The Relief SingleLine Project Authors";
const A_MISTRAL: &str = "The Mistral SingleLine Project Authors";
const A_GLUK: &str = "gluk";

macro_rules! bundled_fonts {
    ($($variant:ident: $file:literal, $kind:ident, $family:literal, $author:expr, $source:literal, $licence:ident;)*) => {
        /// Every bundled font; see `LICENSES/CREDITS.md`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum BundledFont {
            $(#[doc = $family] $variant,)*
        }

        impl BundledFont {
            pub const ALL: &'static [BundledFont] = &[$(BundledFont::$variant),*];

            /// Variant name, stable across reorderings; used to store the font in params.
            pub fn name(self) -> &'static str {
                match self { $(Self::$variant => stringify!($variant),)* }
            }
            pub fn from_name(name: &str) -> Option<BundledFont> {
                match name { $(stringify!($variant) => Some(Self::$variant),)* _ => None }
            }

            pub fn file_name(self) -> &'static str {
                match self { $(Self::$variant => $file,)* }
            }
            pub fn bytes(self) -> &'static [u8] {
                match self { $(Self::$variant => include_bytes!(concat!("../fonts/", $file)),)* }
            }
            pub fn kind(self) -> FontKind {
                match self { $(Self::$variant => FontKind::$kind,)* }
            }
            /// Family name as stored in the font.
            pub fn family(self) -> &'static str {
                match self { $(Self::$variant => $family,)* }
            }
            pub fn credit(self) -> Credit {
                match self {
                    $(Self::$variant => Credit { author: $author, source: $source, licence: Licence::$licence },)*
                }
            }
        }
    };
}

bundled_fonts! {
    HersheyGothicEnglish: "HersheyGothEnglish.svg", Stroke, "Hershey Gothic English", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheyGothicGerman: "HersheyGothGerman.svg", Stroke, "Hershey Gothic German", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheyGothicItalian: "HersheyGothItalian.svg", Stroke, "Hershey Gothic Italian", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySans: "HersheySans1.svg", Stroke, "Hershey Sans 1-stroke", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySansMedium: "HersheySansMed.svg", Stroke, "Hershey Sans medium", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheyScript: "HersheyScript1.svg", Stroke, "Hershey Script 1-stroke", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheyScriptMedium: "HersheyScriptMed.svg", Stroke, "Hershey Script medium", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySerif: "HersheySerifMed.svg", Stroke, "Hershey Serif medium", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySerifItalic: "HersheySerifMedItalic.svg", Stroke, "Hershey Serif medium italic", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySerifBold: "HersheySerifBold.svg", Stroke, "Hershey Serif bold", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySerifBoldItalic: "HersheySerifBoldItalic.svg", Stroke, "Hershey Serif bold italic", A_OSKAY_HERSHEY, "https://gitlab.com/oskay/svg-fonts", Hershey;
    HersheySansBold: "HersheySansBold.svg", Stroke, "Hershey-Sans bold", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyScriptAlt: "HersheyScriptAlt.svg", Stroke, "Hershey-Script 1-stroke (alt)", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyAstrology: "HersheyAstrology.svg", Stroke, "Hershey-Astrology", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyCyrillic: "HersheyCyrillic.svg", Stroke, "Hershey-Cyrillic", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyGreek: "HersheyGreek.svg", Stroke, "Hershey-Greek 1-stroke", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyGreekMedium: "HersheyGreekMedium.svg", Stroke, "Hershey-Greek medium", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyJapanese: "HersheyJapanese.svg", Stroke, "Hershey-Japanese", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyMarkers: "HersheyMarkers.svg", Stroke, "Hershey-Markers", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyMathLower: "HersheyMathLower.svg", Stroke, "Hershey-Math (lower)", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyMathUpper: "HersheyMathUpper.svg", Stroke, "Hershey-Math (upper)", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyMeteorology: "HersheyMeteorology.svg", Stroke, "Hershey-Meteorology", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheyMusic: "HersheyMusic.svg", Stroke, "Hershey-Music", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    HersheySymbolic: "HersheySymbolic.svg", Stroke, "Hershey-Symbolic", A_SHRI_HERSHEY, "https://github.com/Shriinivas/inkscapestrokefont", Hershey;
    EmsAllure: "EMSAllure.svg", Stroke, "EMS Allure", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsBird: "EMSBird.svg", Stroke, "EMS Bird", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsBirdSwashCaps: "EMSBirdSwashCaps.svg", Stroke, "EMS Bird Swash Caps", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsBrush: "EMSBrush.svg", Stroke, "EMS Brush", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsCapitol: "EMSCapitol.svg", Stroke, "EMS Capitol", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsCasualHand: "EMSCasualHand.svg", Stroke, "EMS Casual Hand", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsDecorousScript: "EMSDecorousScript.svg", Stroke, "EMS Decorous Script", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsDelight: "EMSDelight.svg", Stroke, "EMS Delight", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsDelightSwashCaps: "EMSDelightSwashCaps.svg", Stroke, "EMS Delight Swash Caps", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsElfin: "EMSElfin.svg", Stroke, "EMS Elfin", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsFelix: "EMSFelix.svg", Stroke, "EMS Felix", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsHerculean: "EMSHerculean.svg", Stroke, "EMS Herculean", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsInvite: "EMSInvite.svg", Stroke, "EMS Invite", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsLeague: "EMSLeague.svg", Stroke, "EMS League", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsLittlePrincess: "EMSLittlePrincess.svg", Stroke, "EMS Little Princess", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsMistyNight: "EMSMistyNight.svg", Stroke, "EMS Misty Night", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsNeato: "EMSNeato.svg", Stroke, "EMS Neato", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsNixish: "EMSNixish.svg", Stroke, "EMS Nixish", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsNixishItalic: "EMSNixishItalic.svg", Stroke, "EMS Nixish Italic", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsOsmotron: "EMSOsmotron.svg", Stroke, "EMS Osmotron", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsPancakes: "EMSPancakes.svg", Stroke, "EMS Pancakes", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsPepita: "EMSPepita.svg", Stroke, "EMS Pepita", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsQwandry: "EMSQwandry.svg", Stroke, "EMS Qwandry", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsReadability: "EMSReadability.svg", Stroke, "EMS Readability", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsReadabilityItalic: "EMSReadabilityItalic.svg", Stroke, "EMS Readability Italic", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsSociety: "EMSSociety.svg", Stroke, "EMS Society", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsSwiss: "EMSSwiss.svg", Stroke, "EMS Swiss", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    EmsTech: "EMSTech.svg", Stroke, "EMS Tech", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    TwinSans: "TwinSans.svg", Stroke, "Twin Sans", A_EMS, "https://gitlab.com/oskay/svg-fonts", Ofl;
    CutlingsDualis: "CutlingsDualis.svg", Stroke, "CutlingsDualis", A_WASBO, "http://cutlings.wasbo.net/", Ofl;
    CutlingsGeometric: "CutlingsGeometric.svg", Stroke, "CutlingsGeometric", A_WASBO, "http://cutlings.wasbo.net/", Ofl;
    CutlingsGeometricRound: "CutlingsGeometricRound.svg", Stroke, "CutlingsGeometricRound", A_WASBO, "http://cutlings.wasbo.net/", Ofl;
    CutlingsPluralis: "CutlingsPluralis.svg", Stroke, "CutlingsPluralis", A_WASBO, "http://cutlings.wasbo.net/", Ofl;
    CutlingsSingularis: "CutlingsSingularis.svg", Stroke, "CutlingsSingularis", A_WASBO, "http://cutlings.wasbo.net/", Ofl;
    CustomScript: "CustomScript.svg", Stroke, "Custom-Script", A_SHRI, "https://github.com/Shriinivas/inkscapestrokefont", Ofl;
    CustomSquare: "CustomSquare.svg", Stroke, "Custom-Square Normal", A_SHRI, "https://github.com/Shriinivas/inkscapestrokefont", Ofl;
    CustomSquareItalic: "CustomSquareItalic.svg", Stroke, "Custom-Square Italic", A_SHRI, "https://github.com/Shriinivas/inkscapestrokefont", Ofl;
    ReliefSingleLine: "ReliefSingleLine-Regular.ttf", SingleLine, "Relief SingleLine", A_RELIEF, "https://github.com/isdat-type/Relief-SingleLine", Ofl;
    ReliefSingleLineOrnament: "ReliefSingleLine-Ornament.ttf", SingleLine, "Relief SingleLine Ornament", A_RELIEF, "https://github.com/isdat-type/Relief-SingleLine", Ofl;
    MistralSingleLine: "MistralSingleLine-Regular.otf", Outline, "Mistral SingleLine OTF-SVG Regular", A_MISTRAL, "https://github.com/isdat-type/Mistral-SingleLine", Ofl;
    Zniko: "Znikoslsvginot8.ttf", Outline, "ZnikoSLSVGinOT-8", A_GLUK, "http://www.glukfonts.pl", Ofl;
}

#[cfg(test)]
mod lib_test;
