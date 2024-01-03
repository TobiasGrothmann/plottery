use plottery_lib::{Circle, Layer, V2};

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn generate() -> Layer {
    let mut l = Layer::new();
    l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
    l.push(Circle::new_shape(V2::new(2.0, 2.0), 1.0));
    l.push(Circle::new_shape(V2::new(3.0, 3.0), 1.0));
    l
}
