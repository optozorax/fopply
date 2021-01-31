use annotate_snippets::{
	display_list::{DisplayList, FormatOptions},
	snippet::{Annotation, AnnotationType, Snippet},
};

use crate::utils::span::peg_error_to_snippet;

pub mod binding;
pub mod expr;
pub mod parsing;
pub mod proof;
pub mod utils;

pub fn read_fpl(input: &str) -> Result<(), ()> {
	use crate::{parsing::*, proof::*};

	let parsed_math = parser::math(&input).map_err(|err| peg_error_to_snippet(err, input))?;

	let math = read_math(&parsed_math).map_err(|errs| {
		for err in errs {
			err.print_error_snippet(input);
		}
	})?;
	proofs_has_cycles(&parsed_math).map_err(|text| {
		let snippet = Snippet {
			title: Some(Annotation { label: Some(text), id: None, annotation_type: AnnotationType::Error }),
			opt: FormatOptions { color: true, ..Default::default() },
			..Snippet::default()
		};
		println!("{}", DisplayList::from(snippet));
	})?;
	is_proofs_correct(&parsed_math, &math).map_err(|errs| {
		for err in errs {
			err.print_error_snippet(input);
		}
	})?;

	Ok(())
}
