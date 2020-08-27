/// Выражение
#[derive(Clone, Debug)]
enum Expression {
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
		args: Vec<Expression>,
	},

	/// `a+b`, `sin(1)` - функция с именем и определённым набором аргументов
	NamedFunction {
		name: String,
		args: Vec<Expression>,
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

/// `left <-> right`
#[derive(Clone, Debug)]
struct Formula {
	left: Expression,
	right: Expression,
}

/// `variable -> value`, позволяет производить замену с имени паттерна на выражение
// TODO подумать, надо ли разбивать это на key, value
#[derive(Clone, Debug)]
enum Binding {
	Pattern {
		from_name: String,
		to_value: Expression,
	},

	ConstrainedPattern {
		from_name: String,
		to_value: Expression,
	},

	AnyFunctionToNamedFunction {
		from_any_function_name: String,
		to_named_function_name: String,
	},

	AnyFunctionToAnyFunction {
		from_any_function_name: String,
		to_any_function_name: String,
	},
}

impl Binding {
	fn for_pattern(from_name: String, to_value: Expression) -> Binding {
		unimplemented!()
	}

	fn for_constrained_pattern(from_name: String, to_value: Expression) -> Binding {
		unimplemented!()
	}

	fn for_any_function_to_named_function(from_any_function_name: String, to_named_function_name: String) -> Binding {
		unimplemented!()
	}

	fn for_any_function_to_any_function(from_any_function_name: String, to_any_function_name: String) -> Binding {
		unimplemented!()
	}
}

struct BindingStorage {
}

impl BindingStorage {
	/// Добавляет привязку в хранилище. Если такая привязка уже существует, то проверяет что они совпадают. Если это не так, возвращает None.
	fn add(&mut self, binding: Binding) -> Option<()> {
		unimplemented!()	
	}
}

fn constrained_pattern_check(expr: &Expression, name: &str) -> Option<Expression> {
	use Expression::*;

	match &name[..] {
		"n" => match expr {
			IntegerValue { value: _ } => Some(expr.clone()),
			_ => None
		},
		_ => None,
	}
}

fn find_bindings(expr: &Expression, by: &Expression, binding_storage: &mut BindingStorage) -> Option<()> {
	use Expression::*;

	match by {
		Pattern { name: from_name } => {
			binding_storage.add(Binding::for_pattern(from_name.clone(), expr.clone()))
		},
		ConstrainedPattern { name: from_name, constrained_pattern_name } => {
			let to_matched = constrained_pattern_check(expr, constrained_pattern_name)?;
			binding_storage.add(Binding::for_constrained_pattern(from_name.clone(), to_matched))
		},
		AnyFunction { name: from_any_function_name, args: args_from } => {
			match expr {
				AnyFunction { name: to_any_function_name, args: args_to } 
					if args_from.len() == args_to.len()
				=> {
					binding_storage.add(Binding::for_any_function_to_any_function(from_any_function_name.clone(), to_any_function_name.clone()))?;
					for (arg_to, arg_from) in args_to.iter().zip(args_from.iter()) {
						find_bindings(arg_to, arg_from, binding_storage)?;
					}
					Some(())
				},
				NamedFunction { name: name_expr, args: args_expr } 
					if args_from.len() == args_expr.len()
				=> {
					binding_storage.add(Binding::for_any_function_to_named_function(from_any_function_name.clone(), name_expr.clone()))?;
					for (arg_expr, arg_by) in args_expr.iter().zip(args_from.iter()) {
						find_bindings(arg_expr, arg_by, binding_storage)?;
					}
					Some(())
				},
				_ => None,
			}
		},
		NamedFunction { name, args } => {
			match expr {
				NamedFunction { name: name_expr, args: args_expr } 
					if name == name_expr && args.len() == args_expr.len()
				=> {
					for (arg_expr, arg_by) in args_expr.iter().zip(args.iter()) {
						find_bindings(arg_expr, arg_by, binding_storage)?;
					}
					Some(())
				},
				_ => None,
			}
		},
		NamedValue { name } => {
			match expr {
				NamedValue { name: expr_name } if name == expr_name => Some(()),
				_ => None,
			}
		},
		IntegerValue { value } => {
			match expr {
				IntegerValue { value: expr_value } if value == expr_value => Some(()),
				_ => None,
			}
		},
	}
}

// fn apply_bindings()

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