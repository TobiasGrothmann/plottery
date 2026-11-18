#![doc = include_str!("../../samples/text/README.md")]

pub(crate) mod bundled;
mod bundled_test;
pub(crate) mod font;
pub(crate) mod glyph;
pub(crate) mod layout;
mod layout_test;
pub(crate) mod metrics;
mod outline_font;
mod outline_font_test;
mod stroke_font;
mod stroke_font_test;
mod svg_glyph;
mod svg_glyph_test;
mod system;
#[allow(clippy::module_inception)]
pub(crate) mod text;
mod text_test;

pub use font::*;
pub use glyph::*;
pub use layout::*;
pub use metrics::*;
pub use plottery_fonts::{BundledFont, Credit, FontKind, License};
pub use text::*;
