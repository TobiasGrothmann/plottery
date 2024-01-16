use plottery_lib::{Circle, Layer, V2};

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn generate() -> Layer {
    let mut l = Layer::new();
    l.push(Circle::new_shape(V2::xy(1.0), 1.0));
    l.push(Circle::new_shape(V2::xy(2.0), 1.0));
    l.push(Circle::new_shape(V2::xy(3.0), 1.0));
    l.push(Circle::new_shape(V2::xy(4.0), 1.0));
    l.push(Circle::new_shape(V2::xy(5.0), 1.0));
    l.push(Circle::new_shape(V2::xy(6.0), 1.0));
    l.push(Circle::new_shape(V2::xy(7.0), 1.0));
    l
}
