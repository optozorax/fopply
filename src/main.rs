use itertools::Itertools;

peg::parser!( grammar arithmetic() for str {
    pub rule transformation() -> Transformation
        = a:expr() _ "<->" _ b:expr() {
            Transformation { from: a, to: b }
        }

    pub rule expr() -> Tree
        = sum()

    rule sum() -> Tree
        = l:product() _ "+" _ r:sum() { 
            Tree::Function { name: "+".to_string(), args: vec![l, r] } 
        }
        / product()

    rule product() -> Tree
        = l:atom() _ "*" _ r:product() {
            Tree::Function { name: "*".to_string(), args: vec![l, r] } 
        }
        / atom()

    rule atom() -> Tree
        = variable()
        / "(" _ v:sum() _ ")" { v }

    rule variable() -> Tree
        = n:$(['a'..='z']+) { Tree::Variable { name: n.to_string() } }

    rule _() = [' ' | '\n']*
});

#[derive(Debug, Clone)]
pub struct Transformation {
    from: Tree,
    to: Tree,
}

#[derive(Debug, Clone)]
pub enum Tree {
    Function {
        name: String,
        args: Vec<Tree>,
    },
    Variable {
        name: String,
    }
}

struct Binding {
    variable: String,
    formula: Tree,
}

fn find_variables(formula: &Tree, expr: &Tree) -> Option<Vec<Binding>> {
    match (formula, expr) {
        (Tree::Function { name: n1, args: a }, Tree::Function { name: n2, args: b }) 
        if n1 == n2 && a.len() == b.len() => {
            a.iter()
            .zip(b.iter())
            .map(|(a, b)| find_variables(&a, &b))
            .try_fold(vec![], |mut acc, x| { 
                acc.extend(x?); 
                Some(acc)
            })
        },
        (Tree::Variable { name }, expr) => {
            Some(vec![Binding { variable: name.clone(), formula: expr.clone() }])
        },
        _ => None,
    }
}

fn apply(formula: &Tree, bindings: &[Binding]) -> Option<Tree> {
    match formula {
        Tree::Function { name, args } => Some(Tree::Function { 
            name: name.clone(), 
            args: args.iter()
                .map(|arg| apply(arg, bindings))
                .collect::<Option<Vec<Tree>>>()? 
        }),
        Tree::Variable { name } => 
            bindings.iter()
            .find(|x| &x.variable == name)
            .map(|x| x.formula.clone()),
    }
}

use std::fmt;

struct PrintingBrackets<'a, T: 'a> {
    is_print: bool,
    t: &'a T,
}

impl<'a, T: 'a + fmt::Display> fmt::Display for PrintingBrackets<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_print {
            write!(f, "({})", self.t)
        } else {
            write!(f, "{}", self.t)
        }
    }
}

fn calc_brackets<'a>(formula: &'a Tree, current_op: &str, pos: usize) -> PrintingBrackets<'a, Tree> {
    if let Tree::Function { name: next, .. } = formula {
        let is_print = next == "+" && current_op == "*" ||
        next == current_op && pos == 0;
        PrintingBrackets { is_print, t: formula }
    } else {
        PrintingBrackets { is_print: false, t: formula }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tree::Function { name, args } if name == "+" && args.len() == 2 => { write!(f, "{}+{}", calc_brackets(&args[0], "+", 0), calc_brackets(&args[1], "+", 1))  },
            Tree::Function { name, args } if name == "*" && args.len() == 2 => { write!(f, "{}*{}", calc_brackets(&args[0], "*", 0), calc_brackets(&args[1], "*", 1))  },
            Tree::Variable { name } => { write!(f, "{}", name) },
            _ => Err(fmt::Error)
        }
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.variable, self.formula)
    }
}

impl fmt::Display for Transformation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <-> {}", self.from, self.to)
    }
}

struct Bindings {
    bindings: Vec<Binding>
}

impl fmt::Display for Bindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.bindings.iter().with_position() {
            use itertools::Position::*;
            match i {
                First(i) | Middle(i) => writeln!(f, "{};", i)?,
                Last(i) | Only(i) => write!(f, "{};", i)?,
            }
        }
        Ok(())
    }   
}

fn main() {
    use arithmetic::{transformation, expr};
    let formula = transformation("a+b <-> b+a").expect("wrong transformation");
    let expr = expr("x*y*(c+d) + a*b*c").unwrap();
    let bindings = find_variables(&formula.from, &expr).expect("cant find formula pattern");
    let new_expr = apply(&formula.to, &bindings).expect("cant apply formula");

    println!("Formula: {}\n", formula);
    println!("Expression: {}\n", expr);
    println!("Bindings:\n{}\n", Bindings { bindings });
    println!("Expression with applied formula: {}", new_expr);
}