use base64::prelude::*;
use std::sync::LazyLock;

const FONTS: [(&str, &str, &[u8]); 2] = [
    (
        "Inter",
        "100 900",
        include_bytes!("../../public/fonts/Inter-Variable.woff2"),
    ),
    (
        "JetBrains Mono",
        "100 800",
        include_bytes!("../../public/fonts/JetBrainsMono-Variable.woff2"),
    ),
];

/// `@font-face` rules for the bundled fonts as base64 data URLs.
pub fn font_face_css() -> &'static str {
    static CSS: LazyLock<String> = LazyLock::new(|| {
        FONTS
        .iter()
        .map(|(family, weight, data)| {
            format!(
                "@font-face{{font-family:'{family}';font-style:normal;font-weight:{weight};font-display:block;src:url(data:font/woff2;base64,{}) format('woff2')}}",
                BASE64_STANDARD.encode(data)
            )
        })
        .collect()
    });
    &CSS
}
