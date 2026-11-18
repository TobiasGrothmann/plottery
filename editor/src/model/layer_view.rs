use std::collections::{BTreeSet, HashMap};

use plottery_lib::{ColorRgb, Layer, LayerPropsInheritable, Plottable, Shape};
use serde::{Deserialize, Serialize};

/// Groups with more sublayers than this start collapsed.
const DEFAULT_EXPANDED_MAX_SUBLAYERS: usize = 20;
/// Rows shown per expanded group, the rest is summed up in one row.
const MAX_ROWS_PER_GROUP: usize = 200;

/// One step from a layer to a sublayer: its name and its position among the siblings of that name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayerKeySegment {
    pub name: Option<String>,
    pub nth: usize,
}

/// Path of a layer from the root, which is the empty path. Holds across runs while names and order do.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayerKey(pub Vec<LayerKeySegment>);

impl LayerKey {
    fn child(&self, segment: LayerKeySegment) -> Self {
        Self(
            self.0
                .iter()
                .cloned()
                .chain(std::iter::once(segment))
                .collect(),
        )
    }

    /// True for `other` itself and for every layer above it.
    fn is_on_path_to(&self, other: &LayerKey) -> bool {
        other.0.starts_with(&self.0)
    }

    fn is_root(&self) -> bool {
        self.0.is_empty()
    }
}

/// Keys of the sublayers of `layer`, in their order.
fn sublayer_keys<'a>(key: &'a LayerKey, layer: &'a Layer) -> impl Iterator<Item = LayerKey> + 'a {
    let mut seen: HashMap<Option<&str>, usize> = HashMap::new();
    layer.sublayers.iter().map(move |sublayer| {
        let name = sublayer.props.name.as_deref();
        let nth = seen.entry(name).or_default();
        let segment = LayerKeySegment {
            name: name.map(str::to_string),
            nth: *nth,
        };
        *nth += 1;
        key.child(segment)
    })
}

/// Own shapes get a row and a flag of their own where the layer row can't stand for them:
/// next to sublayers, and in the root, which has no row.
fn has_own_shapes_row(key: &LayerKey, sublayers: usize) -> bool {
    key.is_root() || sublayers > 0
}

/// What a row of the layers panel shows and hides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowTarget {
    /// A layer with everything in it.
    Layer(LayerKey),
    /// Only the shapes a layer holds itself.
    OwnShapes(LayerKey),
}

impl RowTarget {
    pub fn key(&self) -> &LayerKey {
        match self {
            RowTarget::Layer(key) | RowTarget::OwnShapes(key) => key,
        }
    }
}

/// What reaches a layer from above while walking down the tree.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Scope {
    /// Drawn, unless a flag hides it.
    Shown,
    /// Not drawn: an ancestor is hidden, or it is outside the solo.
    Hidden,
    /// Not drawn, but the soloed row is further down.
    AboveSolo,
}

/// Which layers are hidden. Flags of a layer stay when a parent hides it, or while another row is soloed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerVisibility {
    pub hidden: BTreeSet<LayerKey>,
    pub hidden_own_shapes: BTreeSet<LayerKey>,
    /// The only row that is drawn, whatever the flags above it say.
    #[serde(skip)]
    pub solo: Option<RowTarget>,
}

impl LayerVisibility {
    /// True when some shapes may be missing from the drawing.
    pub fn hides_anything(&self) -> bool {
        self.solo.is_some() || !self.hidden.is_empty() || !self.hidden_own_shapes.is_empty()
    }

    /// Flips the flag of a row. On the soloed row it ends the solo instead.
    pub fn toggle(&mut self, target: &RowTarget) {
        if self.solo.as_ref() == Some(target) {
            self.solo = None;
            return;
        }
        let (flags, key) = match target {
            RowTarget::Layer(key) => (&mut self.hidden, key),
            RowTarget::OwnShapes(key) => (&mut self.hidden_own_shapes, key),
        };
        if !flags.remove(key) {
            flags.insert(key.clone());
        }
    }

    pub fn toggle_solo(&mut self, target: RowTarget) {
        self.solo = (self.solo.as_ref() != Some(&target)).then_some(target);
    }

    pub fn show_all(&mut self) {
        *self = Self::default();
    }

    /// The scope above the root layer.
    fn outer_scope(&self) -> Scope {
        match self.solo {
            Some(_) => Scope::AboveSolo,
            None => Scope::Shown,
        }
    }

    fn layer_scope(&self, key: &LayerKey, parent: Scope) -> Scope {
        match parent {
            Scope::Hidden => Scope::Hidden,
            Scope::Shown if self.hidden.contains(key) => Scope::Hidden,
            Scope::Shown => Scope::Shown,
            Scope::AboveSolo => match &self.solo {
                Some(RowTarget::Layer(solo)) if solo == key => Scope::Shown,
                Some(solo) if key.is_on_path_to(solo.key()) => Scope::AboveSolo,
                _ => Scope::Hidden,
            },
        }
    }

    fn own_shapes_drawn(&self, key: &LayerKey, scope: Scope, sublayers: usize) -> bool {
        match scope {
            Scope::Shown => {
                !(has_own_shapes_row(key, sublayers) && self.hidden_own_shapes.contains(key))
            }
            Scope::Hidden => false,
            Scope::AboveSolo => {
                matches!(&self.solo, Some(RowTarget::OwnShapes(solo)) if solo == key)
            }
        }
    }
}

/// Which groups the user opened or closed, the others follow the default.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerExpansion {
    pub collapsed: BTreeSet<LayerKey>,
    pub expanded: BTreeSet<LayerKey>,
}

impl LayerExpansion {
    fn is_expanded(&self, key: &LayerKey, sublayers: usize) -> bool {
        !self.collapsed.contains(key)
            && (self.expanded.contains(key) || sublayers <= DEFAULT_EXPANDED_MAX_SUBLAYERS)
    }

    pub fn set(&mut self, key: LayerKey, expanded: bool) {
        let (on, off) = match expanded {
            true => (&mut self.expanded, &mut self.collapsed),
            false => (&mut self.collapsed, &mut self.expanded),
        };
        off.remove(&key);
        on.insert(key);
    }
}

/// State of the layers panel that is saved per project.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayerViewState {
    pub visibility: LayerVisibility,
    pub expansion: LayerExpansion,
}

/// `layer` without everything `visibility` hides. Hidden layers stay in place, empty.
pub fn filter_visible(layer: &Layer, visibility: &LayerVisibility) -> Layer {
    filter_layer(
        layer,
        &LayerKey::default(),
        visibility.outer_scope(),
        visibility,
    )
}

fn filter_layer(
    layer: &Layer,
    key: &LayerKey,
    parent_scope: Scope,
    visibility: &LayerVisibility,
) -> Layer {
    let scope = visibility.layer_scope(key, parent_scope);
    let shapes = if visibility.own_shapes_drawn(key, scope, layer.sublayers.len()) {
        layer.shapes.clone()
    } else {
        Vec::new()
    };
    let sublayers = match scope {
        Scope::Hidden => Vec::new(),
        Scope::Shown | Scope::AboveSolo => layer
            .sublayers
            .iter()
            .zip(sublayer_keys(key, layer))
            .map(|(sublayer, key)| filter_layer(sublayer, &key, scope, visibility))
            .collect(),
    };

    Layer::new_from_shapes_and_layers(shapes, sublayers)
        .with_props_inheritable(layer.props_inheritable.clone())
        .with_props(layer.props.clone())
}

/// Color and width a layer's shapes are drawn with.
#[derive(Debug, Clone, PartialEq)]
pub struct Pen {
    pub color: ColorRgb,
    pub width_cm: f32,
}

/// What the layers panel needs to know of a layer tree, computed once per run.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSummary {
    key: LayerKey,
    name: Option<String>,
    pen: Pen,
    own_shapes: usize,
    own_draw_length: f32,
    sublayers: Vec<LayerSummary>,
}

impl LayerSummary {
    pub fn new(layer: &Layer) -> Self {
        Self::of(
            layer,
            LayerKey::default(),
            &LayerPropsInheritable::default(),
        )
    }

    fn of(layer: &Layer, key: LayerKey, parent_props: &LayerPropsInheritable) -> Self {
        let props = parent_props.overwrite_with(&layer.props_inheritable);
        let sublayers = layer
            .sublayers
            .iter()
            .zip(sublayer_keys(&key, layer))
            .map(|(sublayer, key)| Self::of(sublayer, key, &props))
            .collect();

        Self {
            key,
            name: layer.props.name.clone(),
            pen: Pen {
                color: props.color.unwrap(),
                width_cm: props.pen_width_cm.unwrap(),
            },
            own_shapes: layer.shapes.len(),
            own_draw_length: layer.shapes.iter().map(Shape::length).sum(),
            sublayers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RowKind {
    /// A layer with sublayers.
    Group { expanded: bool },
    /// A layer with shapes only.
    Leaf,
    /// The own shapes of a group or of the root.
    OwnShapes,
    /// A layer with nothing in it.
    Empty,
    /// Stands for the sublayers of a group that got no row.
    More,
}

/// One line of the layers panel. Counts and pens cover everything the row hides.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerRow {
    pub target: RowTarget,
    pub depth: usize,
    pub label: String,
    pub named: bool,
    pub kind: RowKind,
    pub pens: Vec<Pen>,
    pub shapes: usize,
    pub shapes_visible: usize,
    /// Pen-down length in cm.
    pub draw_length: f32,
    /// The row's own flag. Its shapes can be invisible without it, through a parent or a solo.
    pub hidden: bool,
    pub soloed: bool,
}

impl LayerRow {
    /// Unique among the rows of a tree.
    pub fn id(&self) -> String {
        format!("{:?} {:?}", self.kind, self.target)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerRows {
    pub rows: Vec<LayerRow>,
    pub shapes: usize,
    pub shapes_visible: usize,
    /// The soloed row and its label.
    pub solo: Option<(RowTarget, String)>,
}

impl LayerRows {
    pub fn new(
        summary: &LayerSummary,
        visibility: &LayerVisibility,
        expansion: &LayerExpansion,
    ) -> Self {
        let mut walk = RowWalk {
            visibility,
            expansion,
            rows: Vec::new(),
            solo_label: None,
        };
        let scope = visibility.layer_scope(&summary.key, visibility.outer_scope());
        let totals = walk.content(summary, None, scope, 0, true);

        Self {
            rows: walk.rows,
            shapes: totals.shapes,
            shapes_visible: totals.shapes_visible,
            solo: visibility.solo.clone().map(|target| {
                let label = walk
                    .solo_label
                    .unwrap_or_else(|| "missing layer".to_string());
                (target, label)
            }),
        }
    }
}

#[derive(Default)]
struct Totals {
    shapes: usize,
    shapes_visible: usize,
    draw_length: f32,
    pens: Vec<Pen>,
}

impl Totals {
    fn add(mut self, other: Totals) -> Self {
        self.shapes += other.shapes;
        self.shapes_visible += other.shapes_visible;
        self.draw_length += other.draw_length;
        for pen in other.pens {
            if !self.pens.contains(&pen) {
                self.pens.push(pen);
            }
        }
        self
    }
}

struct RowWalk<'a> {
    visibility: &'a LayerVisibility,
    expansion: &'a LayerExpansion,
    rows: Vec<LayerRow>,
    solo_label: Option<String>,
}

impl RowWalk<'_> {
    fn is_soloed(&self, target: &RowTarget) -> bool {
        self.visibility.solo.as_ref() == Some(target)
    }

    /// Sums up a layer and adds its row, and those below it, when `emit` is set.
    fn layer(
        &mut self,
        node: &LayerSummary,
        label: String,
        parent_scope: Scope,
        depth: usize,
        emit: bool,
    ) -> Totals {
        let scope = self.visibility.layer_scope(&node.key, parent_scope);
        let target = RowTarget::Layer(node.key.clone());
        let soloed = self.is_soloed(&target);
        if soloed {
            self.solo_label = Some(label.clone());
        }
        let kind = match (node.sublayers.is_empty(), node.own_shapes) {
            (false, _) => RowKind::Group {
                expanded: self.expansion.is_expanded(&node.key, node.sublayers.len()),
            },
            (true, 0) => RowKind::Empty,
            (true, _) => RowKind::Leaf,
        };

        // The row comes before the rows of its content, its numbers come from them.
        let row_index = self.rows.len();
        if emit {
            self.rows.push(LayerRow {
                hidden: self.visibility.hidden.contains(&node.key),
                target,
                depth,
                named: node.name.is_some(),
                kind,
                pens: Vec::new(),
                shapes: 0,
                shapes_visible: 0,
                draw_length: 0.0,
                soloed,
                label: label.clone(),
            });
        }
        let emit_content = emit && kind == RowKind::Group { expanded: true };
        let totals = self.content(node, Some(&label), scope, depth + 1, emit_content);
        if emit {
            let row = &mut self.rows[row_index];
            row.pens = totals.pens.clone();
            row.shapes = totals.shapes;
            row.shapes_visible = totals.shapes_visible;
            row.draw_length = totals.draw_length;
        }
        totals
    }

    /// Sums up what is in a layer, and adds the rows for it when `emit` is set.
    fn content(
        &mut self,
        node: &LayerSummary,
        label: Option<&str>,
        scope: Scope,
        depth: usize,
        emit: bool,
    ) -> Totals {
        let sublayers = node.sublayers.len();
        let own_drawn = self
            .visibility
            .own_shapes_drawn(&node.key, scope, sublayers);
        let own = Totals {
            shapes: node.own_shapes,
            shapes_visible: if own_drawn { node.own_shapes } else { 0 },
            draw_length: node.own_draw_length,
            pens: (node.own_shapes > 0)
                .then(|| node.pen.clone())
                .into_iter()
                .collect(),
        };

        if node.own_shapes > 0 && has_own_shapes_row(&node.key, sublayers) {
            let target = RowTarget::OwnShapes(node.key.clone());
            let soloed = self.is_soloed(&target);
            if soloed {
                self.solo_label = Some(match label {
                    Some(label) => format!("{label} shapes"),
                    None => "shapes".to_string(),
                });
            }
            if emit {
                self.rows.push(LayerRow {
                    hidden: self.visibility.hidden_own_shapes.contains(&node.key),
                    target,
                    depth,
                    label: "shapes".to_string(),
                    named: false,
                    kind: RowKind::OwnShapes,
                    pens: own.pens.clone(),
                    shapes: own.shapes,
                    shapes_visible: own.shapes_visible,
                    draw_length: own.draw_length,
                    soloed,
                });
            }
        }

        let totals = node
            .sublayers
            .iter()
            .enumerate()
            .fold(own, |totals, (i, sublayer)| {
                let label = sublayer
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("layer {}", i + 1));
                let emit = emit && i < MAX_ROWS_PER_GROUP;
                totals.add(self.layer(sublayer, label, scope, depth, emit))
            });

        if emit && sublayers > MAX_ROWS_PER_GROUP {
            self.rows.push(LayerRow {
                target: RowTarget::Layer(node.key.clone()),
                depth,
                label: format!("+ {} more layers", sublayers - MAX_ROWS_PER_GROUP),
                named: false,
                kind: RowKind::More,
                pens: Vec::new(),
                shapes: 0,
                shapes_visible: 0,
                draw_length: 0.0,
                hidden: false,
                soloed: false,
            });
        }
        totals
    }
}
