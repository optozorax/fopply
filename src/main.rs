use itertools::Itertools;
use colored::*;

peg::parser!( grammar arithmetic() for str {
    pub rule formulas() -> Vec<Transformation>
        = r:(t:transformation() _ ";" _ {t})+ {
            r
        }

    pub rule transformation() -> Transformation
        = l:expr() _ "<->" _ r:expr() {
            Transformation { from: l, to: r }
        }

    pub rule expr() -> Tree
        = sum()

    rule sum() -> Tree
        = l:product() r:(_ z:$("+"/"-") _ p:product() { (z, p) })* {
            r.into_iter().fold(l, |acc, (z, p)| Tree::function(z.to_string(), vec![acc, p]))
        }

    rule product() -> Tree
        = l:power() r:(_ z:$("*"/"/") _ p:power() { (z, p) })* {
            r.into_iter().fold(l, |acc, (z, p)| Tree::function(z.to_string(), vec![acc, p]))
        }

    rule power() -> Tree
        = l:atom() _ r:("^" p:power() { p })? {
            match r {
                Some(r) => Tree::function("^".to_string(), vec![l, r]),
                None => l,
            }            
        }

    rule atom() -> Tree
        = "(" v:expr() ")" { v }
        / float_number()
        / number()
        / variable()
        / "-" _ v:atom() { Tree::function("negative".to_string(), vec![v]) }

    rule float_number() -> Tree
        = n:$(['0'..='9']+ "." ['0'..='9']+) { Tree::float(n.parse().unwrap()) }

    rule number() -> Tree
        = n:$(['0'..='9']+) { Tree::number(n.parse().unwrap()) }

    rule variable() -> Tree
        = n:$(['a'..='z']+) { Tree::variable(n.to_string()) }

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
    },
    Number {
        value: i64,
    },
    Float {
        value: f64,
    }
}

impl Tree {
    fn variable(name: String) -> Self {
        Tree::Variable { name }
    }

    fn number(value: i64) -> Self {
        Tree::Number { value }
    }

    fn float(value: f64) -> Self {
        Tree::Float { value }
    }

    fn function(name: String, args: Vec<Tree>) -> Self {
        Tree::Function { name, args }
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
        (Tree::Number { value: v1 }, Tree::Number { value: v2 })
        if v1 == v2 => Some(vec![]),
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
        x @ Tree::Number { .. } => Some(x.clone()),
        x @ Tree::Float { .. } => Some(x.clone()),
    }
}

fn count_functions(formula: &Tree) -> usize {
    match formula {
        Tree::Function { args, .. } => args.iter().map(|x| count_functions(x)).sum::<usize>() + 1,
        Tree::Variable { .. } | Tree::Number { .. } | Tree::Float { .. } => 1,
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
            Tree::Variable { .. } | Tree::Number { .. } | Tree::Float { .. } => true,
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
            write!(f, "{}{}{}", "(".cyan(), self.t, ")".cyan())
        } else {
            write!(f, "{}", self.t)
        }
    }
}

fn calc_brackets<'a>(formula: &'a Tree, current_op: &str, pos: usize) -> PrintingBrackets<'a, Tree> {
    if let Tree::Function { name: next, .. } = formula {
        let is_print = 
            current_op == "/" && pos == 0 && next != "*" && next != "/" ||
            current_op == "/" && pos == 1 ||
            current_op == "-" && pos == 0 && next != "+" && next != "-" ||
            current_op == "-" && pos == 1 ||
            current_op == "*" && next != "*" && next != "/";
            //current_op == "+" && next == "+" && next == "-";
        PrintingBrackets { is_print, t: formula }
    } else {
        PrintingBrackets { is_print: false, t: formula }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tree::Function { name, args } 
                if args.len() == 2 => { 
                    write!(f, 
                        "{}{}{}", 
                        calc_brackets(&args[0], name, 0), 
                        //if name != "*" {name} else {""}, 
                        name.bright_green(),
                        calc_brackets(&args[1], name, 1)
                    )
                },
            Tree::Variable { name } => { write!(f, "{}", name) },
            Tree::Number { value } => { write!(f, "{}", value.to_string().bright_yellow()) },
            Tree::Float { value } => { write!(f, "{}", value.to_string().bright_yellow()) }
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
        write!(f, "{} {} {}", self.from, "<->".bright_magenta(), self.to)
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

fn eval(expr: &Tree) -> Option<f64> {
    use Tree::*;
    match expr {
        Function { name, args } if name == "negative" && args.len() == 1 => Some(-eval(&args[0])?),
        Function { name, args } if name == "+" && args.len() == 2 => Some(eval(&args[0])? + eval(&args[1])?),
        Function { name, args } if name == "-" && args.len() == 2 => Some(eval(&args[0])? - eval(&args[1])?),
        Function { name, args } if name == "*" && args.len() == 2 => Some(eval(&args[0])? * eval(&args[1])?),
        Function { name, args } if name == "/" && args.len() == 2 => Some(eval(&args[0])? / eval(&args[1])?),
        Variable { .. } => None,
        Number { value } => Some(*value as f64),
        Float { value } => Some(*value), 
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        use arithmetic::*;

        macro_rules! test_expr {
            ($($x:tt)*) => {{
                let string = stringify!($($x)*);
                let expr = expr(&string).expect(&format!("Cant't parse: {}", string));
                assert_eq!(eval(&expr), Some(($($x)*) as f64));
            }};
        }

        test_expr!(5.0);
        test_expr!(-1.0);
        test_expr!((5.0));
        test_expr!((-(5.0)));
        test_expr!(-(-(-5.0)));

        test_expr!(1.0+2.0);
        test_expr!(1.0-2.0);
        test_expr!(1.0*2.0);
        test_expr!(1.0/2.0);

        test_expr!(1.0+-2.0);
        test_expr!(1.0--2.0);
        test_expr!(1.0*-2.0);
        test_expr!(1.0/-2.0);

        test_expr!(1.0+(-2.0));
        test_expr!(1.0-2.0-3.0);
        test_expr!(1.0+2.0+3.0);
        test_expr!(1.0-2.0*2.0/1.0);
        test_expr!(2.0*5.0*4.0/3.0);
        test_expr!(1.0-(2.0-(3.0/5.0)));
    }
}

fn apply_recursively(formula: Tree, transformation: &Transformation) -> (bool, Tree) {
    if let Some(bindings) = find_variables(&transformation.from, &formula) {
        if let Some(new_expr) = apply(&transformation.to, &bindings) {
            return (true, new_expr);
        }
    }
    match formula {
        Tree::Function { mut args, name } => {
            let mut result = false;
            args = args.into_iter().map(|x| {
                let (add_result, expr) = apply_recursively(x, transformation);
                result |= add_result;
                expr
            }).collect();
            (result, Tree::Function { name, args })
        },
        formula => (false, formula),
    }
}

fn aplly_recursively_while_applied(mut e: Tree, formula: &Transformation) -> (bool, Tree) {
    let mut result = false;

    let mut string = format!("{} {}\n", "Formula:".red(), formula);
    for _ in 0..4 {
        string += &format!("{} {} ", e, "--->".bright_blue());
        let (b, e1) = apply_recursively(e, &formula);
        e = e1;
        if !b { break; } else { println!("{}{}", string, e); result = true; string = String::new(); }
    }

    if result {
        println!();
    }

    (result, e)
}

fn apply_formulas() {
    // TODO добавить формулы-or, которые применяются последовательно, и если хотя бы одна применилась, то возвращается true, и соответственно с ними добавить формулы с вычитанием
    use arithmetic::*;
    let formula1 = transformation("a/b*c <-> a*c/b").expect("wrong transformation");
    let formula2 = transformation("a/b/c <-> a/(b*c)").expect("wrong transformation");
    let formula3 = transformation("a/b + c/d <-> (a*d+c*b)/(b*d)").expect("wrong transformation");
    let formula3_1 = transformation("a + c/d <-> (a*d+c)/d").expect("wrong transformation");
    let formula3_2 = transformation("a/b + c <-> (a+c*b)/b").expect("wrong transformation");
    let formula4 = transformation("c*(a+b) <-> (a+b)*c").expect("wrong transformation");
    let formula5 = transformation("(a+b)*c <-> a*c+b*c").expect("wrong transformation");
    let formula6 = transformation("1*a <-> a").expect("wrong transformation");
    let formula7 = transformation("a*1 <-> a").expect("wrong transformation");
    let mut e = expr("c*(a+b)+1/2").unwrap();
    let start = e.clone();

    println!();
    println!("{} {}\n", "Expression:".red(), e);

    e = aplly_recursively_while_applied(e, &formula1).1;
    e = aplly_recursively_while_applied(e, &formula2).1;
    e = aplly_recursively_while_applied(e, &formula3).1;
    e = aplly_recursively_while_applied(e, &formula3_1).1;
    e = aplly_recursively_while_applied(e, &formula3_2).1;

    let mut b = true;
    while b {
        e = aplly_recursively_while_applied(e, &formula4).1;
        let (b1, e1) = aplly_recursively_while_applied(e, &formula5);    
        b = b1;
        e = e1;
    }
    e = aplly_recursively_while_applied(e, &formula6).1;
    e = aplly_recursively_while_applied(e, &formula7).1;

    println!("result:\n{} ---> {}\n", start, e);
}    

fn main() {
    apply_formulas();
    return;

    use arithmetic::*;
    let formula = transformation("a+b <-> b+a").expect("wrong transformation");
    let e = expr("x*y*(c+d) + a*b*c").unwrap();
    let bindings = find_variables(&formula.from, &e).expect("cant find formula pattern");
    let new_expr = apply(&formula.to, &bindings).expect("cant apply formula");

    println!("\nFormula: {}\n", formula);
    println!("Expression: {}\n", e);
    println!("Bindings:\n{}\n", Bindings { bindings });
    println!("Expression with applied formula: {}\n", new_expr);

    println!("Count functions: {}", count_functions(&e));

    for (index, i) in e.iter().enumerate() {
        if let Some(bindings) = find_variables(&formula.from, i) {
            if let Some(new_expr) = apply(&formula.to, &bindings) {
                println!("{}, {} ------> {}", index, i, new_expr);
                continue;
            }
        }
        println!("{}, {}", index, i);
    }

    println!("{:#?}", expr("1-2*3/5^2-0"));

    println!("{:#?}", formulas("a+b <-> b+a; a*b <-> b*a; a-b <-> a+(0-b);"))
}