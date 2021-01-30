use crate::expr::PositionError;
use crate::expr::ExprPositionOwned;
use crate::utils::span::*;
use crate::parsing::Proof;
use crate::binding::FormulaError;
use crate::parsing::process_expression_parsing;
use crate::utils::char_index::get_char_range;
use std::collections::BTreeSet;
use crate::binding::find_bindings;
use crate::binding::apply_bindings;
use std::collections::BTreeMap;
use crate::binding::{Formula, BindingStorage, ManualAnyFunctionBinding};
use crate::parsing::{clear_parsing_info, Math, NamedFormulas, ProofStep};
use petgraph::{Graph, graph::NodeIndex};
use crate::utils::id::*;
use std::borrow::Borrow;
use crate::expr::{ExpressionMeta, ExpressionExtension, Expression};
use thiserror::Error;

#[derive(Default, Ord, PartialOrd, Debug, Clone, Eq, PartialEq, Hash)]
pub struct FormulaPosition {
	pub module_name: String,
	pub position: usize,
}

#[derive(Debug, Error)]
pub enum ReadMathError {
	#[error("wrong number, should be {should_be}")]
	WrongNumberInStart {
		should_be: usize,
	},
	#[error("{0}")]
	FormulaError(FormulaError),
}

pub fn read_math(math: &Math) -> Result<BTreeMap<FormulaPosition, Formula>, Vec<Spanned<ReadMathError>>> {
	let mut errors = Vec::new();
	let mut result = BTreeMap::new();
	for NamedFormulas { name, formulas } in &math.0 {
		for (index, formula) in formulas.iter().enumerate() {
			if index + 1 != formula.position.inner as usize {
				errors.push(Spanned::new(ReadMathError::WrongNumberInStart {
					should_be: index+1,
				}, formula.position.span.clone()));
				continue;
			}

			let position = FormulaPosition {
				module_name: name.clone(),
				position: formula.position.inner as usize
			};

			let formula = match Formula::new(
				clear_parsing_info(formula.formula.inner.left.clone()),
				clear_parsing_info(formula.formula.inner.right.clone())
			) {
				Ok(x) => x,
				Err(x) => {
					errors.push(Spanned::new(ReadMathError::FormulaError(x), formula.formula.span.clone()));
					continue;
				}
			};

			result.insert(position, formula);
		} 
	}
	if errors.len() == 0 {
		Ok(result)
	} else {
		Err(errors)
	}
}

pub fn proofs_has_cycles(math: &Math) -> Result<(), &'static str> {
	let mut id_generator = IdGenerator::default();
	let mut edges = vec![];
	for NamedFormulas { name, formulas } in &math.0 {
		for (index, formula) in formulas.iter().enumerate() {
			let current_position = NodeIndex::new(id_generator.get_or_add_id(FormulaPosition {
				module_name: name.clone(),
				position: index + 1,
			}) as usize);
			if let Some(proof) = &formula.proof {
				for ProofStep { used_formula, .. } in &proof.inner.steps {
					let used_position = NodeIndex::new(id_generator.get_or_add_id(FormulaPosition {
						module_name: used_formula.inner.module_name.clone(),
						position: used_formula.inner.position,
					}) as usize);
					edges.push((current_position, used_position));
				}
			}
		}
	}

	let graph = Graph::<(), ()>::from_edges(&edges);
	if petgraph::algo::is_cyclic_directed(&graph) {
		Err("proof has cycles")
	} else {
		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum ProofError {
	#[error("position is not found")]
	PositionNotFound,
	#[error("result of this step is not equal to expected, actual is {actual}")]
	StepWrong {
		actual: Expression,
	},
	#[error("result of latest step is not equal to right side of formula, actual is {actual}")]
	LatestStepWrong {
		actual: Expression,
	},
	#[error("formula by this name is not found")]
	FormulaNotFound,
	#[error("not all bindings provided")]
	NotAllBindingsProvided, // TODO add which bindings needed
	#[error("not all function bindings provided")]
	NotAllFunctionBindingsProvided,
	#[error("internal error about getting part of formula, in {position:?}, on {error_in:?}")]
	InternalError {
		position: ExprPositionOwned,
		error_in: PositionError,
	},
	#[error("cannot match formula with this equation")]
	CannotFindBindings,
}

pub fn is_proof_correct(formula: &crate::parsing::Formula, proof: &Spanned<Proof>, global_formulas: &BTreeMap<FormulaPosition, Formula>) -> Result<(), Spanned<ProofError>> {
	let mut current = clear_parsing_info(formula.left.clone());

	for ProofStep { string, expr, position, used_formula, bindings, function_bindings } in &proof.inner.steps {
		let expr_parsing = &expr.inner;
		let expr_span = expr.span.clone();
		let (mut expr, position) = {
			let (expr, positions) = process_expression_parsing(expr.inner.clone());
			let position = positions.iter()
				.find(|(_, range)| get_char_range(&string, range.0.clone()).map(|x| x == position.inner).unwrap_or(false)).ok_or(Spanned::new(ProofError::PositionNotFound, position.span.clone()))?.0.clone();
			(expr, position)
		};

		if expr != current {
			return Err(Spanned::new(ProofError::StepWrong { actual: current }, expr_span));
		}

		let formula = {
			let formula_position = FormulaPosition {
				module_name: used_formula.inner.module_name.clone(),
				position: used_formula.inner.position,
			};
			let mut result = global_formulas.get(&formula_position).ok_or(Spanned::new(ProofError::FormulaNotFound, used_formula.span.clone()))?.clone();
			if !used_formula.inner.left_to_right {
				std::mem::swap(&mut result.left, &mut result.right);
			}

			let sorted_unknown_names: BTreeSet<String> = result.left.unknown_patterns_names.iter().cloned().collect();
			let sorted_used_names: BTreeSet<String> = bindings.inner.iter().map(|b| b.pattern_name.clone()).collect();
			if sorted_unknown_names != sorted_used_names {
				return Err(Spanned::new(ProofError::NotAllBindingsProvided, bindings.span.clone()));
			}

			let sorted_unknown_anyfunctions: BTreeSet<(String, usize)> = result.left.anyfunction_names.iter().cloned().collect();
			let sorted_function_bindings: BTreeSet<(String, usize)> = function_bindings.inner.iter().map(|(name, pattern)| (name.clone(), pattern.variables.len())).collect();
			if sorted_unknown_anyfunctions != sorted_function_bindings {
				return Err(Spanned::new(ProofError::NotAllFunctionBindingsProvided, function_bindings.span.clone()));
			}

			result
		};					

		let mut current_expr_part = Expression(ExpressionMeta::IntegerValue { value: 0 });
		let current_expr = expr.get_mut(position.borrow()).map_err(|pos| Spanned::new(ProofError::InternalError { position: position.clone(), error_in: pos }, expr_parsing.get(position.cut_to_error(pos)).unwrap().span.clone().globalize_span(expr_span.0.start)))?;
		std::mem::swap(&mut current_expr_part, current_expr);

		let mut bindings = {
			let mut result = BindingStorage::default();
			for binding in &bindings.inner {
				result.add(binding.clone());
			}
			result
		};
		
		let mut any_function_bindings = {
			let mut binding_map = BTreeMap::new();
			for binding in &function_bindings.inner {
				binding_map.insert(binding.0.clone(), binding.1.clone());
			}

			ManualAnyFunctionBinding::new(binding_map)
		};

		find_bindings(
			current_expr_part, 
			&formula.left.pattern,
			&mut bindings,
			&mut any_function_bindings
		).ok_or(Spanned::new(ProofError::CannotFindBindings, expr_span.clone()))?;
		let mut current_expr_part = apply_bindings(
			formula.right.pattern.clone(),
			&bindings,
			&any_function_bindings
		);

		std::mem::swap(&mut current_expr_part, current_expr);

		current = expr;
	}

	if clear_parsing_info(formula.right.clone()) != current {
		return Err(Spanned::new(ProofError::LatestStepWrong { actual: current }, proof.span.clone()));
	}


	Ok(())
}

pub fn is_proofs_correct(math: &Math, global_formulas: &BTreeMap<FormulaPosition, Formula>) -> Result<(), Vec<Spanned<ProofError>>> {
	let mut result = Vec::new();
	for NamedFormulas { name: _, formulas } in &math.0 {
		for formula in formulas {
			if let Some(proof) = &formula.proof {
				if let Err(error) = is_proof_correct(&formula.formula.inner, proof, global_formulas) {
					result.push(error);
				}
			}
		}
	}

	if result.is_empty() {
		Ok(())
	} else {
		Err(result)
	}
}