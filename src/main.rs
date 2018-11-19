extern crate clap; 
extern crate serde;
extern crate serde_xml_rs;
#[macro_use]
extern crate serde_derive;

use clap::App;
use clap::Arg;
use std::fs::File;

#[derive(Deserialize, Debug)]
#[serde(rename = "GCC_XML")]
struct GccXml {
    cvs_revision: String,
     #[serde(rename = "$value")]
    features: Vec<CodeFeature>,
}

#[derive(Debug, Deserialize)]
enum CodeFeature {
    Namespace(Namespace),
    Function(Function),
    FundamentalType(FundamentalType),
    Variable(Variable),
    File(ZFile)
}

#[derive(Deserialize, Debug)]
struct Namespace {
    id: String,
    name: String,
    members: String, // TODO, do better
    mangled: String,
    demangled: String
}

#[derive(Deserialize, Debug)]
struct Function {
    id: String,
    name: String,
    returns: String,
    context: String,
    location: String,
    file: String,
    line: String,
    endline: String
}

#[derive(Deserialize, Debug)]
struct FundamentalType {
    id: String,
    name: String
}

#[derive(Deserialize, Debug)]
struct Variable {
    id: String,
    name: String
}

#[derive(Deserialize, Debug)]
#[serde(rename = "File")]
struct ZFile {
    id: String,
    name: String
}

fn main() {
    let matches = App::new("cpptors")
       .version("1.0")
       .about("Transpiles C++ to Rust. In theory.")
       .author("Adrian Taylor")
       .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to use")
                               .required(true)
                               .index(1))
       .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    println!("Using input file: {}", input);
    let f = File::open(input).unwrap();
    let program: GccXml = serde_xml_rs::deserialize(f).unwrap();
    println!("Program is: {:?}", program)
}
