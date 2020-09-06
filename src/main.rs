use fopply::read_fpl;

fn check_math() -> Result<(), &'static str> {
	Ok(read_fpl(&std::fs::read_to_string("fpl/math.fpl").map_err(|_| "can't read `math.fpl`")?)?)
}

fn main() {
	println!("{}", check_math().map(|_| "`math.fpl` is OK").unwrap_or_else(|err| err));
}