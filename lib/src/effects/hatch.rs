// Hatch fill of closed rings as zigzags that follow the border between lines, like a 3D printer's infill.
//
// Rings are outer contours and holes in any orientation, filled with the even-odd rule. In a frame
// rotated by `-angle`, horizontal scanlines cut the rings into segments. Every crossing of a ring with
// a scanline has one border piece above and one below it, leading to the next crossing along the ring.
// Segments whose two upper pieces land on the same segment of the next line form a cell, drawn as one
// zigzag. Cells end where the shape splits, merges, around holes and at tips. A finished cell continues
// into an undrawn cell one border piece away, otherwise the pen lifts to the nearest undrawn cell.

use crate::{Angle, Layer, Path, Plottable, Rotate, SampleSettings, V2};

const DEFAULT_OFFSET: f32 = 0.5;
const MIN_LINE_DIST: f32 = 0.001;

/// Fills closed shapes with parallel lines, zigzagging along the border to save pen lifts.
///
/// Apply it with [`Plottable::hatch`] or [`Layer::hatch`].
#[derive(Clone, Debug)]
pub struct Hatch {
    line_dist: f32,
    angle: Angle,
    offset: f32,
    connected: bool,
    sample_settings: SampleSettings,
}

impl Hatch {
    /// `line_dist` in cm, horizontal lines connected along the border.
    pub fn new(line_dist: f32) -> Self {
        Self {
            line_dist: line_dist.max(MIN_LINE_DIST),
            angle: Angle::zero(),
            offset: DEFAULT_OFFSET,
            connected: true,
            sample_settings: SampleSettings::default(),
        }
    }

    #[must_use]
    pub fn with_angle(mut self, angle: Angle) -> Self {
        self.angle = angle;
        self
    }

    /// Shift of the lines as a fraction of `line_dist`, default 0.5. Lines sit on a global grid, so separate shapes line up.
    #[must_use]
    pub fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// `false` draws every line as its own 2-point path.
    #[must_use]
    pub fn with_connected(mut self, connected: bool) -> Self {
        self.connected = connected;
        self
    }

    /// How curved shapes are sampled into rings.
    #[must_use]
    pub fn with_sample_settings(mut self, sample_settings: SampleSettings) -> Self {
        self.sample_settings = sample_settings;
        self
    }

    /// Hatches rings as one even-odd region, pen-down paths in drawing order.
    pub fn hatch_rings(&self, rings: &[Vec<V2>]) -> Layer {
        let to_frame = Angle::from_rad(-self.angle.to_rad());
        let rings: Vec<Vec<V2>> = rings
            .iter()
            .map(|ring| {
                clean_ring(ring)
                    .into_iter()
                    .map(|p| p.rotate(to_frame))
                    .collect()
            })
            .filter(|ring: &Vec<V2>| ring.len() >= 3)
            .collect();
        let topology = Topology::new(&rings, self.line_dist, self.offset);

        let paths: Vec<Vec<V2>> = if self.connected {
            topology.draw_order()
        } else {
            topology
                .segments
                .iter()
                .map(|s| {
                    vec![
                        topology.crossings[s.left].pos,
                        topology.crossings[s.right].pos,
                    ]
                })
                .collect()
        };
        paths
            .into_iter()
            .map(|points| {
                let mut points: Vec<V2> = points.iter().map(|p| p.rotate(self.angle)).collect();
                points.dedup();
                Path::new_from(points)
            })
            .filter(|path| path.get_points_ref().len() >= 2)
            .collect()
    }

    pub(crate) fn hatch_shape<P: Plottable>(&self, shape: &P) -> Layer {
        self.hatch_rings(&[shape.get_points(self.sample_settings)])
    }

    pub(crate) fn hatch_layer(&self, layer: &Layer) -> Layer {
        let rings: Vec<Vec<V2>> = layer
            .iter()
            .filter(|shape| shape.is_closed())
            .map(|shape| shape.get_points(self.sample_settings))
            .collect();
        self.hatch_rings(&rings)
    }
}

/// Ring without repeated points and without the closing point.
fn clean_ring(ring: &[V2]) -> Vec<V2> {
    let mut points = ring.to_vec();
    points.dedup();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
}

struct Crossing {
    pos: V2,
    line: i32,
    ring: usize,
    /// The ring edge starts at this vertex.
    edge: usize,
    t: f32,
    /// The ring passes the line upwards here.
    up: bool,
    next: usize,
    prev: usize,
    segment: usize,
}

struct Segment {
    left: usize,
    right: usize,
    cell: usize,
}

/// Border piece from a crossing to a neighbouring crossing along the ring.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Link {
    to: usize,
    forward: bool,
}

struct Topology<'a> {
    rings: &'a [Vec<V2>],
    crossings: Vec<Crossing>,
    segments: Vec<Segment>,
    /// Segment indices of each cell, bottom to top.
    cells: Vec<Vec<usize>>,
}

impl<'a> Topology<'a> {
    fn new(rings: &'a [Vec<V2>], line_dist: f32, offset: f32) -> Self {
        // A vertex belongs to the band above line `band(p)`. An edge crosses every line between the
        // bands of its ends. A vertex exactly on a line is above it, so nothing is counted twice.
        let band = move |p: V2| (p.y / line_dist - offset).floor() as i32;
        let mut crossings: Vec<Crossing> = rings
            .iter()
            .enumerate()
            .flat_map(|(ring, points)| {
                (0..points.len()).flat_map(move |edge| {
                    let (a, b) = (points[edge], points[(edge + 1) % points.len()]);
                    let (band_a, band_b) = (band(a), band(b));
                    let up = band_b > band_a;
                    let lines = if up {
                        band_a + 1..=band_b
                    } else {
                        band_b + 1..=band_a
                    };
                    lines.map(move |line| {
                        let y = (line as f32 + offset) * line_dist;
                        let t = ((y - a.y) / (b.y - a.y)).clamp(0.0, 1.0);
                        Crossing {
                            pos: V2::new(a.x + (b.x - a.x) * t, y),
                            line,
                            ring,
                            edge,
                            t,
                            up,
                            next: 0,
                            prev: 0,
                            segment: 0,
                        }
                    })
                })
            })
            .collect();

        // ring order
        crossings.sort_by(|a, b| {
            (a.ring, a.edge)
                .cmp(&(b.ring, b.edge))
                .then(a.t.total_cmp(&b.t))
        });
        let mut start = 0;
        for chunk in crossings.chunk_by_mut(|a, b| a.ring == b.ring) {
            let len = chunk.len();
            for (i, c) in chunk.iter_mut().enumerate() {
                c.next = start + (i + 1) % len;
                c.prev = start + (i + len - 1) % len;
            }
            start += len;
        }

        // every line has an even number of crossings, so pairs never span two lines
        let mut by_line: Vec<usize> = (0..crossings.len()).collect();
        by_line.sort_by(|&a, &b| {
            let (a, b) = (&crossings[a], &crossings[b]);
            a.line.cmp(&b.line).then(a.pos.x.total_cmp(&b.pos.x))
        });
        let mut segments: Vec<Segment> = by_line
            .chunks_exact(2)
            .map(|pair| Segment {
                left: pair[0],
                right: pair[1],
                cell: 0,
            })
            .collect();
        for (i, s) in segments.iter().enumerate() {
            crossings[s.left].segment = i;
            crossings[s.right].segment = i;
        }

        let mut topology = Self {
            rings,
            crossings,
            segments: Vec::new(),
            cells: Vec::new(),
        };
        let next: Vec<Option<usize>> = segments.iter().map(|s| topology.continuation(s)).collect();
        let mut continued = vec![false; segments.len()];
        next.iter().flatten().for_each(|&s| continued[s] = true);
        let cells: Vec<Vec<usize>> = (0..segments.len())
            .filter(|&s| !continued[s])
            .map(|s| std::iter::successors(Some(s), |&s| next[s]).collect())
            .collect();
        for (i, cell) in cells.iter().enumerate() {
            cell.iter().for_each(|&s| segments[s].cell = i);
        }
        topology.segments = segments;
        topology.cells = cells;
        topology
    }

    fn above(&self, c: usize) -> Link {
        let c = &self.crossings[c];
        if c.up {
            Link {
                to: c.next,
                forward: true,
            }
        } else {
            Link {
                to: c.prev,
                forward: false,
            }
        }
    }

    fn below(&self, c: usize) -> Link {
        let c = &self.crossings[c];
        if c.up {
            Link {
                to: c.prev,
                forward: false,
            }
        } else {
            Link {
                to: c.next,
                forward: true,
            }
        }
    }

    /// The segment on the next line whose ends both border pieces above `s` lead to.
    fn continuation(&self, s: &Segment) -> Option<usize> {
        let line = self.crossings[s.left].line;
        let left = &self.crossings[self.above(s.left).to];
        let right = &self.crossings[self.above(s.right).to];
        (left.line == line + 1 && right.line == line + 1 && left.segment == right.segment)
            .then_some(left.segment)
    }

    /// Border points after crossing `from`, up to and including the crossing `link` leads to.
    fn piece(&self, from: usize, link: Link) -> Vec<V2> {
        let (a, b) = if link.forward {
            (from, link.to)
        } else {
            (link.to, from)
        };
        let (a, b) = (&self.crossings[a], &self.crossings[b]);
        let ring = &self.rings[a.ring];
        let n = ring.len();
        let count = (b.edge + n - a.edge) % n;
        let mut points: Vec<V2> = (1..=count).map(|i| ring[(a.edge + i) % n]).collect();
        if link.forward {
            points.push(b.pos);
        } else {
            points.reverse();
            points.push(a.pos);
        }
        points
    }

    /// Zigzag through a cell from crossing `start` on its bottom (`up`) or top segment.
    /// Returns the points and the crossing it ends on.
    fn walk_cell(&self, cell: usize, up: bool, start: usize) -> (Vec<V2>, usize) {
        let segments = &self.cells[cell];
        let n = segments.len();
        let mut points = vec![self.crossings[start].pos];
        let mut at = start;
        for i in 0..n {
            let s = &self.segments[segments[if up { i } else { n - 1 - i }]];
            at = if at == s.left { s.right } else { s.left };
            points.push(self.crossings[at].pos);
            if i + 1 < n {
                let link = if up { self.above(at) } else { self.below(at) };
                points.extend(self.piece(at, link));
                at = link.to;
            }
        }
        (points, at)
    }

    /// An undrawn cell that starts one border piece away from crossing `at`.
    fn chain_target(&self, at: usize, up: bool, drawn: &[bool]) -> Option<(Link, usize, bool)> {
        let links = if up {
            [self.above(at), self.below(at)]
        } else {
            [self.below(at), self.above(at)]
        };
        links.into_iter().find_map(|link| {
            let target = link.to;
            let segment = self.crossings[target].segment;
            let cell = self.segments[segment].cell;
            if drawn[cell] {
                return None;
            }
            let arriving = Link {
                to: at,
                forward: !link.forward,
            };
            let from_below = self.below(target) == arriving;
            let segments = &self.cells[cell];
            let at_end = if from_below {
                segments[0]
            } else {
                segments[segments.len() - 1]
            };
            (segment == at_end).then_some((link, cell, from_below))
        })
    }

    /// The undrawn cell corner closest to `pos`: (cell, up, start crossing).
    fn nearest_start(&self, pos: V2, drawn: &[bool]) -> Option<(usize, bool, usize)> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(cell, _)| !drawn[*cell])
            .flat_map(|(cell, segments)| {
                let (bottom, top) = (
                    &self.segments[segments[0]],
                    &self.segments[segments[segments.len() - 1]],
                );
                [
                    (cell, true, bottom.left),
                    (cell, true, bottom.right),
                    (cell, false, top.left),
                    (cell, false, top.right),
                ]
            })
            .min_by(|a, b| {
                let dist = |c: usize| self.crossings[c].pos.dist(pos);
                dist(a.2).total_cmp(&dist(b.2))
            })
    }

    /// Left end of the lowest segment, going up.
    fn lowest_start(&self) -> Option<(usize, bool, usize)> {
        self.cells
            .iter()
            .enumerate()
            .map(|(cell, segments)| (cell, true, self.segments[segments[0]].left))
            .min_by(|a, b| {
                let (pa, pb) = (self.crossings[a.2].pos, self.crossings[b.2].pos);
                pa.y.total_cmp(&pb.y).then(pa.x.total_cmp(&pb.x))
            })
    }

    /// Draws every cell once: chained along the border where possible, otherwise after a pen lift.
    fn draw_order(&self) -> Vec<Vec<V2>> {
        let mut drawn = vec![false; self.cells.len()];
        let mut paths: Vec<Vec<V2>> = Vec::new();
        let mut current: Vec<V2> = Vec::new();
        let mut pen: Option<(usize, bool)> = None;
        let mut pos = V2::new(0.0, 0.0);
        loop {
            let chained = pen
                .and_then(|(at, up)| self.chain_target(at, up, &drawn).map(|target| (at, target)));
            let (cell, up, from) = match chained {
                Some((at, (link, cell, up))) => {
                    current.extend(self.piece(at, link));
                    (cell, up, link.to)
                }
                None => {
                    if !current.is_empty() {
                        paths.push(std::mem::take(&mut current));
                    }
                    let start = if pen.is_none() {
                        self.lowest_start()
                    } else {
                        self.nearest_start(pos, &drawn)
                    };
                    match start {
                        Some(start) => start,
                        None => break,
                    }
                }
            };
            drawn[cell] = true;
            let (points, end) = self.walk_cell(cell, up, from);
            current.extend(points);
            pen = Some((end, up));
            pos = self.crossings[end].pos;
        }
        paths
    }
}
