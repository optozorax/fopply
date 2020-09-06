# fopply - formula applier

This program applies formulas to mathematical expressions.

# example

Assume we have formula: `a+b <-> b+a`, when `<->` means that we can transform first in second and backwards.

Then we have expression: `x*y*(c+d) + a*b*c`. What if we apply formula to this expression? We will get this: `a*b*c + x*y*(c+d)`. 

# binding

In previous example we have replacements for formula called `bindings`:
```
Formula: a+b <-> b+a

Expression: x*y*(c+d) + a*b*c

Bindings:
a := x*y*(c+d);
b := a*b*c;

Expression with applied formula: a*b*c + x*y*(c+d)
```

# function binding

Imagine we have formula `a = b & $f(a) <-> a = b & $f(b)`, this formula means that if we have logic formula where `a = b`, then we can replace `a` to `b`. `$f(a)` means `any function named f`. This formula can be applied to:
```
1. a = 1 & a -> a = 1 & 1
2. a = 1 & c+a -> a = 1 & c+1
3. a = 1 & x*a + b*a + a*a -> a = 1 & x*1 + b*a + a*a
4. a = 1 & x*a + b*a + a*a -> a = 1 & x*1 + b*1 + a*a
5. a = 1 & x*a + b*a + a*a -> a = 1 & x*1 + b*1 + 1*1
```

So, how we find `a` in any function `$f`? I created `functional binding` that can be writed like that: `$f(inner) := a*inner + other`, here we write pattern to find all that we need to match. For this functional binding we get 3 expression. Here is functional bindings for all expressions:
```
1. $f(inner) = inner
2. $f(inner) = c+inner
3. $f(inner) = a*inner + other
4. $f(inner) = x*inner + b*inner + other
5. $f(a) = x*a + b*a + a*a
```

We can write it how we want, it just must fit the pattern.

# proof

We can write how one formula can be derived from others:
```
[eq]
...
2. a = a <-> $true;

[or]
...
5. a | $true <-> $true;

[ltgteq]
1. a <= b <-> (a < b) | (a = b);
...
3. a <= a <-> $true {
	a <= a;
	^^^^^^ ltgteq.1l;
	(a < a) | (a = a);
	.          ^^^^^ eq.2l;
	(a < a) | $true;
	^^^^^^^^^^^^^^^ or.5l;
};
```

# math.fpl

In file `fpl/math.fpl` you can find current axioms-formulas and derived formulas.

# For what?

I think this approach can give to us safe and interactive software for symbolic calculations.