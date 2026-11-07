# plottery_fonts

65 single-line fonts for pen plotters, laser cutters, CNC engravers and anything else that draws strokes instead of filling shapes. All of them are free to redistribute, the licences and credits ship with the crate.

The crate is pure data with no dependencies: every font is embedded with `include_bytes!` and exposed through the `BundledFont` enum. It is made for [Plottery](https://github.com/TobiasGrothmann/plottery), but nothing in it depends on Plottery.

## Usage with Plottery

`plottery_lib` re-exports this crate and turns the fonts into geometry:

```rust
use plottery_lib::*;

let font = Font::bundled(BundledFont::HersheySans);
let layer = Text::new("Hello plotter", &font)
    .size(2.0) // em height in cm
    .to_layer(SampleSettings::default())?;
layer.write_svg("hello.svg".into(), 10.0)?;
```

## Usage without Plottery

Pick a font and feed the bytes to your own parser. `kind()` tells you what the bytes are:

```rust
use plottery_fonts::{BundledFont, FontKind};

let font = BundledFont::ReliefSingleLine;
let bytes: &[u8] = font.bytes();

match font.kind() {
    FontKind::Stroke => {}     // SVG 1.1 font: <glyph d="..."> strokes, plus <hkern> pairs
    FontKind::SingleLine => {} // TrueType whose contours are strokes: do not close them
    FontKind::Outline => {}    // TTF/OTF, the strokes live in its OpenType `SVG ` table
}

for font in BundledFont::ALL {
    let credit = font.credit();
    println!("{} by {} ({})", font.family(), credit.author, credit.licence.name());
}
```

`file_name()` gives the original file name, `credit()` the author, source URL and licence, and `Licence::text()` the full licence text for your "about" screen.

## Licences

Everything is under the [SIL Open Font License 1.1](https://github.com/TobiasGrothmann/plottery/blob/main/fonts/LICENSES/OFL.txt) or the [Hershey font licence](https://github.com/TobiasGrothmann/plottery/blob/main/fonts/LICENSES/HERSHEY.txt). Both allow commercial use and bundling. The Hershey licence asks that its acknowledgement travels with the data, so keep `LICENSES/` (or `Licence::Hershey.text()`) in anything you ship. Per-font credits are in [LICENSES/CREDITS.md](https://github.com/TobiasGrothmann/plottery/blob/main/fonts/LICENSES/CREDITS.md).

## Fonts

### Hershey

The classic plotter fonts by Dr. A. V. Hershey, SVG conversion by Windell H. Oskay. Latin-1 charset.

| Font | Sample |
|---|---|
| `HersheyGothicEnglish` | ![Hershey Gothic English](samples/HersheyGothicEnglish.svg) |
| `HersheyGothicGerman` | ![Hershey Gothic German](samples/HersheyGothicGerman.svg) |
| `HersheyGothicItalian` | ![Hershey Gothic Italian](samples/HersheyGothicItalian.svg) |
| `HersheySans` | ![Hershey Sans 1-stroke](samples/HersheySans.svg) |
| `HersheySansMedium` | ![Hershey Sans medium](samples/HersheySansMedium.svg) |
| `HersheyScript` | ![Hershey Script 1-stroke](samples/HersheyScript.svg) |
| `HersheyScriptMedium` | ![Hershey Script medium](samples/HersheyScriptMedium.svg) |
| `HersheySerif` | ![Hershey Serif medium](samples/HersheySerif.svg) |
| `HersheySerifItalic` | ![Hershey Serif medium italic](samples/HersheySerifItalic.svg) |
| `HersheySerifBold` | ![Hershey Serif bold](samples/HersheySerifBold.svg) |
| `HersheySerifBoldItalic` | ![Hershey Serif bold italic](samples/HersheySerifBoldItalic.svg) |

### Hershey extras

More Hershey sets converted by Shriinivas. Greek, Japanese and the markers map onto Latin letters, the symbol fonts (astrology, math, meteorology, music, symbolic) keep plain letters and put their symbols on digits and punctuation, Cyrillic uses real Cyrillic code points.

| Font | Sample |
|---|---|
| `HersheySansBold` | ![Hershey-Sans bold](samples/HersheySansBold.svg) |
| `HersheyScriptAlt` | ![Hershey-Script 1-stroke (alt)](samples/HersheyScriptAlt.svg) |
| `HersheyAstrology` | ![Hershey-Astrology](samples/HersheyAstrology.svg) |
| `HersheyCyrillic` | ![Hershey-Cyrillic](samples/HersheyCyrillic.svg) |
| `HersheyGreek` | ![Hershey-Greek 1-stroke](samples/HersheyGreek.svg) |
| `HersheyGreekMedium` | ![Hershey-Greek medium](samples/HersheyGreekMedium.svg) |
| `HersheyJapanese` | ![Hershey-Japanese](samples/HersheyJapanese.svg) |
| `HersheyMarkers` | ![Hershey-Markers](samples/HersheyMarkers.svg) |
| `HersheyMathLower` | ![Hershey-Math (lower)](samples/HersheyMathLower.svg) |
| `HersheyMathUpper` | ![Hershey-Math (upper)](samples/HersheyMathUpper.svg) |
| `HersheyMeteorology` | ![Hershey-Meteorology](samples/HersheyMeteorology.svg) |
| `HersheyMusic` | ![Hershey-Music](samples/HersheyMusic.svg) |
| `HersheySymbolic` | ![Hershey-Symbolic](samples/HersheySymbolic.svg) |

### EMS

Fonts drawn for plotters by Sheldon B. Michaels and Windell H. Oskay at Evil Mad Scientist Laboratories.

| Font | Sample |
|---|---|
| `EmsAllure` | ![EMS Allure](samples/EmsAllure.svg) |
| `EmsBird` | ![EMS Bird](samples/EmsBird.svg) |
| `EmsBirdSwashCaps` | ![EMS Bird Swash Caps](samples/EmsBirdSwashCaps.svg) |
| `EmsBrush` | ![EMS Brush](samples/EmsBrush.svg) |
| `EmsCapitol` | ![EMS Capitol](samples/EmsCapitol.svg) |
| `EmsCasualHand` | ![EMS Casual Hand](samples/EmsCasualHand.svg) |
| `EmsDecorousScript` | ![EMS Decorous Script](samples/EmsDecorousScript.svg) |
| `EmsDelight` | ![EMS Delight](samples/EmsDelight.svg) |
| `EmsDelightSwashCaps` | ![EMS Delight Swash Caps](samples/EmsDelightSwashCaps.svg) |
| `EmsElfin` | ![EMS Elfin](samples/EmsElfin.svg) |
| `EmsFelix` | ![EMS Felix](samples/EmsFelix.svg) |
| `EmsHerculean` | ![EMS Herculean](samples/EmsHerculean.svg) |
| `EmsInvite` | ![EMS Invite](samples/EmsInvite.svg) |
| `EmsLeague` | ![EMS League](samples/EmsLeague.svg) |
| `EmsLittlePrincess` | ![EMS Little Princess](samples/EmsLittlePrincess.svg) |
| `EmsMistyNight` | ![EMS Misty Night](samples/EmsMistyNight.svg) |
| `EmsNeato` | ![EMS Neato](samples/EmsNeato.svg) |
| `EmsNixish` | ![EMS Nixish](samples/EmsNixish.svg) |
| `EmsNixishItalic` | ![EMS Nixish Italic](samples/EmsNixishItalic.svg) |
| `EmsOsmotron` | ![EMS Osmotron](samples/EmsOsmotron.svg) |
| `EmsPancakes` | ![EMS Pancakes](samples/EmsPancakes.svg) |
| `EmsPepita` | ![EMS Pepita](samples/EmsPepita.svg) |
| `EmsQwandry` | ![EMS Qwandry](samples/EmsQwandry.svg) |
| `EmsReadability` | ![EMS Readability](samples/EmsReadability.svg) |
| `EmsReadabilityItalic` | ![EMS Readability Italic](samples/EmsReadabilityItalic.svg) |
| `EmsSociety` | ![EMS Society](samples/EmsSociety.svg) |
| `EmsSwiss` | ![EMS Swiss](samples/EmsSwiss.svg) |
| `EmsTech` | ![EMS Tech](samples/EmsTech.svg) |
| `TwinSans` | ![Twin Sans](samples/TwinSans.svg) |

### Cutlings

Single-stroke fonts by Ellen Wasbo.

| Font | Sample |
|---|---|
| `CutlingsDualis` | ![CutlingsDualis](samples/CutlingsDualis.svg) |
| `CutlingsGeometric` | ![CutlingsGeometric](samples/CutlingsGeometric.svg) |
| `CutlingsGeometricRound` | ![CutlingsGeometricRound](samples/CutlingsGeometricRound.svg) |
| `CutlingsPluralis` | ![CutlingsPluralis](samples/CutlingsPluralis.svg) |
| `CutlingsSingularis` | ![CutlingsSingularis](samples/CutlingsSingularis.svg) |

### Custom

Stroke fonts by Shriinivas, derived from Square Grotesk and Pinyon Script.

| Font | Sample |
|---|---|
| `CustomScript` | ![Custom-Script](samples/CustomScript.svg) |
| `CustomSquare` | ![Custom-Square Normal](samples/CustomSquare.svg) |
| `CustomSquareItalic` | ![Custom-Square Italic](samples/CustomSquareItalic.svg) |

### Single-line TrueType and OpenType-SVG

Real font files with proper kerning. Relief stores its strokes as open TrueType contours, Mistral and Zniko keep them in an OpenType `SVG` table.

| Font | Sample |
|---|---|
| `ReliefSingleLine` | ![Relief SingleLine](samples/ReliefSingleLine.svg) |
| `ReliefSingleLineOrnament` | ![Relief SingleLine Ornament](samples/ReliefSingleLineOrnament.svg) |
| `MistralSingleLine` | ![Mistral SingleLine OTF-SVG Regular](samples/MistralSingleLine.svg) |
| `Zniko` | ![ZnikoSLSVGinOT-8](samples/Zniko.svg) |
