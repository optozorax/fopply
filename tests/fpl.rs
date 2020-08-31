#[test]
fn test() {
	assert!(parsing::expr("(a+b)+c").is_ok());
	assert!(parsing::expr("a+b+c").is_ok());
	assert!(parsing::expr("a+sin(b)").is_ok());
	assert!(parsing::expr("a+$f(b)").is_ok());
	assert!(parsing::expr("$true+$f(1)-a#n").is_ok());

	let expression = parsing::expr("part(b = 0, a, a*part($true, 1, $undefined))").unwrap();
	let expression = clear_parsing_info(expression);
	let formula = parsing::formula("part(cond, then, else) <-> part(not(cond), else, then)").unwrap();

	let mut bindings = BindingStorage::default();

	find_bindings(expression, formula.left, &mut bindings).unwrap();
	let result = apply_bindings(formula.right, &bindings).unwrap();

	let should_be = parsing::expr("part(not(b = 0), a*part($true, 1, $undefined), a)").unwrap();
	let should_be = clear_parsing_info(should_be);

	assert_eq!(result, should_be);
}

#[test]
fn test2() {
	let expression = parsing::expr("a").unwrap();
	let expression = clear_parsing_info(expression);
	let formula = parsing::formula("part(x, a, a) <-> a").unwrap();

	let mut bindings = BindingStorage::default();

	bindings.add(parsing::binding("x := b = 0").unwrap());

	find_bindings(expression, formula.right, &mut bindings).unwrap();
	let result = apply_bindings(formula.left, &bindings).unwrap();

	let should_be = parsing::expr("part(b = 0, a, a)").unwrap();
	let should_be = clear_parsing_info(should_be);

	assert_eq!(result, should_be);
}

macro_rules! same {
	($a:expr, $b:expr) => {
		assert_eq!(
			clear_parsing_info(parsing::expr($a).unwrap()), 
			clear_parsing_info(parsing::expr($b).unwrap())
		);
	};
}

#[test]
fn associativity() {
	same!("a+b+c", "a+(b+c)");
	same!("a+b*c", "a+(b*c)");
	same!("a*b+c", "(a*b)+c");
	same!("a*b*c", "a*(b*c)");
	same!("a^b^c", "(a^b)^c");
}

#[test]
fn priority() {
	same!("a*b+c*d", "(a*b)+(c*d)");
	same!("a^b*c^d", "(a^b)*(c^d)");
}

#[test]
fn parsing_info() {
	macro_rules! debug_unwrap {
		($name:ident( $($arg:expr),* )) => {{
			let mut debug_string = String::from(stringify!($name));
			debug_string.push_str("(");
			$(
				debug_string.push_str(concat!(stringify!($arg), " = "));
				debug_string.push_str(format!("{:?}", $arg).as_ref());
				debug_string.push_str(", ");
			)*
			if debug_string.ends_with(", ") {
				debug_string.pop();
				debug_string.pop();
			}
			debug_string.push_str(")");

			$name($($arg),*).unwrap_or_else(|| panic!(debug_string))
		}};
	}

	//let string = "part(b =      0, a, a *part($true, 1, $undefined))";
	let string = "a+b+c+d^f*e";
	let parsed = parsing::expr(string).unwrap();
	let (_, positions) = process_expression_parsing(parsed);
	let positions: Vec<_> = positions
		.into_iter()
		.map(|(pos, range)| {
			let new_range = debug_unwrap!(get_char_range(string, range));
			(pos, new_range)
		})
		.collect();

	for (pos, range) in positions {
		use std::iter;
		println!(
			"{eq}\n{spaces_before}{arrows}{spaces_after} - {position:?}",
			eq = string,
			spaces_before = iter::repeat(' ').take(range.start.0).collect::<String>(),
			arrows = iter::repeat('^').take(range.end.0 - range.start.0).collect::<String>(),
			spaces_after = iter::repeat(' ').take(string.len() - range.end.0).collect::<String>(),
			position = pos,
		)
	}
}