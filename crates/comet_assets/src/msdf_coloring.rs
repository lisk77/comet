//! An "ink trap" edge-coloring pass for MTSDF generation, ported from
//! `msdfgen`'s `edgeColoringInkTrap` (Chlumsky/msdfgen, `edge-coloring.cpp`).
//!
//! `fdsm` only implements the "simple" coloring algorithm, which colors
//! every corner independently. When two corners of the same contour sit
//! close together (a thin notch, a tight bowl — exactly the shapes found
//! in dense letterforms like `e`, `o`, `m`, `c`), their independently
//! chosen colors can end up sharing a channel, producing a smeared,
//! rounded corner instead of a crisp one once the MTSDF is sampled.
//!
//! Ink trap coloring fixes this by classifying corners between short
//! edges as "minor" and blending their color from their neighboring
//! "major" corner instead of picking one independently, so nearby
//! corners agree rather than compete.

use fdsm::{
    bezier::Segment,
    color::Color,
    shape::{ColoredContour, ColoredSegment, Contour, Shape},
};

struct Corner {
    index: usize,
    prev_edge_length_estimate: f64,
    minor: bool,
    color: Color,
}

fn estimate_edge_length(segment: &Segment) -> f64 {
    const STEPS: usize = 4;
    let mut length = 0.0;
    let mut prev = segment.start();
    for i in 1..=STEPS {
        let t = i as f64 / STEPS as f64;
        let p = segment.get(t);
        length += (p - prev).norm();
        prev = p;
    }
    length
}

/// Colors a shape using the ink-trap heuristic.
///
/// `sin_alpha` is the sine of the maximum angle at which a corner is
/// considered sharp (same units as [`fdsm::shape::Shape::edge_coloring_simple`]).
pub fn edge_coloring_ink_trap(
    shape: Shape<Contour>,
    sin_alpha: f64,
    seed: u64,
) -> Shape<ColoredContour> {
    let (mut color, mut seed) = Color::WHITE.switch(seed, Color::BLACK);
    Shape {
        contours: shape
            .contours
            .into_iter()
            .map(|contour| color_contour_ink_trap(contour, sin_alpha, &mut color, &mut seed))
            .collect(),
    }
}

fn color_contour_ink_trap(
    contour: Contour,
    sin_alpha: f64,
    color: &mut Color,
    seed: &mut u64,
) -> ColoredContour {
    let m = contour.segments.len();
    if m == 0 {
        return ColoredContour::default();
    }

    let mut corners: Vec<Corner> = Vec::new();
    let mut spline_length = 0.0;
    let mut prev_index = m - 1;
    for index in 0..m {
        let is_corner = contour.segments[prev_index]
            .corners_into(&contour.segments[index], sin_alpha)
            .is_some();
        if is_corner {
            corners.push(Corner {
                index,
                prev_edge_length_estimate: spline_length,
                minor: false,
                color: Color::BLACK,
            });
            spline_length = 0.0;
        }
        spline_length += estimate_edge_length(&contour.segments[index]);
        prev_index = index;
    }

    // Smooth contours and "teardrop" (single-corner) contours don't benefit
    // from minor-corner blending; fall back to the well-tested simple
    // algorithm, which already handles both (including edge-splitting for
    // contours with fewer than 3 segments).
    if corners.len() < 2 {
        return ColoredContour::edge_coloring_simple(contour, sin_alpha, *seed);
    }

    let corner_count = corners.len();
    let mut major_corner_count = corner_count;
    if corner_count > 3 {
        corners[0].prev_edge_length_estimate += spline_length;
        for i in 0..corner_count {
            let a = corners[i].prev_edge_length_estimate;
            let b = corners[(i + 1) % corner_count].prev_edge_length_estimate;
            let c = corners[(i + 2) % corner_count].prev_edge_length_estimate;
            if a > b && b < c {
                corners[i].minor = true;
                major_corner_count -= 1;
            }
        }
    }

    let mut initial_color = Color::BLACK;
    for i in 0..corner_count {
        if !corners[i].minor {
            major_corner_count -= 1;
            let banned = if major_corner_count == 0 {
                initial_color
            } else {
                Color::BLACK
            };
            let (next_color, next_seed) = color.switch(*seed, banned);
            *color = next_color;
            *seed = next_seed;
            corners[i].color = *color;
            if initial_color == Color::BLACK {
                initial_color = *color;
            }
        }
    }
    for i in 0..corner_count {
        if corners[i].minor {
            let next_color = corners[(i + 1) % corner_count].color;
            corners[i].color =
                Color::new((color.value() & next_color.value()) ^ Color::WHITE.value());
        } else {
            *color = corners[i].color;
        }
    }

    let mut colors = vec![Color::BLACK; m];
    let mut spline = 0usize;
    let start = corners[0].index;
    let mut cur_color = corners[0].color;
    for i in 0..m {
        let index = (start + i) % m;
        if spline + 1 < corner_count && corners[spline + 1].index == index {
            spline += 1;
            cur_color = corners[spline].color;
        }
        colors[index] = cur_color;
    }

    let segments = contour
        .segments
        .into_iter()
        .zip(colors)
        .map(|(segment, color)| ColoredSegment { segment, color })
        .collect();
    ColoredContour { segments }
}
