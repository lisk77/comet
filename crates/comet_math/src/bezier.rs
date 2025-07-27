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

    /// Returns the point at the position `t` along the curve.
    pub fn evaluate(&self, t: f32) -> V {
        let mut new_points = self.points.clone();
        for i in 0..self.degree {
            for j in 0..(self.degree - i) {
                new_points[j] = new_points[j].lerp(&new_points[j + 1], t);
            }
        }
        new_points[0].clone()
    }

    /// Returns the velocity at the position `t` along the curve.
    pub fn velocity(&self, t: f32) -> V {
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

        d_pts[0].clone()
    }

    /// Returns the tangent at the position `t` along the curve.
    pub fn tangent(&self, t: f32) -> V {
        self.velocity(t).normalize()
    }

    /// Returns the acceleration at the position `t` along the curve.
    pub fn acceleration(&self, t: f32) -> V {
        let n = self.degree as f32;

        let mut dd_pts: Vec<V> = self
            .points
            .windows(3)
            .map(|w| (w[2] - w[1] * 2.0 + w[0]) * (n * (n - 1.0)))
            .collect();

        for i in 0..(self.degree - 2) {
            for j in 0..(self.degree - 2 - i) {
                dd_pts[j] = dd_pts[j].lerp(&dd_pts[j + 1], t);
            }
        }

        dd_pts[0].clone()
    }

    /// Returns the curvature at the position `t` along the curve.
    pub fn curvature(&self, t: f32) -> f32 {
        let v = self.velocity(t);
        let a = self.acceleration(t);

        let s2 = v.dot(&v);
        if s2 == 0.0 {
            return 0.0;
        }

        let a_para = v.clone() * (a.dot(&v) / s2);
        let a_perp = a - a_para;

        a_perp.length() / s2.sqrt().powi(2)
    }

    /// Returns the arclength of the curve.
    pub fn arclength(&self) -> f32 {
        let mut length = 0.0;
        let dt = 1.0 / 1000.0;
        let mut prev = self.evaluate(0.0);
        for i in 1..=1000 {
            let t = i as f32 * dt;
            let curr = self.evaluate(t);
            length += (curr - prev).length();
            prev = curr;
        }
        length
    }
}
