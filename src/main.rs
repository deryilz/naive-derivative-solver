// deriv solver

mod term;
mod term_old;

fn main() {
    use term::Term::{self, *};

    // let expr = X / (X + Int(1));
    // let expr = Int(3)*X.sin().pow(Int(2)) + Int(3)*X.cos().pow(Int(2));
    let expr = Int(-2) * X.sin() * X.sin() + (Int(-2) * X.cos() * X.cos());
    // let expr = X.pow(X);
    println!("{:?}", expr);
    let d = expr.derivative();
    println!("{}", d);
}
