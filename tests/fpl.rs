use std::collections::BTreeMap;

use fopply::parsing::*;
use fopply::binding::*;
use fopply::utils::char_index::*;

#[test]
fn test() {
	assert!(parser::expr("(a+b)+c").is_ok());
	assert!(parser::expr("a+b+c").is_ok());
	assert!(parser::expr("a+sin(b)").is_ok());
	assert!(parser::expr("a+$f(b)").is_ok());
	assert!(parser::expr("$true+$f(1)-an").is_ok());

	let mut any_function_bindings = ManualAnyFunctionBinding::new(BTreeMap::default());

	let expression = parser::expr("part(b = 0, a, a*part($true, 1, $undefined))").unwrap();
	let expression = clear_parsing_info(expression);
	let formula = parser::formula("part(cond, then, else) <-> part(not(cond), else, then)").unwrap();

	let mut bindings = BindingStorage::default();

	find_bindings(expression, &formula.left, &mut bindings, &mut any_function_bindings).unwrap();
	let result = apply_bindings(formula.right, &bindings, &any_function_bindings);

	let should_be = parser::expr("part(not(b = 0), a*part($true, 1, $undefined), a)").unwrap();
	let should_be = clear_parsing_info(should_be);

	assert_eq!(result, should_be);
}

#[test]
fn test2() {
	let expression = parser::expr("a").unwrap();
	let expression = clear_parsing_info(expression);
	let formula = parser::formula("part(x, a, a) <-> a").unwrap();

	let mut any_function_bindings = ManualAnyFunctionBinding::new(BTreeMap::default());

	let mut bindings = BindingStorage::default();

	bindings.add(parser::binding("x := b = 0").unwrap());

	find_bindings(expression, &formula.right, &mut bindings, &mut any_function_bindings).unwrap();
	let result = apply_bindings(formula.left, &bindings, &any_function_bindings);

	let should_be = parser::expr("part(b = 0, a, a)").unwrap();
	let should_be = clear_parsing_info(should_be);

	assert_eq!(result, should_be);
}

#[test]
fn test3() {
	let expression = parser::expr("part(not(b = 0), a*part($true, 1, $undefined), a)").unwrap();
	let expression = clear_parsing_info(expression);
	let formula = parser::formula("part(cond, $f(part(cond2, then2, else2)), else) <-> part(cond, $f(part(cond2 & cond, then2, else2)), else)").unwrap();

	let mut binding_map = BTreeMap::new();
	let (key, value) = parser::function_binding("$f(x) := a*x").unwrap();
	binding_map.insert(key, value);
	let mut any_function_bindings = ManualAnyFunctionBinding::new(binding_map);

	let mut bindings = BindingStorage::default();

	find_bindings(expression, &formula.left, &mut bindings, &mut any_function_bindings).unwrap();
	let result = apply_bindings(formula.right, &bindings, &any_function_bindings);

	let should_be = parser::expr("part(not(b = 0), a*part($true & not(b = 0), 1, $undefined), a)").unwrap();
	let should_be = clear_parsing_info(should_be);

	assert_eq!(result, should_be);
}

macro_rules! same {
	($a:expr, $b:expr) => {
		assert_eq!(
			clear_parsing_info(parser::expr($a).unwrap()), 
			clear_parsing_info(parser::expr($b).unwrap())
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
	let parsed = parser::expr(string).unwrap();
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
