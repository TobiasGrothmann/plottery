pub mod bundled;
mod bundled_test;
pub mod font;
pub mod glyph;
pub mod layout;
mod layout_test;
pub mod metrics;
mod outline_font;
mod outline_font_test;
mod stroke_font;
mod stroke_font_test;
mod svg_glyph;
mod svg_glyph_test;
mod system;
#[allow(clippy::module_inception)]
pub mod text;
mod text_test;

pub use font::*;
pub use glyph::*;
pub use layout::*;
pub use metrics::*;
pub use plottery_fonts::{BundledFont, Credit, FontKind, Licence};
pub use text::*;
