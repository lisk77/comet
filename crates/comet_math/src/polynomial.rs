use std::ops::*;

/// Representation of a polynomial of degree `n`.
pub struct Polynomial {
    coefficients: Vec<f32>,
    degree: usize,
}

impl Polynomial {
    /// Creates a new polynomial from a list of coefficients.
    pub fn new(coefficients: Vec<f32>) -> Self {
        let degree = coefficients.len() - 1;
        Self {
            coefficients,
            degree,
        }
    }

    /// Evaluates the polynomial at a given point.
    pub fn evaluate(&self, x: f32) -> f32 {
        let mut result = 0.0;
        for c in &self.coefficients {
            result = result * x + c;
        }
        result
    }

    /// Differentiates the polynomial.
    pub fn differentiate(&self) -> Self {
        let mut new_coefficients = Vec::new();
        for (i, &c) in self.coefficients.iter().enumerate() {
            if i != 0 {
                new_coefficients.push(c * i as f32);
            }
        }
        Self::new(new_coefficients)
    }

    /// Integrates the polynomial.
    pub fn integrate(&self) -> Self {
        let mut new_coefficients = Vec::new();
        new_coefficients.push(0.0);
        for (i, &c) in self.coefficients.iter().enumerate() {
            new_coefficients.push(c / (i + 1) as f32);
        }
        Self::new(new_coefficients)
    }
}

impl Add for Polynomial {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new_coefficients = Vec::new();
        let mut i = 0;
        while i < self.coefficients.len() || i < other.coefficients.len() {
            let a = if i < self.coefficients.len() {
                self.coefficients[i]
            } else {
                0.0
            };
            let b = if i < other.coefficients.len() {
                other.coefficients[i]
            } else {
                0.0
            };
            new_coefficients.push(a + b);
            i += 1;
        }
        Self::new(new_coefficients)
    }
}

impl Sub for Polynomial {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut new_coefficients = Vec::new();
        let mut i = 0;
        while i < self.coefficients.len() || i < other.coefficients.len() {
            let a = if i < self.coefficients.len() {
                self.coefficients[i]
            } else {
                0.0
            };
            let b = if i < other.coefficients.len() {
                other.coefficients[i]
            } else {
                0.0
            };
            new_coefficients.push(a - b);
            i += 1;
        }
        Self::new(new_coefficients)
    }
}

impl Mul for Polynomial {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut new_coefficients = vec![0.0; self.degree + other.degree + 1];
        for (i, &a) in self.coefficients.iter().enumerate() {
            for (j, &b) in other.coefficients.iter().enumerate() {
                new_coefficients[i + j] += a * b;
            }
        }
        Self::new(new_coefficients)
    }
}

impl Div for Polynomial {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        let mut new_coefficients = vec![0.0; self.degree - other.degree + 1];
        let mut dividend = self.coefficients.clone();
        let divisor = other.coefficients.clone();
        while dividend.len() >= divisor.len() {
            let mut quotient = vec![0.0; dividend.len() - divisor.len() + 1];
            let mut i = dividend.len() - divisor.len();
            quotient[i] = dividend.last().unwrap() / divisor.last().unwrap();
            for (j, &d) in divisor.iter().enumerate() {
                dividend[i + j] -= quotient[i] * d;
            }
            new_coefficients[i] = quotient[i];
            dividend.pop();
        }
        Self::new(new_coefficients)
    }
}

impl std::fmt::Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let terms: Vec<String> = self
            .coefficients
            .iter()
            .enumerate()
            .filter(|(_, &c)| c != 0.0)
            .map(|(i, &c)| {
                if i == 0 {
                    format!("{}", c)
                } else if i == 1 {
                    format!("{}x", c)
                } else {
                    format!("{}x^{}", c, i)
                }
            })
            .collect();
        write!(f, "{}", terms.join(" + "))
    }
}
