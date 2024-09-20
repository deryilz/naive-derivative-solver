use std::collections::BTreeMap;
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

use Term::*;

impl Term {
    pub fn derivative(&self) -> Term {
        match self.simplified() {
            E | Pi | Int(_) => Int(0),
            X => Int(1),
            Ln(term) => term.derivative() / *term.clone(),
            Sin(term) => term.derivative() * term.cos(),
            Cos(term) => Int(-1) * term.derivative() * term.sin(),
            Add(term1, term2) => term1.derivative() + term2.derivative(),
            Mul(term1, term2) => {
                term1.derivative() * *term2.clone() + *term1.clone() * term2.derivative()
            }
            Pow(term1, term2) => self.clone() * (*term2.clone() * term1.ln()).derivative(),
        }
        .simplified()
    }

    pub fn simplified(&self) -> Term {
        let simpler = match self {
            E | Pi | Int(_) | X => self.clone(),
            Add(..) => {
                let terms = self.flatten_add();
                let mut occurences = BTreeMap::new();
                for term in terms {
                    let (val, coeff) = match term {
                        Mul(a, b) => match (a.as_ref(), b.as_ref()) {
                            (_, Int(i)) => (*a, *i),
                            (Int(i), _) => (*b, *i),
                            _ => (*a * *b, 1),
                        },
                        other => (other, 1),
                    };
                    let existing = occurences.remove(&val).unwrap_or(0);
                    occurences.insert(val, coeff + existing);
                }
                // sin^2 + cos^2 = 1 (integer version only)
                let sin_squared = X.sin().pow(Int(2));
                let cos_squared = X.cos().pow(Int(2));
                if let Some(&num_sin) = occurences.get(&sin_squared) {
                    if let Some(&num_cos) = occurences.get(&cos_squared) {
                        if num_sin == num_cos {
                            occurences.remove(&sin_squared);
                            occurences.remove(&cos_squared);
                            let times = Int(num_sin);
                            let existing = occurences.remove(&times).unwrap_or(0);
                            occurences.insert(times, 1 + existing);
                        }
                    }
                }
                occurences
                    .into_iter()
                    .map(|(key, value)| (key * Int(value)).simplified())
                    .reduce(|old, new| match (&old, &new) {
                        (Int(0), _) => new,
                        (_, Int(0)) => old,
                        (Int(a), Int(b)) => Int(a + b),
                        _ => old + new,
                    })
                    .unwrap()
                // let simple1 = term1.simplified();
                // let simple2 = term2.simplified();
                // match (&simple1, &simple2) {
                //     (Int(0), _) => simple2,
                //     (_, Int(0)) => simple1,
                //     (Int(a), Int(b)) => Int(a + b),
                //     _ => simple1 + simple2,
                // }
            }
            Mul(..) => {
                // let simple = term1.simplified() * term2.simplified();
                let terms = self.flatten_mul();
                let mut occurences = BTreeMap::new();
                for term in terms {
                    let (base, exp) = match term {
                        Pow(base, exp) => (*base, *exp),
                        other => (other, Int(1)),
                    };
                    if let Some(existing) = occurences.remove(&base) {
                        occurences.insert(base, exp + existing);
                    } else {
                        occurences.insert(base, exp);
                    };
                }
                occurences
                    .into_iter()
                    .map(|(key, value)| key.pow(value).simplified())
                    .reduce(|old, new| match (&old, &new) {
                        (Int(0), _) => Int(0),
                        (_, Int(0)) => Int(0),
                        (Int(1), _) => new,
                        (_, Int(1)) => old,
                        (Int(a), Int(b)) => Int(a * b),
                        _ => old * new,
                    })
                    .unwrap()

                // let simple1 = term1.simplified();
                // let simple2 = term2.simplified();
                // match (&simple1, &simple2) {
                //     (Int(0), _) => Int(0),
                //     (_, Int(0)) => Int(0),
                //     (Int(1), _) => simple2,
                //     (_, Int(1)) => simple1,
                //     (Int(a), Int(b)) => Int(a * b),
                //     _ => simple1 * simple2,
                // }
            }
            Pow(term1, term2) => {
                let simple1 = term1.simplified();
                let simple2 = term2.simplified();
                match (&simple1, &simple2) {
                    (_, Int(0)) => Int(1),
                    (Int(0), _) => Int(0),
                    (_, Int(1)) => simple1,
                    (Int(1), _) => Int(1),
                    (E, Ln(a)) => *a.clone(),
                    (Int(a), Int(b)) if *b > 0 => Int(a.pow(*b as u32)),
                    _ => simple1.pow(simple2), // todo
                }
            } // alright i think
            Ln(inner) => match inner.simplified() {
                Pow(term1, term2) => *term2 * term1.ln(),
                E => Int(1),
                Int(1) => Int(0),
                // Add more later
                simple => simple.ln(),
            },
            Cos(inner) => match inner.simplified() {
                // Add more later
                Int(0) => Int(1),
                Pi => Int(-1),
                simple => simple.cos(),
            },
            Sin(inner) => match inner.simplified() {
                // Be able to take Pi out of stuff
                Int(0) => Int(0),
                Pi => Int(0),
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
            Int(a) => *a as f64,
            X => value,
            E => consts::E,
            Pi => consts::PI,
            Add(a, b) => a.estimate(value) + b.estimate(value),
            Mul(a, b) => a.estimate(value) * b.estimate(value),
            Pow(a, b) => a.estimate(value).powf(b.estimate(value)),
            Ln(a) => a.estimate(value).ln(),
            Sin(a) => a.estimate(value).sin(),
            Cos(a) => a.estimate(value).cos(),
        }
    }

    pub fn flatten_mul(&self) -> Vec<Term> {
        if let Mul(term1, term2) = self {
            [term1.flatten_mul(), term2.flatten_mul()].concat()
        } else {
            vec![self.clone()]
        }
    }

    pub fn flatten_add(&self) -> Vec<Term> {
        if let Add(term1, term2) = self {
            [term1.flatten_add(), term2.flatten_add()].concat()
        } else {
            vec![self.clone()]
        }
    }

    pub fn ln(&self) -> Term {
        Ln(Box::new(self.clone()))
    }

    pub fn log_base(&self, base: Term) -> Term {
        self.ln() / base.ln()
    }

    pub fn sin(&self) -> Term {
        Sin(Box::new(self.clone()))
    }

    pub fn cos(&self) -> Term {
        Cos(Box::new(self.clone()))
    }

    pub fn pow(&self, other: Term) -> Term {
        Pow(Box::new(self.clone()), Box::new(other))
    }

    pub fn sqrt(&self) -> Term {
        self.pow(Int(1) / Int(2))
    }

    // fn hash_num(&self) -> u64 {
    //     use std::collections::hash_map::DefaultHasher;
    //     use std::hash::{Hash, Hasher};
    //     let mut hasher = DefaultHasher::new();
    //     self.hash(&mut hasher);
    //     hasher.finish()
    // }
}

impl std::ops::Add for Term {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Add(Box::new(self), Box::new(other))
    }
}

impl std::ops::Sub for Term {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self + other * Int(-1)
    }
}

impl std::ops::Mul for Term {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Mul(Box::new(self), Box::new(other))
    }
}

impl std::ops::Div for Term {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        self * other.pow(Int(-1))
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
