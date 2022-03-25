#[no_mangle]
pub extern fn greet(excitedness: u32) {
	let mut s = String::from("Hello");
	let exclamation_marks = std::iter::repeat('!').take(excitedness as usize);
	s.extend(exclamation_marks);
	println!("{}", s);
}
