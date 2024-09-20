use std::fmt::Display;
use std::ops::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Term {
    Int(i64),
    E,
    X,
    Pi,
    Add(Box<Term>, Box<Term>),
    Mul(Box<Term>, Box<Term>),
    Pow(Box<Term>, Box<Term>),
    Ln(Box<Term>),
    Sin(Box<Term>),
    Cos(Box<Term>),
}

impl Term {
    pub fn derivative(&self) -> Term {
        match self {
            Self::E | Self::Pi | Self::Int(_) => Self::Int(0),
            Self::X => Self::Int(1),
            Self::Ln(term) => term.derivative() / *term.clone(),
            Self::Sin(term) => term.derivative() * term.cos(),
            Self::Cos(term) => Self::Int(-1) * term.derivative() * term.sin(),
            Self::Add(term1, term2) => term1.derivative() + term2.derivative(),
            Self::Mul(term1, term2) => {
                term1.derivative() * *term2.clone() + *term1.clone() * term2.derivative()
            }
            Self::Pow(term1, term2) => self.clone() * (*term2.clone() * term1.ln()).derivative(),
        }
        .simplified()
    }

    pub fn simplified(&self) -> Term {
        let simpler = match self {
            Self::E | Self::Pi | Self::Int(_) | Self::X => self.clone(), // good
            Self::Add(term1, term2) => {
                let simple1 = term1.simplified();
                let simple2 = term2.simplified();
                match (&simple1, &simple2) {
                    (Self::Int(0), _) => simple2,
                    (_, Self::Int(0)) => simple1,
                    (Self::Int(a), Self::Int(b)) => Self::Int(a + b),
                    _ if simple1 == simple2 => simple1 * Self::Int(2),
                    _ => simple1 + simple2,
                }
            }
            Self::Mul(term1, term2) => {
                let simple1 = term1.simplified();
                let simple2 = term2.simplified();
                match (&simple1, &simple2) {
                    (Self::Int(0), _) => Self::Int(0),
                    (_, Self::Int(0)) => Self::Int(0),
                    (Self::Int(1), _) => simple2,
                    (_, Self::Int(1)) => simple1,
                    (Self::Int(a), Self::Int(b)) => Self::Int(a * b),
                    _ => simple1 * simple2,
                }
            }
            Self::Pow(term1, term2) => {
                let simple1 = term1.simplified();
                let simple2 = term2.simplified();
                match (&simple1, &simple2) {
                    (_, Self::Int(0)) => Self::Int(1),
                    (Self::Int(0), _) => Self::Int(0),
                    (_, Self::Int(1)) => simple1,
                    (Self::Int(1), _) => Self::Int(1),
                    (Self::Int(a), Self::Int(b)) if *b > 0 => Self::Int(a.pow(*b as u32)),
                    _ => simple1.pow(simple2), // todo
                }
            } // alright i think
            Self::Ln(inner) => match inner.simplified() {
                Self::Pow(term1, term2) => *term2.clone() * term1.ln(),
                Self::E => Self::Int(1), // figure out how to do multiples
                Self::Int(1) => Self::Int(0),
                // Add more later
                simple => simple.ln(),
            },
            Self::Cos(inner) => match inner.simplified() {
                // Add more later
                Self::Int(0) => Self::Int(1),
                Self::Pi => Self::Int(-1),
                simple => simple.cos(),
            },
            Self::Sin(inner) => match inner.simplified() {
                // Be able to take Pi out of stuff
                Self::Int(0) => Self::Int(0),
                Self::Pi => Self::Int(0),
                simple => simple.sin(),
            },
        };
        if &simpler == self {
            simpler
        } else {
            simpler.simplified()
        }
    }

    pub fn estimate(&self, value: f64) -> f64 {
        use std::f64::consts;
        match self {
            Self::Int(a) => *a as f64,
            Self::X => value,
            Self::E => consts::E,
            Self::Pi => consts::PI,
            Self::Add(a, b) => a.estimate(value) + b.estimate(value),
            Self::Mul(a, b) => a.estimate(value) * b.estimate(value),
            Self::Pow(a, b) => a.estimate(value).powf(b.estimate(value)),
            Self::Ln(a) => a.estimate(value).ln(),
            Self::Sin(a) => a.estimate(value).sin(),
            Self::Cos(a) => a.estimate(value).cos(),
        }
    }

    pub fn ln(&self) -> Term {
        Self::Ln(Box::new(self.clone()))
    }

    pub fn log_base(&self, base: Term) -> Term {
        self.ln() / base.ln()
    }

    pub fn sin(&self) -> Term {
        Self::Sin(Box::new(self.clone()))
    }

    pub fn cos(&self) -> Term {
        Self::Cos(Box::new(self.clone()))
    }

    pub fn pow(&self, other: Term) -> Term {
        Self::Pow(Box::new(self.clone()), Box::new(other))
    }

    pub fn sqrt(&self) -> Term {
        self.pow(Self::Int(1) / Self::Int(2))
    }

    fn hash_num(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    // Add(3x, Add(Add(7, 3x), 5))
    // [3x, 7, 3x, 5]
    // [3x, 3x, 7, 5]
    //
}

impl Add for Term {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::Add(Box::new(self), Box::new(other))
    }
}

impl Sub for Term {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self + other.mul(Self::Int(-1))
    }
}

impl Mul for Term {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::Mul(Box::new(self), Box::new(other))
    }
}

impl Div for Term {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        self * other.pow(Self::Int(-1))
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E => write!(f, "(e)"),
            Self::Pi => write!(f, "(pi)"),
            Self::X => write!(f, "(x)"),
            Self::Int(a) => write!(f, "({a})"),
            Self::Add(a, b) => write!(f, "({a} + {b})"),
            Self::Mul(a, b) => write!(f, "({a} * {b})"),
            Self::Pow(a, b) => write!(f, "({a} ^ {b})"),
            Self::Ln(a) => write!(f, "(ln{a})"),
            Self::Sin(a) => write!(f, "(sin{a})"),
            Self::Cos(a) => write!(f, "(cos{a})"),
        }
    }
}
