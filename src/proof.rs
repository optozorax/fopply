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

#[derive(Default, Ord, PartialOrd, Debug, Clone, Eq, PartialEq, Hash)]
pub struct FormulaPosition {
	pub module_name: String,
	pub position: usize,
}

pub fn read_math(math: &Math) -> Option<BTreeMap<FormulaPosition, Formula>> {
	let mut result = BTreeMap::new();
	for NamedFormulas { name, formulas } in &math.0 {
		for (index, formula) in formulas.iter().enumerate() {
			if index + 1 != formula.position as usize {
				return None;
			}

			let position = FormulaPosition {
				module_name: name.clone(),
				position: formula.position as usize
			};
			let formula = Formula::new(
				clear_parsing_info(formula.formula.left.clone()),
				clear_parsing_info(formula.formula.right.clone())
			)?;

			result.insert(position, formula);
		} 
	}
	Some(result)
}

pub fn proofs_has_cycles(math: &Math) -> bool {
	let mut id_generator = IdGenerator::default();
	let mut edges = vec![];
	for NamedFormulas { name, formulas } in &math.0 {
		for (index, formula) in formulas.iter().enumerate() {
			let current_position = NodeIndex::new(id_generator.get_or_add_id(FormulaPosition {
				module_name: name.clone(),
				position: index + 1,
			}) as usize);
			if let Some(proof) = &formula.proof {
				for ProofStep { used_formula, .. } in &proof.steps {
					let used_position = NodeIndex::new(id_generator.get_or_add_id(FormulaPosition {
						module_name: used_formula.module_name.clone(),
						position: used_formula.position,
					}) as usize);
					edges.push((current_position, used_position));
				}
			}
		}
	}

	let graph = Graph::<(), ()>::from_edges(&edges);
	petgraph::algo::is_cyclic_directed(&graph)
}

pub fn is_proofs_correct(math: &Math, global_formulas: &BTreeMap<FormulaPosition, Formula>) -> Result<(), &'static str> {
	for NamedFormulas { name: _, formulas } in &math.0 {
		for formula in formulas {
			if let Some(proof) = &formula.proof {
				let mut current = clear_parsing_info(formula.formula.left.clone());

				for ProofStep { string, expr, position, used_formula, bindings, function_bindings } in &proof.steps {
					let (mut expr, position) = {
						let (expr, positions) = process_expression_parsing(expr.clone());
						let position = positions.iter()
							.find(|(_, range)| get_char_range(&string, range.clone()) == Some(position.clone())).ok_or("position not found")?.0.clone();
						(expr, position)
					};

					if expr != current {
						return Err("proof step wrong");
					}

					let formula = {
						let formula_position = FormulaPosition {
							module_name: used_formula.module_name.clone(),
							position: used_formula.position,
						};
						let mut result = global_formulas.get(&formula_position).ok_or("formula not found")?.clone();
						if !used_formula.left_to_right {
							std::mem::swap(&mut result.left, &mut result.right);
						}

						let sorted_unknown_names: BTreeSet<String> = result.left.unknown_patterns_names.iter().cloned().collect();
						let sorted_used_names: BTreeSet<String> = bindings.iter().map(|b| b.pattern_name.clone()).collect();
						if sorted_unknown_names != sorted_used_names {
							return Err("not all bindings provided");
						}

						let sorted_unknown_anyfunctions: BTreeSet<(String, usize)> = result.left.anyfunction_names.iter().cloned().collect();
						let sorted_function_bindings: BTreeSet<(String, usize)> = function_bindings.iter().map(|(name, pattern)| (name.clone(), pattern.variables.len())).collect();
						if sorted_unknown_anyfunctions != sorted_function_bindings {
							return Err("not all function bindings provided");
						}

						result
					};					

					let mut current_expr_part = Expression(ExpressionMeta::IntegerValue { value: 0 });
					let current_expr = expr.get_mut(position.borrow()).map_err(|_| "position not found in expression")?;
					std::mem::swap(&mut current_expr_part, current_expr);

					let mut bindings = {
						let mut result = BindingStorage::default();
						for binding in bindings {
							result.add(binding.clone());
						}
						result
					};
					
					let mut any_function_bindings = {
						let mut binding_map = BTreeMap::new();
						for binding in function_bindings {
							binding_map.insert(binding.0.clone(), binding.1.clone());
						}

						ManualAnyFunctionBinding::new(binding_map)
					};

					find_bindings(
						current_expr_part, 
						&formula.left.pattern,
						&mut bindings,
						&mut any_function_bindings
					).ok_or("cannot find bindings")?;
					let mut current_expr_part = apply_bindings(
						formula.right.pattern.clone(),
						&bindings,
						&any_function_bindings
					);

					std::mem::swap(&mut current_expr_part, current_expr);

					current = expr;
				}
			}
		}
	}

	Ok(())
}