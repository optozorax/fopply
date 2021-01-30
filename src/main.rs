use fopply::read_fpl;

fn check_math() -> Result<(), ()> {
	Ok(read_fpl(&std::fs::read_to_string("fpl/math.fpl").map_err(|_| println!("can't read `math.fpl`"))?)?)
}

fn main() {
	if let Ok(_) = check_math() {
		println!("`math.fpl` is OK");
	}
}