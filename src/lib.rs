pub mod binding;
pub mod parsing;
pub mod expr;
pub mod utils;
pub mod proof;

pub fn read_fpl(input: &str) -> Result<(), &'static str> {
	use crate::parsing::*;;
	use crate::proof::*;
	let parsed_math = parser::math(&input).unwrap();

	let math = read_math(&parsed_math)?;
	proofs_has_cycles(&parsed_math)?;
	is_proofs_correct(&parsed_math, &math)?;

	Ok(())
}