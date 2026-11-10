use std::io::{BufRead, Cursor, Seek};
use std::path::Path as FilePath;

use ::image::{DynamicImage, ImageDecoder, ImageReader, RgbImage};
use anyhow::{Context, Result};

use crate::{Alignment, ColorRgb, Fit, Histogram, Rect, V2i, V2};

/// What [`Image::color_at`] returns outside of the image bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OutOfBounds {
    #[default]
    Black,
    White,
    Clamp,
    Wrap,
}

/// Raster image placed in plotter space, see [`Image::fit_inside`].
#[derive(Clone, Debug)]
pub struct Image {
    pixels: RgbImage,
    bounds: Rect,
    out_of_bounds: OutOfBounds,
}

impl Image {
    /// Loads a png or jpeg, upright per EXIF. Initially one pixel is one unit, with the bottom left at the origin.
    pub fn load(path: impl AsRef<FilePath>) -> Result<Self> {
        let path = path.as_ref();
        ImageReader::open(path)
            .map_err(anyhow::Error::from)
            .and_then(Self::decode)
            .with_context(|| format!("Failed to load image {}", path.display()))
    }

    /// Decodes a png or jpeg from memory, upright per EXIF.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Self::decode(ImageReader::new(Cursor::new(bytes))).context("Failed to decode image")
    }

    fn decode(reader: ImageReader<impl BufRead + Seek>) -> Result<Self> {
        let mut decoder = reader.with_guessed_format()?.into_decoder()?;
        let orientation = decoder.orientation()?;
        let mut image = DynamicImage::from_decoder(decoder)?;
        image.apply_orientation(orientation);
        Ok(Self::from_rgb(image.to_rgb8()))
    }

    pub(crate) fn from_rgb(pixels: RgbImage) -> Self {
        let bounds = Rect::new(
            V2::zero(),
            V2::new(pixels.width() as f32, pixels.height() as f32),
        );
        Self {
            pixels,
            bounds,
            out_of_bounds: OutOfBounds::default(),
        }
    }

    /// Places the image exactly on `bounds`, ignoring the aspect ratio.
    #[must_use]
    pub fn with_bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    /// Scales the image to fit inside `target`, keeping the aspect ratio.
    #[must_use]
    pub fn fit_inside(mut self, target: Rect, alignment: Alignment) -> Self {
        self.bounds = self
            .bounds
            .fit_inside(target, alignment)
            .expect("target rect must have a size");
        self
    }

    /// Scales the image to cover `target`, keeping the aspect ratio.
    #[must_use]
    pub fn fit_around(mut self, target: Rect, alignment: Alignment) -> Self {
        self.bounds = self
            .bounds
            .fit_around(target, alignment)
            .expect("target rect must have a size");
        self
    }

    #[must_use]
    pub fn out_of_bounds(mut self, out_of_bounds: OutOfBounds) -> Self {
        self.out_of_bounds = out_of_bounds;
        self
    }

    /// Bounds in plotter space.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn pixel_size(&self) -> V2i {
        V2i::new(self.pixels.width() as i32, self.pixels.height() as i32)
    }

    /// Color of a pixel, `(0, 0)` is the top left.
    pub fn pixel(&self, pixel: V2i) -> ColorRgb {
        let size = self.pixel_size();
        let inside = pixel.x >= 0 && pixel.y >= 0 && pixel.x < size.x && pixel.y < size.y;
        match self.out_of_bounds {
            OutOfBounds::Black if !inside => ColorRgb::black(),
            OutOfBounds::White if !inside => ColorRgb::white(),
            _ => self.pixel_clamped_or_wrapped(pixel.x, pixel.y),
        }
    }

    /// Brightness 0..1 at `location`, see [`Image::color_at`].
    pub fn brightness_at(&self, location: V2) -> f32 {
        self.color_at(location).brightness()
    }
    /// Perceived lightness 0..1 at `location`, see [`ColorRgb::lightness`].
    pub fn lightness_at(&self, location: V2) -> f32 {
        self.color_at(location).lightness()
    }
    /// `1 - brightness` at `location`, dark areas give high values.
    pub fn darkness_at(&self, location: V2) -> f32 {
        1.0 - self.brightness_at(location)
    }
    /// The brightness as a scalar field, e.g. for [`crate::MarchingSquares`] or weighted relaxation.
    pub fn brightness_fn(&self) -> impl Fn(V2) -> f32 + '_ {
        move |location| self.brightness_at(location)
    }
    /// The darkness as a scalar field, see [`Image::brightness_fn`].
    pub fn darkness_fn(&self) -> impl Fn(V2) -> f32 + '_ {
        move |location| self.darkness_at(location)
    }
    /// The perceived lightness as a scalar field, see [`Image::brightness_fn`].
    pub fn lightness_fn(&self) -> impl Fn(V2) -> f32 + '_ {
        move |location| self.lightness_at(location)
    }
    /// Any color metric as a scalar field, e.g. `image.field_fn(ColorRgb::luminance)`.
    pub fn field_fn<'a, F: Fn(&ColorRgb) -> f32 + 'a>(
        &'a self,
        metric: F,
    ) -> impl Fn(V2) -> f32 + 'a {
        move |location| metric(&self.color_at(location))
    }

    /// All pixels, row-major starting at the top left.
    pub fn pixels(&self) -> impl Iterator<Item = ColorRgb> + '_ {
        self.pixels.pixels().map(|pixel| to_color(pixel.0))
    }

    /// Histogram of `metric` over every pixel, e.g. `image.histogram(ColorRgb::lightness)`.
    pub fn histogram<F: Fn(&ColorRgb) -> f32>(&self, metric: F) -> Histogram {
        Histogram::from_values(self.pixels().map(|pixel| metric(&pixel)))
    }
    /// Histogram of `metric` sampled on a grid inside `region` (plotter space),
    /// e.g. only the part of the image that is inside the frame.
    pub fn histogram_in<F: Fn(&ColorRgb) -> f32>(
        &self,
        region: Rect,
        max_samples_per_side: usize,
        metric: F,
    ) -> Histogram {
        Histogram::sample_field(region, max_samples_per_side, self.field_fn(metric))
    }

    /// Bilinearly interpolated color at a location in plotter space.
    pub fn color_at(&self, location: V2) -> ColorRgb {
        let mut u = (location.x - self.bounds.bl().x) / self.bounds.width();
        let mut v = (self.bounds.tr().y - location.y) / self.bounds.height();
        let inside = (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v);
        match self.out_of_bounds {
            OutOfBounds::Black if !inside => return ColorRgb::black(),
            OutOfBounds::White if !inside => return ColorRgb::white(),
            OutOfBounds::Wrap => {
                u = u.rem_euclid(1.0);
                v = v.rem_euclid(1.0);
            }
            _ => {
                u = u.clamp(0.0, 1.0);
                v = v.clamp(0.0, 1.0);
            }
        }

        let x = u * self.pixels.width() as f32 - 0.5;
        let y = v * self.pixels.height() as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (tx, ty) = (x - x0, y - y0);
        let (x0, y0) = (x0 as i32, y0 as i32);

        let c00 = self.pixel_clamped_or_wrapped(x0, y0);
        let c10 = self.pixel_clamped_or_wrapped(x0 + 1, y0);
        let c01 = self.pixel_clamped_or_wrapped(x0, y0 + 1);
        let c11 = self.pixel_clamped_or_wrapped(x0 + 1, y0 + 1);
        c00.lerp(c10, tx).lerp(c01.lerp(c11, tx), ty)
    }

    fn pixel_clamped_or_wrapped(&self, x: i32, y: i32) -> ColorRgb {
        let (w, h) = (self.pixels.width() as i32, self.pixels.height() as i32);
        let (x, y) = if self.out_of_bounds == OutOfBounds::Wrap {
            (x.rem_euclid(w), y.rem_euclid(h))
        } else {
            (x.clamp(0, w - 1), y.clamp(0, h - 1))
        };
        to_color(self.pixels.get_pixel(x as u32, y as u32).0)
    }
}

fn to_color([r, g, b]: [u8; 3]) -> ColorRgb {
    ColorRgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
