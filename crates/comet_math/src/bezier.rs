use crate::Point;

pub struct Bezier<P: Point> {
	points: Vec<P>,
	degree: usize
}

impl<P: Point + Clone> Bezier<P> {
	pub fn new(points: Vec<P>) -> Self {
		let degree = points.len() - 1;

		Self {
			points,
			degree
		}
	}

	pub fn evaluate(&self, t: f32) -> P {
		let mut new_points = self.points.clone();
		for i in 0..self.degree {
			for j in 0..(self.degree - i) {
				new_points[j] = new_points[j].lerp(&new_points[j + 1], t);
			}
		}
		new_points[0].clone()
	}
}