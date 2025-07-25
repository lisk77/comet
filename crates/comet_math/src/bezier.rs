use crate::{InnerSpace, Point};

pub struct Bezier<V: InnerSpace> {
    points: Vec<V>,
    degree: usize,
}

impl<V: InnerSpace + Clone> Bezier<V> {
    pub fn new(points: Vec<V>) -> Self {
        let degree = points.len() - 1;

        Self { points, degree }
    }

    pub fn evaluate(&self, t: f32) -> V {
        let mut new_points = self.points.clone();
        for i in 0..self.degree {
            for j in 0..(self.degree - i) {
                new_points[j] = new_points[j].lerp(&new_points[j + 1], t);
            }
        }
        new_points[0].clone()
    }

    pub fn evaluate_tangent(&self, t: f32) -> V {
        let n = self.degree as f32;

        let mut d_pts: Vec<V> = self
            .points
            .windows(2)
            .map(|w| ((w[1] - w[0]) * n))
            .collect::<Vec<V>>();

        for i in 0..(self.degree - 1) {
            for j in 0..(self.degree - 1 - i) {
                d_pts[j] = d_pts[j].lerp(&d_pts[j + 1], t);
            }
        }

        d_pts[0].clone().normalize()
    }
}
