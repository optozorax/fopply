#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CharIndex(usize);
pub fn get_char_range(s: &str, range: Range<usize>) -> Option<Range<CharIndex>> {
	let mut iter = s
		.char_indices()
		.enumerate()
		.map(|(char_position, (index, _))| (CharIndex(char_position), index))

		// TODO сделать чтобы были ленивые вычисления
		.chain(std::iter::once((CharIndex(s.chars().count()), s.len())));
	let start = iter.find(|(_, index)| *index == range.start)?.0;
	let end = iter.find(|(_, index)| *index == range.end)?.0;
	Some(start..end)
}
