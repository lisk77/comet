use crate::utilities::{PI, sin, cos, sqrt};

pub fn ease_in_sine(x: f32) -> f32 {
	1.0 - cos((x * PI) / 2.0)
}

pub fn ease_out_sine(x: f32) -> f32 {
	sin((x * PI) / 2.0)
}

pub fn ease_in_out_sine(x: f32) -> f32 {
	-(cos(PI * x) - 1.0) / 2.0
}

pub fn ease_in_quad(x: f32) -> f32 {
	x * x
}

pub fn ease_out_quad(x: f32) -> f32 {
	1.0 - (1.0 - x) * (1.0 - x)
}

pub fn ease_in_out_quad(x: f32) -> f32 {
	if x < 0.5 { 2.0 * x * x } else { 1.0 - (-2.0 * x + 2.0).powf(2.0) / 2.0 }
}

pub fn ease_in_cubic(x: f32) -> f32 {
	x * x * x
}

pub fn ease_out_cubic(x: f32) -> f32 {
	1.0 - (1.0 - x).powf(3.0)
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
	if x < 0.5 { 4.0 * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powf(3.0) / 2.0 }
}

pub fn ease_in_quart(x: f32) -> f32 {
	x * x * x * x
}

pub fn ease_out_quart(x: f32) -> f32 {
	1.0 - (1.0 - x).powf(4.0)
}

pub fn ease_in_out_quart(x: f32) -> f32 {
	if x < 0.5 { 8.0 * x * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powf(4.0) / 2.0 }
}

pub fn ease_in_quint(x: f32) -> f32 {
	x * x * x * x * x
}

pub fn ease_out_quint(x: f32) -> f32 {
	1.0 - (1.0 - x).powf(5.0)
}

pub fn ease_in_out_quint(x: f32) -> f32 {
	if x < 0.5 { 16.0 * x * x * x * x * x } else { 1.0 - (-2.0 * x + 2.0).powf(5.0) / 2.0 }
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
	1.0 - sqrt(1.0 - x * x)
}

pub fn ease_out_circ(x: f32) -> f32 {
	sqrt(1.0 - (x - 1.0).powf(2.0))
}

pub fn ease_in_out_circ(x: f32) -> f32 {
	if x < 0.5 { sqrt(1.0 - (1.0 - 2.0 * x).powf(2.0)) / 2.0 } else { (sqrt(1.0 - (-2.0 * x + 2.0).powf(2.0)) + 1.0) / 2.0 }
}

pub fn ease_in_back(x: f32) -> f32 {
	let c1 = 1.70158;
	let c3 = c1 + 1.0;
	c3 * x * x * x - c1 * x * x
}

pub fn ease_out_back(x: f32) -> f32 {
	let c1 = 1.70158;
	let c3 = c1 + 1.0;
	1.0 + c3 * (x - 1.0).powf(3.0) + c1 * (x - 1.0).powf(2.0)
}

pub fn ease_in_out_back(x: f32) -> f32 {
	let c1 = 1.70158;
	let c2 = c1 * 1.525;
	if x < 0.5 { (2.0 * x).powf(2.0) * ((c2 + 1.0) * 2.0 * x - c2) / 2.0 } else { ((2.0 * x - 2.0).powf(2.0) * ((c2 + 1.0) * (2.0 * x - 2.0) + c2) + 2.0) / 2.0 }
}

pub fn ease_in_elastic(x: f32) -> f32 {
	let c4 = (2.0 * PI) / 3.0;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else { -2.0_f32.powf(10.0 * x - 10.0) * sin((x * 10.0 - 10.75) * c4) }
}

pub fn ease_out_elastic(x: f32) -> f32 {
	let c4 = (2.0 * PI) / 3.0;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else { 2.0_f32.powf(-10.0 * x) * sin((x * 10.0 - 0.75) * c4) + 1.0 }
}

pub fn ease_in_out_elastic(x: f32) -> f32 {
	let c5 = (2.0 * PI) / 4.5;
	if x == 0.0 { 0.0 } else if x == 1.0 { 1.0 } else if x < 0.5 { -(2.0_f32.powf(20.0 * x - 10.0) * sin((20.0 * x - 11.125) * c5)) / 2.0 } else { (2.0_f32.powf(-20.0 * x + 10.0) * sin((20.0 * x - 11.125) * c5)) / 2.0 + 1.0 }
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