pub mod base64;
use plottery_lib::{Layer, Shape};

#[derive(Debug)]
pub enum Task {
    PlotShape(Shape),
    Plot(Layer),
    Abort,
}
