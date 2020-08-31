use crate::parsing::FormulaPosition;
use crate::expr::*;
use crate::binding::*;

pub struct ProofStep {
	current_expression: Expression,
	position: ExprPositionOwned,
	used_formula: FormulaPosition,
	bindings: Vec<Binding>,
	// ??? for $f(x) = a*x, таких может быть несколько, и они могут применяться рекурсивно
}

pub struct Proof {
	steps: Vec<ProofStep>,
}
