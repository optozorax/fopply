use std::ops::Range;
use crate::utils::span::*;

use crate::expr::*;
use crate::binding::{Binding, AnyFunctionPattern};
use crate::utils::char_index::*;
use crate::utils::apply::*;

#[derive(Debug)]
pub struct FormulaPosition {
	pub module_name: String,
	pub position: usize,
	pub left_to_right: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionParsingGlobal {
	pub span: GlobalSpan,
	pub node: ExpressionMeta<ExpressionParsingGlobal>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionParsing {
	pub span: LocalSpan,
	pub node: ExpressionMeta<ExpressionParsing>,
}

impl GetInnerExpression for ExpressionParsingGlobal {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.node }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.node }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.node }
}

impl GetInnerExpression for ExpressionParsing {
	fn get_inner_expression(self) -> ExpressionMeta<Self> { self.node }
	fn get_inner_expression_ref(&self) -> &ExpressionMeta<Self> { &self.node }
	fn get_inner_expression_mut(&mut self) -> &mut ExpressionMeta<Self> { &mut self.node }
}

#[derive(Debug)]
pub struct Formula {
	pub left: ExpressionParsing,
	pub right: ExpressionParsing,
}

#[derive(Debug)]
pub struct ProofStep {
	pub string: String,
	pub expr: Spanned<ExpressionParsing>,
	pub position: Spanned<Range<CharIndex>>,
	pub used_formula: Spanned<FormulaPosition>,
	pub bindings: Spanned<Vec<Binding>>,
	pub function_bindings: Spanned<Vec<(String, AnyFunctionPattern)>>,
}

#[derive(Debug)]
pub struct Proof {
	pub steps: Vec<ProofStep>,
}

#[derive(Debug)]
pub struct FullFormula {
	pub position: Spanned<u64>,
	pub formula: Spanned<Formula>,
	pub proof: Option<Spanned<Proof>>,
}

#[derive(Debug)]
pub struct NamedFormulas {
	pub name: String,
	pub formulas: Vec<FullFormula>,
}

#[derive(Debug)]
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
			= start:position!() position:integer() end:position!()  "." _ 
			  start2:position!() formula:formula() end2:position!() _ 
			  start3:position!() proof:proof()? end3:position!() _ ";" {
				FullFormula {
					position: Spanned {
						span: GlobalSpan(start..end),
						inner: position,
					},
					formula: Spanned {
						span: GlobalSpan(start2..end2),
						inner: formula,
					},
					proof: proof.map(|x| Spanned {
						span: GlobalSpan(start3..end3),
						inner: x,
					}),
				}
			}

		pub rule proof() -> Proof
			= "{" _ steps:(steps:proof_step() _ { steps })+ _ "}" {
				Proof {
					steps,
				}
			}

		pub rule proof_step() -> ProofStep
			= start1:position!() expr:&expr_normalized() string:$(expr_normalized()) end1:position!() _ ";" _ 
			  start2:position!() position:visual_positon() end2:position!() _ 
			  start3:position!() used_formula:formula_position() end3:position!() _ 
			  start4:position!() bindings:binding() ** (_ "," _ ) end4:position!() _ 
			  start5:position!() function_bindings:function_binding() ** (_ "," _ ) end5:position!() _ ";" {
				ProofStep {
					string: string.to_string(),
					expr: Spanned {
						span: GlobalSpan(start1..end1),
						inner: expr,
					},
					position: Spanned {
						span: GlobalSpan(start2..end2),
						inner: position,
					},
					used_formula: Spanned {
						span: GlobalSpan(start3..end3),
						inner: used_formula,
					},
					bindings: Spanned {
						span: GlobalSpan(start4..end4),
						inner: bindings,
					},
					function_bindings: Spanned {
						span: GlobalSpan(start5..end5),
						inner: function_bindings,
					},
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
				localize_span(expr_start, result)
			}

		pub rule expr() -> ExpressionParsingGlobal
			= or()

		rule or() -> ExpressionParsingGlobal
			= start:position!() 
			  l:and() 
			  r:(_ z:$("|") _ r:and() end:position!()  { (z, r, end) })* 
			  end:position!() 
			{
				let mut result = l;
				for (z, r, end) in r {
					result = ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![result, r] }
					};
				}
				result
			}

		rule and() -> ExpressionParsingGlobal
			= start:position!() 
			  l:equality() 
			  r:(_ z:$("&") _ r:equality() end:position!()  { (z, r, end) })* 
			  end:position!() 
			{
				let mut result = l;
				for (z, r, end) in r {
					result = ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![result, r] }
					};
				}
				result
			}

		rule equality() -> ExpressionParsingGlobal
			= start:position!() 
			  l:sum() 
			  r:(_ z:$("="/"!="/">="/"<="/">"/"<") _ p:sum() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule sum() -> ExpressionParsingGlobal
			= start:position!() 
			  l:product() 
			  r:(_ z:$("+"/"-") _ r:product() end:position!()  { (z, r, end) })* 
			{
				let mut result = l;
				for (z, r, end) in r {
					result = ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![result, r] }
					};
				}
				result
			}
			/ start:position!() "-" _ l:product() end:position!() {
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
					node: ExpressionMeta::NamedFunction { name: "negative".to_string(), args: vec![l] }
				}
			}

		rule product() -> ExpressionParsingGlobal
			= start:position!() 
			  l:power() 
			  r:(_ z:$("*"/"/") _ r:power() end:position!()  { (z, r, end) })* 
			  end:position!() 
			{
				let mut result = l;
				for (z, r, end) in r {
					result = ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![result, r] }
					};
				}
				result
			}

		rule power() -> ExpressionParsingGlobal
			= start:position!() 
			  l:atom()
			  r:(_ z:$("^") _ p:power() { (z, p) })? 
			  end:position!() 
			{
				match r {
					Some((z, p)) => ExpressionParsingGlobal {
						span: GlobalSpan(start..end),
						node: ExpressionMeta::NamedFunction { name: z.to_string(), args: vec![l, p] }
					},
					None => l,
				}
			}

		rule atom() -> ExpressionParsingGlobal
			= "(" v:expr() ")" { v }

			/ any_function()
			/ named_value()

			/ function()
			/ pattern()

			/ integer_value()

		rule pattern() -> ExpressionParsingGlobal
			= start:position!() name:identifier() end:position!() { 
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
					node: ExpressionMeta::Pattern { name } 
				}
			}

		rule function() -> ExpressionParsingGlobal
			= start:position!() name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" end:position!() { 
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
					node: ExpressionMeta::NamedFunction { name, args }
				}
			}

		rule any_function() -> ExpressionParsingGlobal
			= start:position!() "$" name:identifier() "(" _ args:expr() ** (_ "," _) _ ")" end:position!() { 
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
					node: ExpressionMeta::AnyFunction { name, args }
				}
			}

		rule integer_value() -> ExpressionParsingGlobal
			= start:position!() value:integer() end:position!() { 
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
					node: ExpressionMeta::IntegerValue { value: value as i64 }
				}
			}

		rule named_value() -> ExpressionParsingGlobal
			= start:position!() "$" name:identifier() end:position!() { 
				ExpressionParsingGlobal {
					span: GlobalSpan(start..end),
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

pub fn localize_span(start: usize, expr: ExpressionParsingGlobal) -> ExpressionParsing {
	expr.retype(
		&|expr| (expr.span.localize_span(start), expr.node), 
		&|span, node| ExpressionParsing { span, node }
	)
}

pub fn clear_parsing_info(expr: ExpressionParsing) -> Expression {
	expr.retype(
		&|expr| ((), expr.node), 
		&|(), node| Expression(node)
	)
}

pub fn process_expression_parsing(expr: ExpressionParsing) -> (Expression, Vec<(ExprPositionOwned, LocalSpan)>) {
	fn process(
		expr: ExpressionMeta<ExpressionParsing>, 
		current_position: &mut ExprPositionOwned, 
		storage: &mut Vec<(ExprPositionOwned, LocalSpan)>
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
