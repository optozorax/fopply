use std::collections::BTreeSet;
use std::borrow::Borrow;
use crate::utils::apply::*;

/// Обобщённое выражение. Обобщённость нужна для возможности как задать положения в парсинге, так и для возмоности задания обычного выражения. Была выбрана такая обобщённость вместо копипасты данной структуры отдельно для парсинга.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ExpressionMeta<Arg> {
	/// В математике называется "переменной", но здесь это называется паттерном. Матчится с чем угодно, именованная часть выражения. В выражении выглядит как: `a`, `b`, `c`.
	// TODO переименовать в Any
	Pattern {
		name: String
	},

	/// Любая функция с неизвестным именем с конкретным числом аргументов. В выражении выглядит как: `$f(a, b)`.
	AnyFunction {
		name: String,
		args: Vec<Arg>,
	},

	/// Функция с именем и определённым набором аргументов. В выражении выглядит как: `a+b`, `sin(1)`.
	NamedFunction {
		name: String,
		args: Vec<Arg>,
	},

	/// Именованная константа. В выражении выглядит как: `$false`, `$true`, `$i`, `$undefined`.
	NamedValue {
		name: String
	},

	/// Числовая константа. В выражении выглядит как: `1`, `1000`.
	IntegerValue {
		value: i64
	},
}

// TODO применить где-нибудь
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub enum ExpressionKind {
	Pattern,
	AnyFunction,
	NamedFunction,
	NamedValue,
	IntegerValue,
}

impl<Arg> From<&ExpressionMeta<Arg>> for ExpressionKind {
    fn from(expr: &ExpressionMeta<Arg>) -> Self {
    	use ExpressionMeta::*;

        match expr {
			Pattern { .. } => ExpressionKind::Pattern,
			AnyFunction { .. } => ExpressionKind::AnyFunction,
			NamedFunction { .. } => ExpressionKind::NamedFunction,
			NamedValue { .. } => ExpressionKind::NamedValue,
			IntegerValue { .. } => ExpressionKind::IntegerValue,
        }
    }
}

/// Ввиду обобщённости `ExpressionMeta`, нужно как-то получать его обратно когда обращаешься к `args`, поэтому сделан такой трейт.
pub trait GetInnerExpression: Sized {
	fn get_inner_expression(self) -> ExpressionMeta<Self>;
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self>;
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self>;
}

/// Выражение без дополнительной информации.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Expression(pub ExpressionMeta<Expression>);

impl GetInnerExpression for Expression {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.0 }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.0 }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.0 }
}

/// Положение в выражении.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExprPositionOwned(pub Vec<usize>);

/// Положение в выражении для передачи в функции. Аналог `[usize]`.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct ExprPosition(pub [usize]);

impl ExprPosition {
	/// Создать ссылку на `ExprPosition` из слайса на `usize`.
	pub fn from_slice(slice: &[usize]) -> &Self {
		unsafe { &*(slice as *const [usize] as *const ExprPosition) }
	}

	/// Создать мутабельную ссылку на `ExprPosition` из слайса на `usize`.
	pub fn from_slice_mut(slice: &mut [usize]) -> &mut Self {
		unsafe { &mut *(slice as *mut [usize] as *mut ExprPosition) }
	}
}

impl Borrow<ExprPosition> for ExprPositionOwned {
	fn borrow(&self) -> &ExprPosition {
		ExprPosition::from_slice(self.0.borrow())
	}
}

/// Показывает в каком положении в массиве `ExprPosition` не было найдено то что нужно.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy)]
pub struct PositionError(usize);

impl<Arg> ExpressionExtensionInner for Arg where
	Arg: GetInnerExpression,
{
	fn get_inner<'a>(&'a self, position: &ExprPosition, deep: usize) -> Result<&'a Self, PositionError> {
		use ExpressionMeta::*;

		match &position.0 {
			[start, tail @ ..] => {
				match self.get_inner_expression_ref() {
					AnyFunction { name: _, args } |
					NamedFunction { name: _, args } => args
						.get(*start)
						.ok_or(PositionError(deep))?
						.get_inner(ExprPosition::from_slice(tail), deep+1),

					Pattern { .. } |
					NamedValue { .. } |
					IntegerValue { .. } => Err(PositionError(deep)),
				}
			},
			[] => Ok(self),
		}
	}

	fn get_mut_inner<'a>(&'a mut self, position: &ExprPosition, deep: usize) -> Result<&'a mut Self, PositionError> {
		use ExpressionMeta::*;

		match &position.0 {
			[start, tail @ ..] => {
				match self.get_inner_expression_mut() {
					AnyFunction { name: _, args } |
					NamedFunction { name: _, args } => args
						.get_mut(*start)
						.ok_or(PositionError(deep))?
						.get_mut_inner(ExprPosition::from_slice(tail), deep+1),

					Pattern { .. } |
					NamedValue { .. } |
					IntegerValue { .. } => Err(PositionError(deep)),
				}
			},
			[] => Ok(self),
		}
	}

	fn travel_positions_inner<F: FnMut(&Self, &ExprPosition)>(&self, current_position: &mut ExprPositionOwned, f: &mut F) {
		use ExpressionMeta::*;

		f(self, (&*current_position).borrow());

		let mut process_args = |args: &[Arg]| {
			args.iter().enumerate().for_each(|(pos, arg)| {
				current_position.0.push(pos);
				arg.travel_positions_inner(current_position, f);
				current_position.0.pop().unwrap();
			})
		};

		match self.get_inner_expression_ref() {
			AnyFunction { name: _, args } |
			NamedFunction { name: _, args } => process_args(&*args),

			Pattern { name: _ } |
			NamedValue { name: _ } |
			IntegerValue { value: _ } => (),
		}
	}
}

/// Внутренние методы для `ExpressionExtension`. Пользоваться этими методами не нужно. Я делаю его публичным только потому что компилятор ругается, вообще эти методы надо как-то сделать приватными. TODO.
pub trait ExpressionExtensionInner: GetInnerExpression {
	fn get_inner<'a>(&'a self, position: &ExprPosition, deep: usize) -> Result<&'a Self, PositionError>;
	fn get_mut_inner<'a>(&'a mut self, position: &ExprPosition, deep: usize) -> Result<&'a mut Self, PositionError>;
	fn travel_positions_inner<F: FnMut(&Self, &ExprPosition)>(&self, current_position: &mut ExprPositionOwned, f: &mut F);
}

pub trait ExpressionExtension: GetInnerExpression {
	fn get<'a>(&'a self, position: &ExprPosition) -> Result<&'a Self, PositionError>;
	fn get_mut<'a>(&'a mut self, position: &ExprPosition) -> Result<&'a mut Self, PositionError>;
	fn travel<F: FnMut(&Self)>(&self, f: &mut F);
	fn travel_positions<F: FnMut(&Self, &ExprPosition)>(&self, f: F);
	fn get_pattern_names(&self) -> BTreeSet<String>;
	fn get_anyfunction_names(&self) -> BTreeSet<(String, usize)>;
}

impl<Arg> ExpressionExtension for Arg where
	Arg: GetInnerExpression + ExpressionExtensionInner,
{
	/// Получить ссылку на внутреннюю часть выражения. 
	fn get<'a>(&'a self, position: &ExprPosition) -> Result<&'a Self, PositionError> {
		self.get_inner(position, 0)
	}

	/// Получить изменяемую ссылку на внутреннюю часть выражения.
	fn get_mut<'a>(&'a mut self, position: &ExprPosition) -> Result<&'a mut Self, PositionError> {
		self.get_mut_inner(position, 0)
	}

	/// Обход всего выражения с передачей позиции.
	fn travel<F: FnMut(&Self)>(&self, f: &mut F) {
		use ExpressionMeta::*;

		f(self);

		match self.get_inner_expression_ref() {
			AnyFunction { name: _, args } |
			NamedFunction { name: _, args } => args.iter().for_each(|arg| {
				arg.travel(f);
			}),

			Pattern { name: _ } |
			NamedValue { name: _ } |
			IntegerValue { value: _ } => (),
		}
	}

	/// Обход всего выражения с передачей позиции.
	fn travel_positions<F: FnMut(&Self, &ExprPosition)>(&self, mut f: F) {
		let mut current_position = Vec::new().apply(ExprPositionOwned);
		self.travel_positions_inner(&mut current_position, &mut f);
	}

	/// Возвращает имена всех паттернов в выражении.
	fn get_pattern_names(&self) -> BTreeSet<String> {
		let mut result = BTreeSet::new();
		self.travel(&mut |expr| {
			if let ExpressionMeta::Pattern { name } = expr.get_inner_expression_ref() {
				result.insert(name.clone());
			}
		});
		result
	}

	/// Возвращает имена и количество аргументов всех anyfunction в выражении. Если в выражении имеется `$f` от двух аргументов и от трёх, то возвратятся оба.
	fn get_anyfunction_names(&self) -> BTreeSet<(String, usize)> {
		let mut result = BTreeSet::new();
		self.travel(&mut |expr| {
			if let ExpressionMeta::AnyFunction { name, args } = expr.get_inner_expression_ref() {
				result.insert((name.clone(), args.len()));
			}
		});
		result
	}
}
