extern crate clap; 
use clap::App; 

fn main() {
    App::new("cpptors")
       .version("1.0")
       .about("Transpiles C++ to Rust. In theory.")
       .author("Adrian Taylor")
       .get_matches(); 
    println!("Hello, world!");
}
