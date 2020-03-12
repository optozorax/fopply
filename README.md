# fopply - formula applier

This program applies formulas to mathematical expressions.

# example

Assume we have formula: `a+b <-> b+a`, when `<->` means that we can transform first in second and backwards.

Then we have expression: `x*y*(c+d) + a*b*c`. What if we apply formula to this expression? We will get this: `a*b*c + x*y*(c+d)`. 

So, program does exactly this. After `cargo run` we will see:
```
Formula: a+b <-> b+a

Expression: x*y*(c+d)+a*b*c

Bindings:
a -> x*y*(c+d);
b -> a*b*c;

Expression with applied formula: a*b*c+x*y*(c+d)
```

Formula and expression parsed from string by rust-peg.

# For what?

I think this approach can give to us safe and interactive software for symbolic calculations.