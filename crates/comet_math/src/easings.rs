use std::f32::consts::PI;

pub fn ease_in_sine(x: f32) -> f32 {
	1.0 - ((x * PI) / 2.0).cos()
}

pub fn ease_out_sine(x: f32) -> f32 {
	((x * PI) / 2.0).sin()
}

pub fn ease_in_out_sine(x: f32) -> f32 {
	-((PI * x).cos() - 1.0) / 2.0
}

pub fn ease_in_quad(x: f32) -> f32 {
	x * x
}

pub fn ease_out_quad(x: f32) -> f32 {
	1.0 - (1.0 - x) * (1.0 - x)
}

pub fn ease_in_out_quad(x: f32) -> f32 {
	if x < 0.5 { 2.0 * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0 }
}

pub fn ease_in_cubic(x: f32) -> f32 {
	x * x * x
}

pub fn ease_out_cubic(x: f32) -> f32 {
	1.0 - (1.0 - x).powi(3)
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
	if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(3) / 2.0 }
}

pub fn ease_in_quart(x: f32) -> f32 {
	x * x * x * x
}

pub fn ease_out_quart(x: f32) -> f32 {
	1.0 - (1.0 - x).powi(4)
}

pub fn ease_in_out_quart(x: f32) -> f32 {
	if x < 0.5 { 8.0 * x * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(4) / 2.0 }
}

pub fn ease_in_quint(x: f32) -> f32 {
	x * x * x * x * x
}

pub fn ease_out_quint(x: f32) -> f32 {
	1.0 - (1.0 - x).powi(5)
}

pub fn ease_in_out_quint(x: f32) -> f32 {
	if x < 0.5 { 16.0 * x * x * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(5) / 2.0 }
}

pub fn ease_in_expo(x: f32) -> f32 {
	if x == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * x - 10.0) }
}

pub fn ease_out_expo(x: f32) -> f32 {
	if x == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * x) }
}

pub fn ease_in_out_expo(x: f32) -> f32 {
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else if x < 0.5 { 2.0_f32.powf(20.0 * x - 10.0) / 2.0 } else { (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) / 2.0 }
}

pub fn ease_in_circ(x: f32) -> f32 {
	1.0 - (1.0 - x * x).sqrt()
}

pub fn ease_out_circ(x: f32) -> f32 {
	(1.0 - (x - 1.0).powi(2)).sqrt()
}

pub fn ease_in_out_circ(x: f32) -> f32 {
	if x < 0.5 { (1.0 - (1.0 - 2.0 * x).powi(2)).sqrt() / 2.0 } else { ((1.0 - (-2.0 * x + 2.0).powi(2)).sqrt() + 1.0) / 2.0 }
}

pub fn ease_in_back(x: f32) -> f32 {
	2.70158 * x * x * x - 1.70158 * x * x
}

pub fn ease_out_back(x: f32) -> f32 {
	1.0 + 2.70158 * (x - 1.0).powi(3) + 1.70158 * (x - 1.0).powi(2)
}

pub fn ease_in_out_back(x: f32) -> f32 {
	let c1 = 1.70158;
	let c2 = c1 * 1.525;
	if x < 0.5 { (2.0 * x).powi(2) * ((c2 + 1.0) * 2.0 * x - c2) / 2.0 } else { ((2.0 * x - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * x - 2.0) + c2) + 2.0) / 2.0 }
}

pub fn ease_in_elastic(x: f32) -> f32 {
	let c4 = (2.0 * PI) / 3.0;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else { -2.0_f32.powf(10.0 * x - 10.0) * ((x * 10.0 - 10.75) * c4).sin() }
}

pub fn ease_out_elastic(x: f32) -> f32 {
	let c4 = (2.0 * PI) / 3.0;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else { 2.0_f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c4).sin() + 1.0 }
}

pub fn ease_in_out_elastic(x: f32) -> f32 {
	let c5 = (2.0 * PI) / 4.5;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else if x < 0.5 { -(2.0_f32.powf(20.0 * x - 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0 } else { (2.0_f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c5).sin()) / 2.0 + 1.0 }
}

pub fn ease_in_bounce(x: f32) -> f32 {
	1.0 - ease_out_bounce(1.0 - x)
}

pub fn ease_out_bounce(x: f32) -> f32 {
	let n1 = 7.5625;
	let d1 = 2.75;
	if x < 1.0 / d1 { n1 * x * x } else if x < 2.0 / d1 { n1 * (x - 1.5 / d1) * (x - 1.5 / d1) + 0.75 } else if x < 2.5 / d1 { n1 * (x - 2.25 / d1) * (x - 2.25 / d1) + 0.9375 } else { n1 * (x - 2.625 / d1) * (x - 2.625 / d1) + 0.984375 }
}

pub fn ease_in_out_bounce(x: f32) -> f32 {
	if x < 0.5 { (1.0 - ease_out_bounce(1.0 - 2.0 * x)) / 2.0 } else { (1.0 + ease_out_bounce(2.0 * x - 1.0)) / 2.0 }
}
