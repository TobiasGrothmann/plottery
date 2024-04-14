use base64::prelude::*;

pub fn format_svg(svg: &'static [u8]) -> String {
    format!("data:image/svg+xml;base64,{}", &BASE64_STANDARD.encode(svg))
}
