use std::{fs, env};
use egg::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let src_rvsdg: String = fs::read_to_string(&args[1]).unwrap();
    println!("{}", src_rvsdg);
}
