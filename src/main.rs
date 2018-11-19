extern crate clap; 
extern crate serde;
extern crate serde_xml_rs;
#[macro_use]
extern crate serde_derive;

use clap::App;
use clap::Arg;
use std::fs::File;
use std::collections::HashMap;

// TODO abstract out into another file
#[derive(Deserialize, Debug)]
#[serde(rename = "GCC_XML")]
struct GccXml {
    cvs_revision: String,
     #[serde(rename = "$value")]
    features: Vec<CodeFeature>,
}

trait GetId
{
    fn get_id(&self) -> &CodeFeatureId;
}

#[derive(Debug, Deserialize)]
enum CodeFeature {
    Namespace(Namespace),
    Function(Function),
    FundamentalType(FundamentalType),
    Variable(Variable),
    File(ZFile)
}

impl GetId for CodeFeature {
    fn get_id(&self) -> &CodeFeatureId {
        match &self {
            CodeFeature::Namespace(namespace) => &namespace.id,
            CodeFeature::Function(function) => &function.id,
            CodeFeature::FundamentalType(fundamental_type) => &fundamental_type.id,
            CodeFeature::Variable(variable) => &variable.id,
            CodeFeature::File(file) => &file.id
        }
    }
}

#[derive(Deserialize, Debug)]
struct CodeFeatureId {
    id: String,
    name: String
}

#[derive(Deserialize, Debug)]
struct Namespace {
    #[serde(flatten)]
    id: CodeFeatureId,
    members: String, // TODO, do better
    mangled: String,
    demangled: String
}

#[derive(Deserialize, Debug)]
struct Function {
    #[serde(flatten)]
    id: CodeFeatureId,
    returns: String,
    context: String,
    location: String,
    file: String,
    line: String,
    endline: String
}

#[derive(Deserialize, Debug)]
struct FundamentalType {
    #[serde(flatten)]
    id: CodeFeatureId,
}

#[derive(Deserialize, Debug)]
struct Variable {
    #[serde(flatten)]
    id: CodeFeatureId,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "File")]
struct ZFile {
    #[serde(flatten)]
    id: CodeFeatureId,
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
    // TODO: call gccxml directly with output into a temporary file
    let f = File::open(input).unwrap();
    let program: GccXml = serde_xml_rs::deserialize(f).unwrap();
    println!("Program is: {:?}", program);
    let mut feature_by_id = HashMap::new();
    for x in program.features {
        feature_by_id.insert(x.get_id().id.clone(), x);
    }
    
}
