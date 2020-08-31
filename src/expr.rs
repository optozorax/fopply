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

pub trait GetInnerExpression: Sized {
	fn get_inner_expression(self) -> ExpressionMeta<Self>;
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self>;
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self>;
}

/// Выражение
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression(pub ExpressionMeta<Expression>);

impl GetInnerExpression for Expression {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.0 }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.0 }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.0 }
}

pub struct ExprPositionOwned(Vec<usize>);
pub struct ExprPosition([usize]);
// TODO impl Borrow<ExprPosition> for ExprPositionOwned


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
