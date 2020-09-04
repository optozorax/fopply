use std::ops::Range;

use crate::expr::*;
use crate::binding::{Binding, AnyFunctionPattern};
use crate::utils::char_index::*;
use crate::utils::apply::*;

pub struct FormulaPosition {
	pub module_name: String,
	pub position: usize,
	pub left_to_right: bool,
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

pub struct Formula {
	pub left: ExpressionParsing,
	pub right: ExpressionParsing,
}

pub struct ProofStep {
	pub string: String,
	pub expr: ExpressionParsing,
	pub position: Range<CharIndex>,
	pub used_formula: FormulaPosition,
	pub bindings: Vec<Binding>,
	pub function_bindings: Vec<(String, AnyFunctionPattern)>,
}

pub struct Proof {
	pub steps: Vec<ProofStep>,
}

pub struct FullFormula {
	pub position: u64,
	pub formula: Formula,
	pub proof: Option<Proof>,
}

pub struct NamedFormulas {
	pub name: String,
	pub formulas: Vec<FullFormula>,
}

pub struct Math(pub Vec<NamedFormulas>);

// TODO переделать на собственный алгоритм precedence!(), убрать костыль для парсинга неравенств и равенств
peg::parser!(
	pub grammar parser() for str {
		pub rule math() -> Math
			= _ named_formulas:(named_formulas:named_formulas() _ { named_formulas })+ {
				Math(named_formulas)
			}

		pub rule named_formulas() -> NamedFormulas 
			= "[" name:identifier() "]" _ formulas:(formulas:full_formula() _ { formulas })+ {
				NamedFormulas {
					name,
					formulas,
				}
			}

		pub rule full_formula() -> FullFormula
			= position:integer() "." _ formula:formula() _ proof:proof()? _ ";" {
				FullFormula {
					position,
					formula,
					proof,
				}
			}

		pub rule proof() -> Proof
			= "{" _ steps:(steps:proof_step() _ { steps })+ _ "}" {
				Proof {
					steps,
				}
			}

		pub rule proof_step() -> ProofStep
			= expr:&expr_normalized() string:$(expr_normalized()) _ ";" _ 
			  position:visual_positon() _ used_formula:formula_position() _ 
			  bindings:binding() ** (_ "," _ ) _ 
			  function_bindings:function_binding() ** (_ "," _ ) _ ";" {
				ProofStep {
					string: string.to_string(),
					expr,
					position,
					used_formula,
					bindings,
					function_bindings,
				}
			}

		pub rule formula() -> Formula
			= left:expr_normalized() _ "<->" _ right:expr_normalized() {
				Formula { 
					left, 
					right 
				}
			}

		pub rule function_binding() -> (String, AnyFunctionPattern)
			= "$" name:identifier() "(" _ variables:identifier() ** (_ "," _ ) _ ")" _ ":=" _ pattern:expr_normalized() {
				AnyFunctionPattern {
					pattern: clear_parsing_info(pattern),
					variables,
				}
				.apply(|x| (name, x))
			}

		pub rule binding() -> Binding
			= name:identifier() _ ":=" _ to:expr_normalized() { Binding::new(name, clear_parsing_info(to)) }
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

		pub rule expr_normalized() -> ExpressionParsing
			= expr_start:position!() result:expr() {
				let mut expr_result = result;
				substract_position(expr_start, &mut expr_result);
				expr_result
			}

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
			  r:(_ z:$("="/"!="/">="/"<="/">"/"<") _ p:sum() { (z, p) })? 
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
			/ pattern()

			/ integer_value()

		rule pattern() -> ExpressionParsing
			= start:position!() name:identifier() end:position!() { 
				ExpressionParsing {
					span: start..end,
					node: ExpressionMeta::Pattern { name } 
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

		rule _() = quiet!{[' ' | '\n' | '\r' | '\t']*}
	}
);

pub fn substract_position(to_substract: usize, expr: &mut ExpressionParsing) {
	expr.travel_mut(&mut |expr| {
		expr.span.start -= to_substract;
		expr.span.end -= to_substract;
	});
}

pub fn clear_parsing_info(expr: ExpressionParsing) -> Expression {
	use ExpressionMeta::*;

	Expression(
		match expr.node {
			AnyFunction { name, args } => 
				AnyFunction { name, args: args.into_iter().map(clear_parsing_info).collect() },
			NamedFunction { name, args } =>
				NamedFunction { name, args: args.into_iter().map(clear_parsing_info).collect() },
			Pattern { name } => 
				Pattern { name },
			NamedValue { name } => 
				NamedValue { name },
			IntegerValue { value } => 
				IntegerValue { value },
		}
	)
}

pub fn process_expression_parsing(expr: ExpressionParsing) -> (Expression, Vec<(ExprPositionOwned, Range<usize>)>) {
	fn process(
		expr: ExpressionMeta<ExpressionParsing>, 
		current_position: &mut ExprPositionOwned, 
		storage: &mut Vec<(ExprPositionOwned, Range<usize>)>
	) -> Expression {
		use ExpressionMeta::*;

		let mut process_args = |args: Vec<ExpressionParsing>| {
			args.into_iter().enumerate().map(|(pos, arg)| {
				current_position.0.push(pos);
				let ExpressionParsing { span, node } = arg;
				storage.push((current_position.clone(), span));
				let result = process(node, current_position, storage);
				current_position.0.pop().unwrap();
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
				NamedValue { name } => 
					NamedValue { name },
				IntegerValue { value } => 
					IntegerValue { value },
			}
		)
	}

	let mut storage = Vec::new();
	let mut current_position = Vec::new().apply(ExprPositionOwned);
	let ExpressionParsing { span, node } = expr;
	storage.push((current_position.clone(), span));
	(process(node, &mut current_position, &mut storage), storage)
}
