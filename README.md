# naive derivative solver

a very basic derivative solver i made to test my understanding of simple calculus:
- chain rule
- product rule
- power rule

this is a toy project, considered finished. i don't plan on adding any other major features.

## terms

a `Term` is an abstract math expression.

the following operations are supported between `Term`s:
- `Add(t1, t2)` (subtraction is done using negative coefficients)
- `Mul(t1, t2)` (division is done using negative exponents)
- `Pow(t1, t2)`
- `Ln(t1)` (log_a of b must be represented as ln(b)/ln(a))
- `Sin(t1)`
- `Cos(t1)`

in addition, the following constants/variables are given:
- `X` (the only variable)
- `E` (euler's constant)
- `Pi`
- `Int(i)` (uses 64-bit signed integers)

## estimation

`Term::estimate()` allows you to plug in a 64-bit floating point value and approximate the result of substituting that number for `X`

`NaN` is returned on failure.

## differentiation

`Term::derivative()` returns the simplified derivative of the term. derivatives are always guaranteed to be correct - however, note that expressions usually aren't reduced optimally, so some results may be more complex than necessary.

if you want the second derivative, you can simply do `t1.derivative().derivative()`... and so on.

## reduction

`Term::simplified()` does very basic reductions on a term, for example:
- `Add(Mul(Int(1), Int(2)), Int(3))` -> `Int(5)`
- `Ln(E)` -> `Int(1)`
- `Mul(Int(0), Ln(Pow(X, Int(8))))` -> `Int(0)`
- `Mul(Pow(X, Int(2)), Pow(X, Int(-2)))` -> `Int(1)`

i'm pretty sure all of my reductions are mathematically correct (hopefully they are). my goal is for these to be *good enough*, not perfect. i know i'm missing a lot of reductions.

also, the way i did reduction is probably not super efficient - i wouldn't be surprised if it could be 10+ times faster. but it's good enough for me.

i actually did implement the property that sin(x)^2 + cos(x)^2 = 1, and it should be able to simplify that in the most basic of cases.

## note on printing values

i'm aware that there are way too many parentheses when you print a term. however, i don't really feel like fixing it.