use std::collections::HashMap;

use crate::{Angle, Path, LARGE_EPSILON, V2};

use super::path_end::PathEnd;

type Cell = (i64, i64);

/// Path ends bucketed by position. Entries are `path index * 2 + end bit`, with bit 0 for the start.
struct EndIndex {
    buckets: HashMap<Cell, Vec<usize>>,
}

impl EndIndex {
    fn new(ends: &[[PathEnd; 2]]) -> Self {
        let mut buckets: HashMap<Cell, Vec<usize>> = HashMap::with_capacity(ends.len() * 2);
        for (i, path_ends) in ends.iter().enumerate() {
            for (bit, end) in path_ends.iter().enumerate() {
                buckets
                    .entry(cell(end.point))
                    .or_default()
                    .push(i * 2 + bit);
            }
        }
        Self { buckets }
    }

    fn remove(&mut self, ends: &[[PathEnd; 2]], path_index: usize) {
        for (bit, end) in ends[path_index].iter().enumerate() {
            if let Some(bucket) = self.buckets.get_mut(&cell(end.point)) {
                bucket.retain(|&entry| entry != path_index * 2 + bit);
            }
        }
    }

    /// Lowest entry near `point` that passes `matches`.
    fn find(&self, point: V2, matches: impl Fn(usize) -> bool) -> Option<usize> {
        let (x, y) = cell(point);
        (-1..=1)
            .flat_map(|dx| (-1..=1).map(move |dy| (x + dx, y + dy)))
            .filter_map(|neighbour| self.buckets.get(&neighbour))
            .flatten()
            .copied()
            .filter(|&entry| matches(entry))
            .min()
    }
}

/// Points closer than [`LARGE_EPSILON`] per axis always fall into neighbouring cells.
fn cell(point: V2) -> Cell {
    let epsilon = f64::from(LARGE_EPSILON);
    (
        (f64::from(point.x) / epsilon).floor() as i64,
        (f64::from(point.y) / epsilon).floor() as i64,
    )
}

/// Joins paths whose ends meet into chains, in a single pass. Paths with fewer than 2 points are dropped.
pub(crate) fn combine_paths(paths: Vec<Path>, max_angle_delta: Option<Angle>) -> Vec<Path> {
    let paths: Vec<Path> = paths.into_iter().filter(|path| path.len() > 1).collect();
    let ends: Vec<[PathEnd; 2]> = paths
        .iter()
        .map(|path| [PathEnd::from_path_start(path), PathEnd::from_path_end(path)])
        .collect();
    let mut joiner = Joiner {
        index: EndIndex::new(&ends),
        used: vec![false; paths.len()],
        paths: &paths,
        ends: &ends,
        max_angle_delta,
    };

    (0..paths.len())
        .filter_map(|i| {
            if joiner.used[i] {
                return None;
            }
            joiner.take(i);
            let mut forward = paths[i].points().to_vec();
            joiner.extend(ends[i][1], &mut forward);
            // Walking backward is walking forward on the reversed chain, so these points come out reversed.
            let mut backward = Vec::new();
            joiner.extend(ends[i][0].flipped(), &mut backward);

            Some(backward.into_iter().rev().chain(forward).collect())
        })
        .collect()
}

struct Joiner<'a> {
    paths: &'a [Path],
    ends: &'a [[PathEnd; 2]],
    index: EndIndex,
    used: Vec<bool>,
    max_angle_delta: Option<Angle>,
}

impl Joiner<'_> {
    fn take(&mut self, path_index: usize) {
        self.used[path_index] = true;
        self.index.remove(self.ends, path_index);
    }

    /// Appends matching paths to `points`, starting at `tail`, which points away from the chain.
    fn extend(&mut self, mut tail: PathEnd, points: &mut Vec<V2>) {
        while let Some(entry) = self.index.find(tail.point, |entry| {
            let candidate = &self.ends[entry / 2];
            match entry % 2 {
                0 => tail.is_compatible(&candidate[0], &self.max_angle_delta),
                _ => tail.is_compatible(&candidate[1].flipped(), &self.max_angle_delta),
            }
        }) {
            let (j, candidate) = (entry / 2, self.paths[entry / 2].points());
            if entry % 2 == 0 {
                points.extend(candidate.iter().skip(1));
                tail = self.ends[j][1];
            } else {
                points.extend(candidate.iter().rev().skip(1));
                tail = self.ends[j][0].flipped();
            }
            self.take(j);
        }
    }
}
