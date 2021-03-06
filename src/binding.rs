use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
	expr::*,
	utils::{apply::*, joined_by::*},
};

/// Одна часть в формуле `formula_part <-> ...`.
#[derive(Clone, Debug)]
pub struct FormulaPart {
	pub pattern: Expression,
	pub unknown_patterns_names: Vec<String>,
	pub anyfunction_names: Vec<(String, usize)>,
}

#[derive(Debug, Error)]
pub enum FormulaError {
	#[error(
		"wrong count of arguments in anyfunctions: in function `{name}`, should be {should_be}, but {actual} is presented"
	)]
	WrongCountOfArgumentsInAnyFunctions {
		name: String,
		should_be: usize,
		actual: usize,
	},
	#[error("not equal anyfunctions in both sides of equation: only left side has this [{}], and only right side has this [{}]", left_side.iter().joined_by(", "), right_side.iter().joined_by(", "))]
	NotEqualAnyfunctions {
		left_side: Vec<AnyFunctionNames>,
		right_side: Vec<AnyFunctionNames>,
	},
}

impl Formula {
	pub fn new(left: Expression, right: Expression) -> Result<Formula, FormulaError> {
		let left_patterns = left.get_pattern_names();
		let right_patterns = right.get_pattern_names();

		let left_unknown_patterns: Vec<String> = right_patterns.difference(&left_patterns).cloned().collect();
		let right_unknown_patterns: Vec<_> = left_patterns.difference(&right_patterns).cloned().collect();

		let left_anyfunctions = left.get_anyfunction_names();
		let right_anyfunctions = right.get_anyfunction_names();

		let check_functions_equal_arguments = |functions| -> Result<_, FormulaError> {
			let mut arg_count: BTreeMap<String, usize> = BTreeMap::new();
			for AnyFunctionNames { name, arguments_count } in functions {
				use std::collections::btree_map::Entry::*;

				match arg_count.entry(name.clone()) {
					Vacant(vacant) => {
						vacant.insert(arguments_count);
					},
					Occupied(occupied) => {
						if *occupied.get() != arguments_count {
							return Err(FormulaError::WrongCountOfArgumentsInAnyFunctions {
								name: name.to_string(),
								should_be: *occupied.get(),
								actual: arguments_count,
							});
						}
					},
				}
			}
			Ok(arg_count)
		};

		// Проверяем, что везде совпадает число аргументов
		let left_anyfunctions = check_functions_equal_arguments(left_anyfunctions)?;
		let right_anyfunctions = check_functions_equal_arguments(right_anyfunctions)?;

		// Проверяем с обоих сторон одинаковые имена
		if left_anyfunctions != right_anyfunctions {
			let left_anyfunctions = left_anyfunctions
				.into_iter()
				.map(|x| AnyFunctionNames { name: x.0, arguments_count: x.1 })
				.collect::<BTreeSet<_>>();
			let right_anyfunctions = right_anyfunctions
				.into_iter()
				.map(|x| AnyFunctionNames { name: x.0, arguments_count: x.1 })
				.collect::<BTreeSet<_>>();
			return Err(FormulaError::NotEqualAnyfunctions {
				left_side: left_anyfunctions.difference(&right_anyfunctions).cloned().collect(),
				right_side: right_anyfunctions.difference(&left_anyfunctions).cloned().collect(),
			});
		}

		Ok(Formula {
			left: FormulaPart {
				pattern: left,
				unknown_patterns_names: left_unknown_patterns,
				anyfunction_names: left_anyfunctions.into_iter().collect(),
			},
			right: FormulaPart {
				pattern: right,
				unknown_patterns_names: right_unknown_patterns,
				anyfunction_names: right_anyfunctions.into_iter().collect(),
			},
		})
	}
}

/// `left <-> right`
#[derive(Clone, Debug)]
pub struct Formula {
	pub left: FormulaPart,
	pub right: FormulaPart,
}

/// `variable -> value`, позволяет производить замену с имени паттерна на выражение
#[derive(Clone, Debug)]
pub struct Binding {
	pub pattern_name: String,
	pub to_value: Expression,
}

impl Binding {
	pub fn new(pattern_name: String, to_value: Expression) -> Binding { Binding { pattern_name, to_value } }
}

#[derive(Default, Debug, Clone)]
pub struct BindingStorage(BTreeMap<String, Expression>);

impl BindingStorage {
	/// Добавляет биндинг в хранилище. Если он уже существует, то проверяет что они совпадают. Если это не так, возвращает None.
	pub fn add(&mut self, binding: Binding) -> Option<()> {
		use std::collections::btree_map::Entry::*;

		match self.0.entry(binding.pattern_name) {
			Vacant(vacant) => {
				vacant.insert(binding.to_value);
				Some(())
			},
			Occupied(occupied) => {
				if *occupied.get() == binding.to_value {
					Some(())
				} else {
					None
				}
			},
		}
	}
}

pub trait AnyFunctionBinding {
	fn find_bindings(
		&mut self,
		any_function_name: &str,
		args: &[Expression],
		expr: Expression,
		binding_storage: &mut BindingStorage,
	) -> Option<()>;

	fn apply_bindings(
		&self,
		any_function_name: &str,
		args: Vec<Expression>,
		binding_storage: &BindingStorage,
	) -> Option<Expression>;
}

pub fn find_bindings<A: AnyFunctionBinding>(
	expr: Expression,
	by: &Expression,
	binding_storage: &mut BindingStorage,
	any_function_binding: &mut A,
) -> Option<()> {
	use ExpressionMeta::*;

	match &by.0 {
		Pattern { name } => binding_storage.add(Binding::new(name.to_string(), expr)),
		AnyFunction { name, args } => any_function_binding.find_bindings(&name, &args, expr, binding_storage),
		NamedFunction { name, args } => match expr.0 {
			NamedFunction { name: name_expr, args: args_expr }
				if *name == name_expr && args.len() == args_expr.len() =>
			{
				for (arg_expr, arg_by) in args_expr.into_iter().zip(args.iter()) {
					find_bindings(arg_expr, &arg_by, binding_storage, any_function_binding)?;
				}
				Some(())
			}
			_ => None,
		},
		NamedValue { name } => match expr.0 {
			NamedValue { name: expr_name } if *name == expr_name => Some(()),
			_ => None,
		},
		IntegerValue { value } => match expr.0 {
			IntegerValue { value: expr_value } if *value == expr_value => Some(()),
			_ => None,
		},
	}
}

pub fn apply_bindings<A: AnyFunctionBinding>(
	expr: Expression,
	binding_storage: &BindingStorage,
	any_function_binding: &A,
) -> Expression {
	use ExpressionMeta::*;

	match expr.0 {
		Pattern { name } => {
			if let Some(found) = binding_storage.0.get(&name) {
				found.clone()
			} else {
				Pattern { name }.apply(Expression)
			}
		},
		AnyFunction { name, args } => any_function_binding
			.apply_bindings(&name, args, binding_storage)
			.unwrap(),
		NamedFunction { name, args } => NamedFunction {
			name,
			args: args
				.into_iter()
				.map(|arg| apply_bindings(arg, binding_storage, any_function_binding))
				.collect(),
		}
		.apply(Expression),
		NamedValue { name } => NamedValue { name }.apply(Expression),
		IntegerValue { value } => IntegerValue { value }.apply(Expression),
	}
}

/// `$f(..variables) := pattern`
#[derive(Clone, Debug)]
pub struct AnyFunctionPattern {
	pub pattern: Expression,
	pub variables: Vec<String>,
}

/// Позволяет матчиться с `AnyFunction` путём ручного задания паттерна который там должен находиться.
pub struct ManualAnyFunctionBinding {
	to_match: BTreeMap<String, AnyFunctionPattern>,
	bindings: BTreeMap<String, BindingStorage>,
}

impl ManualAnyFunctionBinding {
	pub fn new(to_match: BTreeMap<String, AnyFunctionPattern>) -> Self {
		Self { to_match, bindings: BTreeMap::default() }
	}
}

impl AnyFunctionBinding for ManualAnyFunctionBinding {
	fn find_bindings(
		&mut self,
		any_function_name: &str,
		args: &[Expression],
		expr: Expression,
		global_bindings: &mut BindingStorage,
	) -> Option<()> {
		let AnyFunctionPattern { pattern, variables } = self.to_match.get(any_function_name)?.clone();
		let mut local_bingings = BindingStorage::default();
		crate::binding::find_bindings(expr, &pattern, &mut local_bingings, self)?;

		if variables.len() != args.len() {
			return None;
		}
		for (name, arg) in variables.iter().zip(args.iter()) {
			let binding = local_bingings.0.remove(name)?;
			crate::binding::find_bindings(binding, &arg, global_bindings, self)?;
		}

		self.bindings.insert(any_function_name.to_string(), local_bingings);

		Some(())
	}

	fn apply_bindings(
		&self,
		any_function_name: &str,
		args: Vec<Expression>,
		global_bindings: &BindingStorage,
	) -> Option<Expression> {
		let AnyFunctionPattern { pattern, variables } = self.to_match.get(any_function_name)?.clone();
		let mut local_bindings = self.bindings.get(any_function_name)?.clone();

		for (name, arg) in variables.into_iter().zip(args.into_iter()) {
			local_bindings.add(Binding::new(name, apply_bindings(arg, global_bindings, self)));
		}

		Some(apply_bindings(pattern, &local_bindings, self))
	}
}
