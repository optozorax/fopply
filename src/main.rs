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

fn count_functions(formula: &Tree) -> usize {
    match formula {
        Tree::Function { args, .. } => args.iter().map(|x| count_functions(x)).sum::<usize>() + 1,
        Tree::Variable { .. } => 1,
    }
}

enum TreeIteratorState {
    Start,
    Left,
    Right,
    Up,
    Return,
    End,
}

impl Tree {
    fn is_leaf(&self) -> bool {
        match self {
            Tree::Variable { .. } => true,
            Tree::Function { args, .. } => args.is_empty(),
        }
    }

    fn iter(&self) -> TreeIterator {
        /*TreeIterator {
            stack: vec![(0, self)],
            state: TreeIteratorState::Start,
        }*/
        let mut stack = Vec::with_capacity(100);
        stack.push(self);
        TreeIterator { stack }
    }
}

struct TreeIterator<'a> {
    /*stack: Vec<(usize, &'a Tree)>,
    state: TreeIteratorState,*/
    stack: Vec<&'a Tree>,
}

/*impl TreeIterator<'_> {
    fn current(&self) -> Option<&Tree> {
        Some(self.stack.last()?.1)
    }

    fn has_right(&self) -> bool {
        if let Some((pos, current)) = self.stack.last() {
            if let Tree::Function { args, .. } = current {
                pos < &args.len()
            } else {
                false
            }
        } else {
            false
        }
    }

    fn move_left(&mut self) -> Option<()> {
        if let Tree::Function { args, .. } = self.stack.last()?.1 {
            self.stack.push((0, &args[0]));
        } else {
            return None;
        }
        Some(())
    }

    fn move_right(&mut self) -> Option<()> {
        self.stack.last_mut()?.0 += 1;

        if self.has_right() {
            if let Tree::Function { args, .. } = self.stack.last()?.1 {
                self.stack.push((0, &args[self.stack.last()?.0]));
            } else {
                return None;
            }
        } else {
            return None;
        }
        Some(())
    }
}*/

impl<'a> Iterator for TreeIterator<'a> {
    type Item = &'a Tree;

    fn next(&mut self) -> Option<Self::Item> {
        let item: Self::Item = self.stack.pop()?;
        if let Tree::Function { args, .. } = item {
            for entry in args.iter().rev() {
                self.stack.push(&*entry);
            }
        }
        Some(item)
        /*use TreeIteratorState::*;
        loop {
            match self.state {
                Start => {
                    if self.current()?.is_leaf() {
                        self.state = Return;
                    } else {
                        self.state = Left;
                    }
                },
                Left => {
                    self.move_left()?;

                    if self.current()?.is_leaf() {
                        self.state = Return;
                    } else {
                        self.state = Left;
                    }
                },
                Right => {
                    self.move_right()?;

                    if self.current()?.is_leaf() {
                        self.state = Return;
                    } else {
                        self.state = Left;
                    }
                },
                Up => {
                    self.stack.pop()?;

                    if self.stack.is_empty() {
                        self.state = End;
                    } else if self.has_right() {
                        self.state = Right;
                    } else {
                        self.state = Return;
                    }

                    return Some(self.stack.last()?.1);
                },
                Return => {
                    self.state = Up;
                    return Some(self.stack.last()?.1);
                },
                End => {
                    return None;
                },
            }
        }*/
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

    println!("\nFormula: {}\n", formula);
    println!("Expression: {}\n", expr);
    println!("Bindings:\n{}\n", Bindings { bindings });
    println!("Expression with applied formula: {}\n", new_expr);

    println!("Count functions: {}", count_functions(&expr));

    for (index, i) in expr.iter().enumerate() {
        if let Some(bindings) = find_variables(&formula.from, i) {
            if let Some(new_expr) = apply(&formula.to, &bindings) {
                println!("{}, {} ------> {}", index, i, new_expr);
                continue;
            }
        }
        println!("{}, {}", index, i);
    }
}