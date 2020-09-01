use std::borrow::Borrow;

/// Обобщённое выражение. Обобщённость нужна для возможности как задать положения в парсинге, так и для возмоности задания обычного выражения. Была выбрана такая обобщённость вместо копипасты данной структуры отдельно для парсинга.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ExpressionMeta<Arg> {
	/// В математике называется "переменной", но здесь это называется паттерном. Матчится с чем угодно, именованная часть выражения. В выражении выглядит как: `a`, `b`, `c`.
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

impl<Arg: GetInnerExpression> ExpressionMeta<Arg> {
	fn get_inner(self, position: &ExprPosition, deep: usize) -> Result<Self, PositionError> {
		use ExpressionMeta::*;

		match &position.0 {
			[start, tail @ ..] => {
				match self {
					AnyFunction { name: _, args } |
					NamedFunction { name: _, args } => args
						.into_iter()
						.nth(*start)
						.ok_or(PositionError(deep))?
						.get_inner_expression()
						.get_inner(ExprPosition::from_slice(tail), deep+1),

					Pattern { .. } |
					NamedValue { .. } |
					IntegerValue { .. } => Err(PositionError(deep)),
				}
			},
			[] => Ok(self),
		}
	}

	/// Получить владение на внутреннюю часть выражения.
	pub fn get(self, position: &ExprPosition) -> Result<Self, PositionError> {
		self.get_inner(position, 0)
	}

	fn get_ref_inner<'a>(&'a self, position: &ExprPosition, deep: usize) -> Result<&'a Self, PositionError> {
		use ExpressionMeta::*;

		match &position.0 {
			[start, tail @ ..] => {
				match self {
					AnyFunction { name: _, args } |
					NamedFunction { name: _, args } => args
						.get(*start)
						.ok_or(PositionError(deep))?
						.get_inner_expression_ref()
						.get_ref_inner(ExprPosition::from_slice(tail), deep+1),

					Pattern { .. } |
					NamedValue { .. } |
					IntegerValue { .. } => Err(PositionError(deep)),
				}
			},
			[] => Ok(self),
		}
	}

	/// Получить ссылку на внутреннюю часть выражения. 
	pub fn get_ref<'a>(&'a self, position: &ExprPosition) -> Result<&'a Self, PositionError> {
		self.get_ref_inner(position, 0)
	}

	fn get_mut_inner<'a>(&'a mut self, position: &ExprPosition, deep: usize) -> Result<&'a mut Self, PositionError> {
		use ExpressionMeta::*;

		match &position.0 {
			[start, tail @ ..] => {
				match self {
					AnyFunction { name: _, args } |
					NamedFunction { name: _, args } => args
						.get_mut(*start)
						.ok_or(PositionError(deep))?
						.get_inner_expression_mut()
						.get_mut_inner(ExprPosition::from_slice(tail), deep+1),

					Pattern { .. } |
					NamedValue { .. } |
					IntegerValue { .. } => Err(PositionError(deep)),
				}
			},
			[] => Ok(self),
		}
	}

	/// Получить изменяемую ссылку на внутреннюю часть выражения.
	pub fn get_mut<'a>(&'a mut self, position: &ExprPosition) -> Result<&'a mut Self, PositionError> {
		self.get_mut_inner(position, 0)
	}
}
