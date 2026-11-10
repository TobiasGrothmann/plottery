use std::collections::HashMap;

use crate::{Path, V2};

/// Where an iso-line crosses the grid edge with ID `edge`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Crossing {
    pub edge: usize,
    pub pos: V2,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Segment(pub [Crossing; 2]);

const NONE: usize = usize::MAX;

/// Joins segments into paths through their shared grid edges. Open contours come first, then closed ones.
pub(super) fn stitch(segments: Vec<Segment>) -> Vec<Path> {
    // A grid edge touches at most two cells with at most one segment each, so the graph has only chains and cycles.
    let mut touching: HashMap<usize, [usize; 2]> = HashMap::with_capacity(segments.len());
    for (index, segment) in segments.iter().enumerate() {
        for crossing in &segment.0 {
            let slots = touching.entry(crossing.edge).or_insert([NONE; 2]);
            let free = slots.iter().position(|&slot| slot == NONE);
            debug_assert!(
                free.is_some(),
                "edge {} has more than two segments",
                crossing.edge
            );
            if let Some(free) = free {
                slots[free] = index;
            }
        }
    }

    let mut stitcher = Stitcher {
        segments: &segments,
        touching,
        used: vec![false; segments.len()],
    };

    // Starting at a bounds edge, the only kind with one segment, yields each open contour whole.
    let open: Vec<Path> = (0..segments.len())
        .filter_map(|index| {
            let start = (0..2).find(|&end| stitcher.is_bounds(segments[index].0[end].edge))?;
            stitcher.walk(index, start)
        })
        .filter(|path| path.len() >= 2)
        .collect();
    let closed: Vec<Path> = (0..segments.len())
        .filter_map(|index| stitcher.walk(index, 0))
        .filter(|path| path.len() >= 4)
        .collect();

    open.into_iter().chain(closed).collect()
}

struct Stitcher<'a> {
    segments: &'a [Segment],
    touching: HashMap<usize, [usize; 2]>,
    used: Vec<bool>,
}

impl Stitcher<'_> {
    fn is_bounds(&self, edge: usize) -> bool {
        self.touching[&edge][1] == NONE
    }

    fn other(&self, edge: usize, segment: usize) -> Option<usize> {
        self.touching[&edge]
            .into_iter()
            .find(|&slot| slot != NONE && slot != segment)
    }

    /// Follows the chain from crossing `entry` of an unused `segment` to a bounds edge or back to the start.
    fn walk(&mut self, mut segment: usize, mut entry: usize) -> Option<Path> {
        if self.used[segment] {
            return None;
        }
        let mut points = vec![self.segments[segment].0[entry].pos];
        loop {
            self.used[segment] = true;
            let exit = self.segments[segment].0[1 - entry];
            // Degenerate segments, where the contour touches a sample, add the same point twice.
            if points.last() != Some(&exit.pos) {
                points.push(exit.pos);
            }
            match self.other(exit.edge, segment) {
                Some(next) if !self.used[next] => {
                    entry = usize::from(self.segments[next].0[0].edge != exit.edge);
                    segment = next;
                }
                _ => return Some(Path::from(points)),
            }
        }
    }
}
