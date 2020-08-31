use crate::expr::*;
use std::collections::BTreeMap;

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
	pub fn for_pattern(from_name: String, to_value: Expression) -> Binding {
		Binding::Pattern {
			key: from_name,
			value: to_value,
		}
	}

	pub fn for_constrained_pattern(from_name: String, to_value: Expression) -> Binding {
		Binding::ConstrainedPattern {
			key: from_name,
			value: to_value,
		}
	}

	pub fn for_any_function_to_named_function(from_any_function_name: String, to_named_function_name: String) -> Binding {
		Binding::AnyFunctionToNamedFunction {
			key: from_any_function_name,
			value: to_named_function_name,
		}
	}

	pub fn for_any_function_to_any_function(from_any_function_name: String, to_any_function_name: String) -> Binding {
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

pub fn constrained_pattern_check(expr: Expression, name: &str) -> Option<Expression> {
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
