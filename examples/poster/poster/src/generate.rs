use plottery_lib::*;
use plottery_project::*;

// A title fitted to the page width, a rule and a wrapped paragraph below it.
#[derive(PlotteryParams)]
pub struct Params {
    /// paper size, orientation and margin
    pub paper: Paper,

    #[section("Title")]
    #[value("Plottery".to_string())]
    pub title: String,

    #[value(BundledFont::MistralSingleLine)]
    pub title_font: BundledFont,

    #[section("Text")]
    #[value(PARAGRAPH.to_string())]
    pub text: String,

    #[value(BundledFont::EmsReadability)]
    pub text_font: BundledFont,

    /// em size of the paragraph in cm
    #[value(1.0)]
    #[range(0.2, 2.0)]
    pub text_size: f32,
}

const PARAGRAPH: &str = "Plottery is a generative art engine for pen plotters. \
    A project is a small Rust program that turns parameters into lines. \
    Every parameter shows up in the editor as a slider, switch or picker, \
    and every change redraws the page. \
    Text is set in single-line fonts, so each letter is a few pen strokes \
    and the plotter writes it like a hand would. \
    Change the title, this paragraph or the fonts and the layout follows.";

pub fn generate(params: Params) -> Layer {
    let frame = params.paper.frame();
    let inner = frame.inner_rect();
    let sample_settings = SampleSettings::default();

    // the em size is chosen so the title spans the inner width
    let title = Text::new(params.title.as_str(), &Font::bundled(params.title_font))
        .fit_width(inner.width())
        .in_rect(inner)
        .align(TextAlign::Center)
        .vertical_align(VerticalAlign::Top)
        .to_layer(sample_settings)
        .expect("title");
    let title_bottom = title
        .bounding_box()
        .map_or(inner.tl().y, |bounds| bounds.bl().y);

    let rule_y = title_bottom - params.text_size;
    let rule = Path::line(V2::new(inner.bl().x, rule_y), V2::new(inner.tr().x, rule_y));

    let text_rect = Rect::new(inner.bl(), V2::new(inner.tr().x, rule_y - params.text_size));
    let text = Text::new(params.text.as_str(), &Font::bundled(params.text_font))
        .size(params.text_size)
        .in_rect(text_rect)
        .align(TextAlign::Left)
        .wrap(Wrap::Balanced)
        .vertical_align(VerticalAlign::Top)
        .to_layer(sample_settings)
        .expect("text");

    let mut l = Layer::new();
    l.push(frame.outer_rect());
    l.push(rule);
    l.push_layer(title.with_name("title"));
    l.push_layer(text.with_name("text"));
    l.with_name("root").optimize_recursive()
}
