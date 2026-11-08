use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Frame, Rect, V2};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    /// DIN A series, e.g. `DinA(4)` for A4.
    DinA(u8),
    /// DIN B series, e.g. `DinB(4)` for B4.
    DinB(u8),
    /// Width and height in cm.
    Custom(V2),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Margin {
    Cm(f32),
    /// Fraction of the shorter side.
    Fraction(f32),
}

/// Paper size, orientation and margin. Defaults to A4 portrait with a 10% margin.
///
/// ```
/// # use plottery_lib::*;
/// let paper = Paper::default();
/// assert_eq!(paper.size(), V2::a4());
/// let drawable = paper.frame().inner_rect();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Paper {
    pub size: PaperSize,
    pub landscape: bool,
    pub margin: Margin,
}

impl Default for Paper {
    fn default() -> Self {
        Self {
            size: PaperSize::DinA(4),
            landscape: false,
            margin: Margin::Fraction(0.1),
        }
    }
}

impl Paper {
    pub fn new(size: PaperSize, landscape: bool, margin: Margin) -> Self {
        Self {
            size,
            landscape,
            margin,
        }
    }
    /// Portrait DIN A paper with the default margin.
    pub fn din_a(number: u8) -> Self {
        Self {
            size: PaperSize::DinA(number),
            ..Self::default()
        }
    }
    /// Portrait DIN B paper with the default margin.
    pub fn din_b(number: u8) -> Self {
        Self {
            size: PaperSize::DinB(number),
            ..Self::default()
        }
    }
    /// Custom paper size in cm with the default margin.
    pub fn custom(size: V2) -> Self {
        Self {
            size: PaperSize::Custom(size),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_landscape(self, landscape: bool) -> Self {
        Self { landscape, ..self }
    }
    #[must_use]
    pub fn with_margin(self, margin: Margin) -> Self {
        Self { margin, ..self }
    }

    /// Size in cm. DIN sizes follow `landscape`, custom sizes swap their axes when it is set.
    pub fn size(&self) -> V2 {
        let raw = match self.size {
            PaperSize::DinA(number) => V2::din_a(number.min(10)),
            PaperSize::DinB(number) => V2::din_b(number.min(10)),
            PaperSize::Custom(size) => {
                return if self.landscape { size.swap_axes() } else { size };
            }
        };
        let (short, long) = (raw.min_axis(), raw.max_axis());
        if self.landscape {
            V2::new(long, short)
        } else {
            V2::new(short, long)
        }
    }

    pub fn margin_cm(&self) -> f32 {
        match self.margin {
            Margin::Cm(cm) => cm,
            Margin::Fraction(fraction) => self.size().min_axis() * fraction,
        }
    }

    /// The full paper with its bottom left corner at the origin.
    pub fn rect(&self) -> Rect {
        Rect::new(V2::zero(), self.size())
    }

    /// The paper as a [`Frame`], `inner_rect()` is the area inside the margin.
    pub fn frame(&self) -> Frame {
        Frame::new_xy(self.size(), self.margin_cm())
    }
}

impl From<&Paper> for Frame {
    fn from(paper: &Paper) -> Self {
        paper.frame()
    }
}

impl From<Paper> for Frame {
    fn from(paper: Paper) -> Self {
        paper.frame()
    }
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.size {
            PaperSize::DinA(number) => write!(f, "A{}", number)?,
            PaperSize::DinB(number) => write!(f, "B{}", number)?,
            PaperSize::Custom(_) => {
                let size = self.size();
                return write!(f, "{} × {} cm", size.x, size.y);
            }
        }
        if self.landscape {
            write!(f, " landscape")?;
        }
        Ok(())
    }
}
