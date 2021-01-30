use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use std::fmt::Display;
use std::ops::Range;
use crate::utils::joined_by::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct GlobalSpan(pub Range<usize>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LocalSpan(pub Range<usize>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Spanned<T> {
	pub span: GlobalSpan,
	pub inner: T,
}

impl GlobalSpan {
	pub fn localize_span(self, start: usize) -> LocalSpan {
		LocalSpan(self.0.start - start..self.0.end - start)
	}
}

impl LocalSpan {
	pub fn globalize_span(self, start: usize) -> GlobalSpan {
		GlobalSpan(self.0.start + start..self.0.end + start)
	}
}

impl<T> Spanned<T> {
	pub fn new(t: T, span: GlobalSpan) -> Self {
		Spanned {
			span,
			inner: t,
		}
	}

	pub fn map<Y, F: FnOnce(T) -> Y>(self, f: F) -> Spanned<Y> {
		Spanned {
			span: self.span,
			inner: f(self.inner), 
		}
	}
}

impl<T: Display> Spanned<T> {
	pub fn print_error_snippet<'a>(&self, string: &'a str) {
		let text = self.inner.to_string();
		let (line_no, line_range_start) = find_line_number(string, self.span.0.start);
		let (_, line_range_end) = find_line_number(string, self.span.0.end);
		let snippet = Snippet {
			title: Some(Annotation {
				label: Some(&text),
				id: None,
				annotation_type: AnnotationType::Error,
			}),
			footer: vec![],
			slices: vec![
				Slice {
					source: &string[line_range_start.start..line_range_end.end],
					line_start: line_no,
					origin: None,
					fold: true,
					annotations: vec![
						SourceAnnotation {
							label: "",
							annotation_type: AnnotationType::Error,
							range: (self.span.0.start - line_range_start.start, self.span.0.end - line_range_start.start),
						},
					],
				},
			],
			opt: FormatOptions {
				color: true,
				..Default::default()
			},
		};
		println!("{}", DisplayList::from(snippet));
	}
}

fn find_char_pos(string: &str, byte_pos: usize) -> usize {
	string
		.char_indices()
		.map(|(index, _)| index)
		.enumerate()
		.find(|(_, index)| index == &byte_pos)
		.map(|(char_pos, _)| char_pos)
		.unwrap_or_else(|| panic!("Wrong position `{:?}` on string: {:?}", byte_pos.clone(), string))
}

fn find_line_number(string: &str, pos: usize) -> (usize, Range<usize>) {
	string.char_indices().filter(|(_, ch)| *ch == '\n').enumerate().scan(0, |state, x| {
		let result = (x.0, *state+1..x.1.0);
		*state = x.1.0;
		Some(result)
	}).find(|(_, range)| range.contains(&pos)).map(|(line_no, range)| (line_no+1, range)).unwrap()
}

pub trait GetErrorCharsRange {
	fn get_error_range(&self, string: &str) -> (usize, usize);
}

impl GetErrorCharsRange for usize {
	fn get_error_range(&self, string: &str) -> (usize, usize) {
		(
			find_char_pos(string, *self),
			find_char_pos(string, *self) + 1,
		)
	}	
}

impl GetErrorCharsRange for peg::str::LineCol {
	fn get_error_range(&self, string: &str) -> (usize, usize) {
		(
			find_char_pos(string, self.offset),
			find_char_pos(string, self.offset) + 1,
		)
	}
}


impl GetErrorCharsRange for std::ops::Range<usize> {
	fn get_error_range(&self, string: &str) -> (usize, usize) {
		(
			find_char_pos(string, self.start),
			find_char_pos(string, self.end),
		)
	}
}

/// Преобразование ошибки `rust-peg` в формат `snippet`.
pub fn peg_error_to_snippet<T: GetErrorCharsRange>(err: peg::error::ParseError<T>, string: &str) {
	let inner_text = format!("expected tokens: {}", 
		err.expected
			.tokens()
			.collect::<Vec<_>>()
			.into_iter()
			.joined_by(", ")
	);

	let snippet = Snippet {
		title: Some(Annotation {
			label: Some("unexpected token"),
			id: None,
			annotation_type: AnnotationType::Error,
		}),
		footer: vec![Annotation {
			label: Some(&inner_text),
			id: None,
			annotation_type: AnnotationType::Note,
		}],
		slices: vec![
			Slice {
				source: string,
				line_start: 0,
				origin: None,
				fold: true,
				annotations: vec![
					SourceAnnotation {
						label: "unexpected token",
						annotation_type: AnnotationType::Error,
						range: err.location.get_error_range(string),
					},
				],
			},
		],
		opt: FormatOptions {
			color: true,
			..Default::default()
		},
	};
	println!("{}", DisplayList::from(snippet));
}