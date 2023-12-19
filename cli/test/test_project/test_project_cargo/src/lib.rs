use plottery_lib::{Circle, Layer, V2};

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn generate() -> Layer {
    let mut l = Layer::new();
    l.push(Circle::new(V2::new(0.0, 0.0), 1.0));
    l
}
