use plottery_lib::Layer;

#[derive(Debug)]
pub enum Task {
    Plot(Layer),
    Abort,
}
