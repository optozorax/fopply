use std::ops::Range;
use std::fmt::Debug;
use std::collections::BTreeMap;

trait GetInnerExpression: Sized {
	fn get_inner_expression(self) -> ExpressionMeta<Self>;
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self>;
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self>;
}

/// Обобщённое выражение, нужно для возможности делать парсинг
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionMeta<ArgType> {
	/// `a`, `b`, `c` - матчится с чем угодно, именованная часть выражения
	Pattern {
		name: String
	},

	/// `a#n` - матчится только с тем, что проходит внутреннюю проверку
	ConstrainedPattern {
		/// a
		name: String,
		/// n
		// TODO сделать это enum'ом
		constrained_pattern_name: String,
	},

	/// `$f(a, b)` - функция с неизвестным именем с любым числом аргументов
	AnyFunction {
		name: String,
		args: Vec<ArgType>,
	},

	/// `a+b`, `sin(1)` - функция с именем и определённым набором аргументов
	NamedFunction {
		name: String,
		args: Vec<ArgType>,
	},

	/// `$false`, `$true`, `$empty` - конкретная строка-значение
	NamedValue {
		name: String
	},

	/// `1`, `1000` - конкретное число-значение
	IntegerValue {
		value: i64
	},
}

/// Выражение
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression(ExpressionMeta<Expression>);

impl GetInnerExpression for Expression {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.0 }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.0 }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.0 }
}

/// `left <-> right`
#[derive(Clone, Debug)]
pub struct Formula {
	pub left: Expression,
	pub right: Expression,

	// TODO добавить проверку этих штук, что они должны быть найдены
	//left_to_right_needed_bindings: Vec<BindingKey>,
	//right_to_left_needed_bindings: Vec<BindingKey>,
}

/// `variable -> value`, позволяет производить замену с имени паттерна на выражение
#[derive(Clone, Debug)]
pub enum Binding {
	Pattern {
		key: String,
		value: Expression,
	},
	ConstrainedPattern {
		key: String,
		value: Expression,
	},
	AnyFunctionToNamedFunction {
		key: String,
		value: String,
	},
	AnyFunctionToAnyFunction {
		key: String,
		value: String,
	},
}

impl Binding {
	fn for_pattern(from_name: String, to_value: Expression) -> Binding {
		Binding::Pattern {
			key: from_name,
			value: to_value,
		}
	}

	fn for_constrained_pattern(from_name: String, to_value: Expression) -> Binding {
		Binding::ConstrainedPattern {
			key: from_name,
			value: to_value,
		}
	}

	fn for_any_function_to_named_function(from_any_function_name: String, to_named_function_name: String) -> Binding {
		Binding::AnyFunctionToNamedFunction {
			key: from_any_function_name,
			value: to_named_function_name,
		}
	}

	fn for_any_function_to_any_function(from_any_function_name: String, to_any_function_name: String) -> Binding {
		Binding::AnyFunctionToAnyFunction {
			key: from_any_function_name,
			value: to_any_function_name,
		}
	}
}

#[derive(Default, Debug)]
pub struct BindingStorage {
	map_pattern: BTreeMap<String, Expression>,
	map_constrained_pattern: BTreeMap<String, Expression>,
	map_any_function_to_named_function: BTreeMap<String, String>,
	map_any_function_to_any_function: BTreeMap<String, String>,
}

impl BindingStorage {
	/// Добавляет привязку в хранилище. Если такая привязка уже существует, то проверяет что они совпадают. Если это не так, возвращает None.
	pub fn add(&mut self, binding: Binding) -> Option<()> {
		use std::collections::btree_map::Entry::*;
		use Binding::*;

		match binding {
			Pattern { key, value } => { 
				match self.map_pattern.entry(key) {
					Vacant(vacant) => {
						vacant.insert(value);
						Some(())
					},
					Occupied(occupied) => {
						if *occupied.get() == value {
							Some(())
						} else {
							None
						}
					},
				}
			},
			ConstrainedPattern { key, value } => { 
				match self.map_constrained_pattern.entry(key) {
					Vacant(vacant) => {
						vacant.insert(value);
						Some(())
					},
					Occupied(occupied) => {
						if *occupied.get() == value {
							Some(())
						} else {
							None
						}
					},
				}
			},
			AnyFunctionToNamedFunction { key, value } => { 
				match self.map_any_function_to_named_function.entry(key) {
					Vacant(vacant) => {
						vacant.insert(value);
						Some(())
					},
					Occupied(occupied) => {
						if *occupied.get() == value {
							Some(())
						} else {
							None
						}
					},
				}
			},
			AnyFunctionToAnyFunction { key, value } => { 
				match self.map_any_function_to_any_function.entry(key) {
					Vacant(vacant) => {
						vacant.insert(value);
						Some(())
					},
					Occupied(occupied) => {
						if *occupied.get() == value {
							Some(())
						} else {
							None
						}
					},
				}
			},
		}
	}
}

fn constrained_pattern_check(expr: Expression, name: &str) -> Option<Expression> {
	use ExpressionMeta::*;

	match &name[..] {
		"n" => match expr.0 {
			IntegerValue { value: _ } => Some(expr),
			_ => None
		},
		_ => None,
	}
}

// TODO сделать чтобы возвращалось в каких позициях не сматчилось, и чтобы это делалось параллельно, чтобы было много ошибок, и все их можно было визуализировать
pub fn find_bindings(expr: Expression, by: Expression, binding_storage: &mut BindingStorage) -> Option<()> {
	use ExpressionMeta::*;

	match by.0 {
		Pattern { name: from_name } => {
			binding_storage.add(Binding::for_pattern(from_name, expr.clone()))
		},
		ConstrainedPattern { name: from_name, constrained_pattern_name } => {
			let to_matched = constrained_pattern_check(expr, &constrained_pattern_name)?;
			binding_storage.add(Binding::for_constrained_pattern(from_name, to_matched))
		},
		AnyFunction { name: from_any_function_name, args: args_from } => {
			match expr.0 {
				AnyFunction { name: to_any_function_name, args: args_to } 
					if args_from.len() == args_to.len()
				=> {
					binding_storage.add(Binding::for_any_function_to_any_function(from_any_function_name, to_any_function_name))?;
					for (arg_to, arg_from) in args_to.into_iter().zip(args_from.into_iter()) {
						find_bindings(arg_to, arg_from, binding_storage)?;
					}
					Some(())
				},
				NamedFunction { name: name_expr, args: args_expr } 
					if args_from.len() == args_expr.len()
				=> {
					binding_storage.add(Binding::for_any_function_to_named_function(from_any_function_name, name_expr))?;
					for (arg_expr, arg_by) in args_expr.into_iter().zip(args_from.into_iter()) {
						find_bindings(arg_expr, arg_by, binding_storage)?;
					}
					Some(())
				},
				_ => None,
			}
		},
		NamedFunction { name, args } => {
			match expr.0 {
				NamedFunction { name: name_expr, args: args_expr } 
					if name == name_expr && args.len() == args_expr.len()
				=> {
					for (arg_expr, arg_by) in args_expr.into_iter().zip(args.into_iter()) {
						find_bindings(arg_expr, arg_by, binding_storage)?;
					}
					Some(())
				},
				_ => None,
			}
		},
		NamedValue { name } => {
			match expr.0 {
				NamedValue { name: expr_name } if name == expr_name => Some(()),
				_ => None,
			}
		},
		IntegerValue { value } => {
			match expr.0 {
				IntegerValue { value: expr_value } if value == expr_value => Some(()),
				_ => None,
			}
		},
	}
}

pub fn apply_bindings(expr: Expression, binding_storage: &BindingStorage) -> Option<Expression> {
	use ExpressionMeta::*;

	match expr.0 {
		Pattern { name } => {
			Some(binding_storage.map_pattern.get(&name)?.clone())
		},
		ConstrainedPattern { name, constrained_pattern_name: _ } => {
			Some(binding_storage.map_constrained_pattern.get(&name)?.clone())
		},
		AnyFunction { name, args } => {
			// TODO объединить эти два
			if let Some(found) = binding_storage.map_any_function_to_any_function.get(&name) {
				let args = args.into_iter().map(|arg| apply_bindings(arg, binding_storage)).collect::<Option<Vec<_>>>()?;
				Some(Expression(AnyFunction { name: found.to_string(), args }))
			} else if let Some(found) = binding_storage.map_any_function_to_named_function.get(&name) {
				let args = args.into_iter().map(|arg| apply_bindings(arg, binding_storage)).collect::<Option<Vec<_>>>()?;
				Some(Expression(NamedFunction { name: found.to_string(), args }))
			} else {
				None
			}
		},
		NamedFunction { name, args } => {
			let args = args.into_iter().map(|arg| apply_bindings(arg, binding_storage)).collect::<Option<Vec<_>>>()?;
			Some(Expression(NamedFunction { name, args }))
		},
		NamedValue { name } => Some(Expression(NamedValue { name })),
		IntegerValue { value } => Some(Expression(IntegerValue { value })),
	}
}

pub struct FormulaPosition {
	pub module_name: String,
	pub position: usize,
	pub left_to_right: bool,
}

pub struct ExprPositionOwned(Vec<usize>);
pub struct ExprPosition([usize]);

// TODO impl Borrow<ExprPosition> for ExprPositionOwned

pub struct ProofStep {
	current_expression: Expression,
	position: ExprPositionOwned,
	used_formula: FormulaPosition,
	bindings: Vec<Binding>,
	// ??? for $f(x) = a*x, таких может быть несколько, и они могут применяться рекурсивно
}

pub struct Proof {
	steps: Vec<ProofStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionParsing {
	span: Range<usize>,
	node: ExpressionMeta<ExpressionParsing>,
}

impl GetInnerExpression for ExpressionParsing {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.node }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.node }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.node }
}

// TODO должно возвращать позицию, чтобы потом по ^^^ можно было искать то что надо
// TODO переделать на собственный алгоритм precedence!(), равенства должны парситься обычно со стороны парсера, это уже потом проверка типов должна говорить что типы не совпали
peg::parser!(
	grammar parsing() for str {
		pub rule formulas() -> Vec<Formula> 
			= r:(t:formula() _ ";" _ {t})+ { r }

		pub rule formula() -> Formula
			= left:expr() _ "<->" _ right:expr() {
				Formula { 
					left: clear_parsing_info(left), 
					right: clear_parsing_info(right) 
				}
			}

		pub rule binding() -> Binding
			= name:identifier() _ ":=" _ to:expr() { Binding::for_pattern(name, clear_parsing_info(to)) }
			/ name:identifier() "#" constrained_pattern_name:identifier() _ ":=" _ to:expr() {? || -> Result<_, _> {
				let to = clear_parsing_info(to);
				let to_matched = constrained_pattern_check(to, &constrained_pattern_name)
					.ok_or("This not fit to constrain")?;
				Ok(Binding::for_constrained_pattern(name, to_matched))
			}() }
			// TODO add function binding, but it requires matching to many things

		pub rule formula_position() -> FormulaPosition
			= module_name:identifier() "." position:integer() left_to_right:("r" { false } / "l" { true }) {
				FormulaPosition {
					module_name,
					position: position as usize,
					left_to_right,
				}
			}

		pub rule visual_positon() -> Range<CharIndex>
			= before:$("." " "*) position:$("^"+) { CharIndex(before.len())..CharIndex(before.len() + position.len()) }
			/ position:$("^"+) { CharIndex(0)..CharIndex(position.len()) }

		pub rule expr() -> ExpressionParsing
			= or()

		rule or() -> ExpressionParsing
			= start:position!() 
			  l:and() 
			  r:(_ z:$("|") _ p:or() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsing {
						span: start..end,
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule and() -> ExpressionParsing
			= start:position!() 
			  l:equality() 
			  r:(_ z:$("&") _ p:and() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsing {
						span: start..end,
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule equality() -> ExpressionParsing
			= start:position!() 
			  l:sum() 
			  r:(_ z:$("="/"!="/">"/"<"/">="/"<=") _ p:sum() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsing {
						span: start..end,
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule sum() -> ExpressionParsing
			= start:position!() 
			  l:product() 
			  r:(_ z:$("+"/"-") _ p:sum() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsing {
						span: start..end,
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}
			/ start:position!() "-" _ l:product() end:position!() {
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::NamedFunction { name: "negative".to_string(), args: vec![l] }
				}
			}

		rule product() -> ExpressionParsing
			= start:position!() 
			  l:power() 
			  r:(_ z:$("*"/"/") _ p:product() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsing {
						span: start..end,
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule power() -> ExpressionParsing
			= start:position!() 
			  l:atom()
			  r:(_ z:$("^") _ p:power() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => match p.node {
						ExpressionMeta::NamedFunction { name, mut args } if name == z && args.len() == 2 => {
							let c = args.pop().unwrap();
							let b = args.pop().unwrap();
							let a = l;

							let a_pos = start;
							let b_pos = p.span.start;
							let c_pos = p.span.end;

							let l = ExpressionParsing {
								span: a_pos..b_pos,
								node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![a, b] }
							};

							ExpressionParsing {
								span: b_pos..c_pos,
								node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, c] }
							}
						},
						other => {
							let p = ExpressionParsing { span: p.span, node: other };
							ExpressionParsing {
								span: start..end,
								node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
							}
						},
					},
					None => l,
				}
			}

		rule atom() -> ExpressionParsing
			= "(" v:expr() ")" { v }

			/ any_function()
			/ named_value()

			/ function()
			/ constrained_pattern()
			/ pattern()

			/ integer_value()

		rule pattern() -> ExpressionParsing
			= start:position!() name:identifier() end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::Pattern { name } 
				}
			}

		rule constrained_pattern() -> ExpressionParsing
			= start:position!() name:identifier() "#" constrained_pattern_name:identifier() end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::ConstrainedPattern { name, constrained_pattern_name }
				}
			}

		rule function() -> ExpressionParsing
			= start:position!() name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::NamedFunction { name, args }
				}
			}

		rule any_function() -> ExpressionParsing
			= start:position!() "$" name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::AnyFunction { name, args }
				}
			}

		rule integer_value() -> ExpressionParsing
			= start:position!() value:integer() end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::IntegerValue { value: value as i64 }
				}
			}

		rule named_value() -> ExpressionParsing
			= start:position!() "$" name:identifier() end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::NamedValue { name }
				}
			}

		rule integer() -> u64
			= n:$(['0'..='9']+) {? n.parse().map_err(|_| "number is too big") }

		rule identifier() -> String 
			= n:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) {
				String::from(n)
			}

		rule _() = quiet!{[' ' | '\n' | '\t']*}
	}
);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CharIndex(usize);
pub fn get_char_range(s: &str, range: Range<usize>) -> Option<Range<CharIndex>> {
	let mut iter = s
		.char_indices()
		.enumerate()
		.map(|(char_position, (index, _))| (CharIndex(char_position), index))

		// TODO сделать чтобы были ленивые вычисления
		.chain(std::iter::once((CharIndex(s.chars().count()), s.len())));
	let start = iter.find(|(_, index)| *index == range.start)?.0;
	let end = iter.find(|(_, index)| *index == range.end)?.0;
	Some(start..end)
}

fn clear_parsing_info(expr: ExpressionParsing) -> Expression {
	use ExpressionMeta::*;

	Expression(
		match expr.node {
			AnyFunction { name, args } => 
				AnyFunction { name, args: args.into_iter().map(clear_parsing_info).collect() },
			NamedFunction { name, args } =>
				NamedFunction { name, args: args.into_iter().map(clear_parsing_info).collect() },
			Pattern { name } => 
				Pattern { name },
			ConstrainedPattern { name, constrained_pattern_name } => 
				ConstrainedPattern { name, constrained_pattern_name },
			NamedValue { name } => 
				NamedValue { name },
			IntegerValue { value } => 
				IntegerValue { value },
		}
	)
}

fn process_expression_parsing(expr: ExpressionParsing) -> (Expression, Vec<(Vec<usize>, Range<usize>)>) {
	fn process(
		expr: ExpressionMeta<ExpressionParsing>, 
		current_position: &mut Vec<usize>, 
		storage: &mut Vec<(Vec<usize>, Range<usize>)>
	) -> Expression {
		use ExpressionMeta::*;

		let mut process_args = |args: Vec<ExpressionParsing>| {
			args.into_iter().enumerate().map(|(pos, arg)| {
				current_position.push(pos);
				let ExpressionParsing { span, node } = arg;
				storage.push((current_position.clone(), span));
				let result = process(node, current_position, storage);
				current_position.pop().unwrap();
				result
			}).collect()
		};

		Expression(
			match expr {
				AnyFunction { name, args } => 
					AnyFunction { 
						name, 
						args: process_args(args),
					},
				NamedFunction { name, args } =>
					NamedFunction { 
						name, 
						args: process_args(args),
					},
				Pattern { name } => 
					Pattern { name },
				ConstrainedPattern { name, constrained_pattern_name } => 
					ConstrainedPattern { name, constrained_pattern_name },
				NamedValue { name } => 
					NamedValue { name },
				IntegerValue { value } => 
					IntegerValue { value },
			}
		)
	}

	let mut storage = Vec::new();
	let mut current_position = Vec::new();
	let ExpressionParsing { span, node } = expr;
	storage.push((current_position.clone(), span));
	(process(node, &mut current_position, &mut storage), storage)
}

// TODO сделать тут Result, который возвращает позицию
fn get<'a, Expr: GetInnerExpression>(expr: &'a Expr, position: &[usize]) 
	-> Option<&'a Expr>
{
	use ExpressionMeta::*;

	match position {
		[start, tail @ ..] => {
			match expr.get_inner_expression_ref() {
				AnyFunction { name: _, args } |
				NamedFunction { name: _, args } => get(args.get(*start)?, tail),

				Pattern { .. } |
				ConstrainedPattern { .. } |
				NamedValue { .. } |
				IntegerValue { .. } => None,
			}
		},
		[] => Some(expr),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test() {
		assert!(parsing::expr("(a+b)+c").is_ok());
		assert!(parsing::expr("a+b+c").is_ok());
		assert!(parsing::expr("a+sin(b)").is_ok());
		assert!(parsing::expr("a+$f(b)").is_ok());
		assert!(parsing::expr("$true+$f(1)-a#n").is_ok());

		let expression = parsing::expr("part(b = 0, a, a*part($true, 1, $undefined))").unwrap();
		let expression = clear_parsing_info(expression);
		let formula = parsing::formula("part(cond, then, else) <-> part(not(cond), else, then)").unwrap();

		let mut bindings = BindingStorage::default();

		find_bindings(expression, formula.left, &mut bindings).unwrap();
		let result = apply_bindings(formula.right, &bindings).unwrap();

		let should_be = parsing::expr("part(not(b = 0), a*part($true, 1, $undefined), a)").unwrap();
		let should_be = clear_parsing_info(should_be);

		assert_eq!(result, should_be);
	}

	#[test]
	fn test2() {
		let expression = parsing::expr("a").unwrap();
		let expression = clear_parsing_info(expression);
		let formula = parsing::formula("part(x, a, a) <-> a").unwrap();

		let mut bindings = BindingStorage::default();

		bindings.add(parsing::binding("x := b = 0").unwrap());

		find_bindings(expression, formula.right, &mut bindings).unwrap();
		let result = apply_bindings(formula.left, &bindings).unwrap();

		let should_be = parsing::expr("part(b = 0, a, a)").unwrap();
		let should_be = clear_parsing_info(should_be);

		assert_eq!(result, should_be);
	}

	macro_rules! same {
		($a:expr, $b:expr) => {
			assert_eq!(
				clear_parsing_info(parsing::expr($a).unwrap()), 
				clear_parsing_info(parsing::expr($b).unwrap())
			);
		};
	}

	#[test]
	fn associativity() {
		same!("a+b+c", "a+(b+c)");
		same!("a+b*c", "a+(b*c)");
		same!("a*b+c", "(a*b)+c");
		same!("a*b*c", "a*(b*c)");
		same!("a^b^c", "(a^b)^c");
	}

	#[test]
	fn priority() {
		same!("a*b+c*d", "(a*b)+(c*d)");
		same!("a^b*c^d", "(a^b)*(c^d)");
	}

	#[test]
	fn parsing_info() {
		macro_rules! debug_unwrap {
			($name:ident( $($arg:expr),* )) => {{
				let mut debug_string = String::from(stringify!($name));
				debug_string.push_str("(");
				$(
					debug_string.push_str(concat!(stringify!($arg), " = "));
					debug_string.push_str(format!("{:?}", $arg).as_ref());
					debug_string.push_str(", ");
				)*
				if debug_string.ends_with(", ") {
					debug_string.pop();
					debug_string.pop();
				}
				debug_string.push_str(")");

				$name($($arg),*).unwrap_or_else(|| panic!(debug_string))
			}};
		}

		//let string = "part(b =      0, a, a *part($true, 1, $undefined))";
		let string = "a+b+c+d^f*e";
		let parsed = parsing::expr(string).unwrap();
		let (_, positions) = process_expression_parsing(parsed);
		let positions: Vec<_> = positions
			.into_iter()
			.map(|(pos, range)| {
				let new_range = debug_unwrap!(get_char_range(string, range));
				(pos, new_range)
			})
			.collect();

		for (pos, range) in positions {
			use std::iter;
			println!(
				"{eq}\n{spaces_before}{arrows}{spaces_after} - {position:?}",
				eq = string,
				spaces_before = iter::repeat(' ').take(range.start.0).collect::<String>(),
				arrows = iter::repeat('^').take(range.end.0 - range.start.0).collect::<String>(),
				spaces_after = iter::repeat(' ').take(string.len() - range.end.0).collect::<String>(),
				position = pos,
			)
		}
	}
}
