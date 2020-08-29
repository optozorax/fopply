use std::ops::Range;
use std::fmt::Debug;
use std::collections::BTreeMap;

/// Обобщённое выражение, нужно для возможности делать парсинг
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionMeta<ArgType: Clone + Debug + Eq + PartialEq> {
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
pub struct ExprPosition<'a>(&'a [usize]);

pub struct ProofStep {
	position: ExprPositionOwned,
	used_formula: FormulaPosition,
	bindings: Vec<Binding>,
	// ??? for $f(x) = a*x
}

pub struct Proof {
	start: Expression,
	steps: Vec<ProofStep>,
	end: Expression,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpressionParsing {
	span: Range<usize>,
	node: ExpressionMeta<ExpressionParsing>,
}

// TODO должно возвращать позицию, чтобы потом по ^^^ можно было искать то что надо
peg::parser!(
	grammar parsing() for str {
		pub rule formulas() -> Vec<Formula> 
			= r:(t:formula() _ ";" _ {t})+ { r }

		pub rule formula() -> Formula
			= left:expr() _ "<->" _ right:expr() { Formula { left, right } }

		pub rule expr() -> Expression = precedence! {
			x:(@) _ "|" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "|".to_string(), args: vec![x, y] }) }
			--
			x:(@) _ "&" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "&".to_string(), args: vec![x, y] }) }
			--
			e:equality() {e}
		}

		pub rule binding() -> Binding
			= name:identifier() _ ":=" _ to:expr() { Binding::for_pattern(name, to) }
			/ name:identifier() "#" constrained_pattern_name:identifier() _ ":=" _ to:expr() {? || -> Result<_, _> {
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

		rule equality() -> Expression
			= l:sumproduct() r:(_ z:$("="/"!="/">"/"<"/">="/"<=") _ p:sumproduct() { (z, p) })? {
				match r {
					Some((z, p)) => Expression(ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }),
					None => l,
				}
			}

		rule sumproduct() -> Expression = precedence! {
			x:(@) _ "+" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "+".to_string(), args: vec![x, y] }) }
			x:(@) _ "-" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "-".to_string(), args: vec![x, y] }) }
					"-" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "negative".to_string(), args: vec![y] }) }
			--
			x:(@) _ "*" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "*".to_string(), args: vec![x, y] }) }
			x:(@) _ "/" _ y:@ { Expression(ExpressionMeta::NamedFunction { name: "/".to_string(), args: vec![x, y] }) }
			--
			x:@ _ "^" _ y:(@) { Expression(ExpressionMeta::NamedFunction { name: "^".to_string(), args: vec![x, y] }) }
			--
			a:atom() { a }
		}

		rule atom() -> Expression
			= "(" v:expr() ")" { v }

			/ any_function()
			/ named_value()

			/ function()
			/ constrained_pattern()
			/ pattern()

			/ integer_value()

		rule pattern() -> Expression
			= name:identifier() { Expression(ExpressionMeta::Pattern { name }) }

		rule constrained_pattern() -> Expression
			= name:identifier() "#" constrained_pattern_name:identifier() { 
				Expression(ExpressionMeta::ConstrainedPattern { name, constrained_pattern_name })
			}

		rule function() -> Expression
			= name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" { 
				Expression(ExpressionMeta::NamedFunction { name, args })
			}

		rule any_function() -> Expression
			= "$" name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" { 
				Expression(ExpressionMeta::AnyFunction { name, args })
			}

		rule integer_value() -> Expression
			= value:integer() { Expression(ExpressionMeta::IntegerValue { value: value as i64 }) }

		rule named_value() -> Expression
			= start:position!() "$" name:identifier() end:position!() { Expression(ExpressionMeta::NamedValue { name }) }

		rule integer() -> u64
			= n:$(['0'..='9']+) {? n.parse().map_err(|_| "number is too big") }

		rule identifier() -> String 
			= n:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '_' | '0'..='9']*) {
				String::from(n)
			}

		rule _() = quiet!{[' ' | '\n' | '\t']*}
	}
);

fn process_expression_parsing(expr: ExpressionParsing) -> (Expression, Vec<(ExprPositionOwned, Range<usize>)>) {
	unimplemented!()
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
		let formula = parsing::formula("part(cond, then, else) <-> part(not(cond), else, then)").unwrap();

		let mut bindings = BindingStorage::default();

		find_bindings(expression, formula.left, &mut bindings).unwrap();
		let result = apply_bindings(formula.right, &bindings).unwrap();

		let should_be = parsing::expr("part(not(b = 0), a*part($true, 1, $undefined), a)").unwrap();

		assert_eq!(result, should_be);
	}

	#[test]
	fn test2() {
		let expression = parsing::expr("a").unwrap();
		let formula = parsing::formula("part(x, a, a) <-> a").unwrap();

		let mut bindings = BindingStorage::default();

		bindings.add(parsing::binding("x := b = 0").unwrap());

		find_bindings(expression, formula.right, &mut bindings).unwrap();
		let result = apply_bindings(formula.left, &bindings).unwrap();

		let should_be = parsing::expr("part(b = 0, a, a)").unwrap();

		assert_eq!(result, should_be);
	}
}

/*

```rust
struct BindingSetter(...);
impl BindingSetter {
	/// Если уже присутствует, то сравнивает равны ли они, если не равны, то отвергает
	fn add(&mut self, binding: Binding) -> Option<()> {
		...
	}

	fn to_getter(self) -> BindingGetter {
		...
	}
}

struct BindingGetter(...);
impl BindingGetter {
	fn get(name: String /* For function? */) -> Equation {
		...
	}
}

fn find_bindings(where: &Expression, by: &Expression) -> Option<BindingSetter> {
	unimplemented!()
}

fn apply_bindings(to: &Formula, bindings: &BindingsGetter) -> Option<Equation> {
	unimplemented!()
}

???
fn find_equation(&mut Equation, position: Vec<usize>) -> Option<&mut Equation> {

}

fn apply_formula(Equation, FormulaTree, position) -> Option<Equation> {
	
}

a/b/a <-> 1/b
5/2/5 -> Some(a = 5, b = 2, a = 5) -> Some(1/b)
5/2/3 -> Some(a = 5, b = 2, a = 3) -> None
```

*/