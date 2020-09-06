use std::ops::Range;

/// Индекс в строке по `char`'ам.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CharIndex(pub usize);

/// Преобразовать промежуток для строки из промежутка по байтам в промежутом по `char`'ам. Возвращает `None` если `range` ложится в некорректные места кодировки `utf-8`.
pub fn get_char_range(s: &str, range: Range<usize>) -> Option<Range<CharIndex>> {
	let mut iter = s
		.char_indices()
		.enumerate()
		.map(|(char_position, (index, _))| (CharIndex(char_position), index));

	let start = iter.find(|(_, index)| *index == range.start)?.0;
	let end = iter
		.find(|(_, index)| *index == range.end)
		.map(|x| x.0)
		.or_else(|| {
			if range.end == s.len() {
				Some(CharIndex(s.chars().count()))
			} else {
				None
			}
		})?;

	Some(start..end)
}
