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
    fn get_id(&self) -> &String;
    fn get_name(&self) -> &String;
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
    fn get_id(&self) -> &String {
        match &self {
            CodeFeature::Namespace(namespace) => &namespace.id,
            CodeFeature::Function(function) => &function.id,
            CodeFeature::FundamentalType(fundamental_type) => &fundamental_type.id,
            CodeFeature::Variable(variable) => &variable.id,
            CodeFeature::File(file) => &file.id
        }
    }
    fn get_name(&self) -> &String {
        match &self {
            CodeFeature::Namespace(namespace) => &namespace.name,
            CodeFeature::Function(function) => &function.name,
            CodeFeature::FundamentalType(fundamental_type) => &fundamental_type.name,
            CodeFeature::Variable(variable) => &variable.name,
            CodeFeature::File(file) => &file.name
        }
    }
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
    endline: String,
    #[serde(rename = "$value")]
    dump: Dump
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

#[derive(Deserialize, Debug)]
struct Dump {

}

// Now onto statements which may be found in a body

#[derive(Debug, Deserialize)]
enum Statement {
    #[serde(rename = "Var_Decl")]
    VarDecl(VarDecl),
    #[serde(rename = "Integer_Cst")]
    IntegerCst(IntegerCst),
    #[serde(rename = "Modify_Expr")]
    ModifyExpr(ModifyExpr),
    #[serde(rename = "Return_Stmt")]
    ReturnStmt(ReturnStmt)
}

#[derive(Debug, Deserialize)]
enum Expr {
    #[serde(rename = "Var_Decl")]
    VarDecl(VarDecl),
    #[serde(rename = "Integer_Cst")]
    IntegerCst(IntegerCst),
    #[serde(rename = "Modify_Expr")]
    ModifyExpr(ModifyExpr)
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Modify_Expr")]
struct ModifyExpr {
    target: Box<Expr>,
    source: Box<Expr>
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Return_Stmt")]
struct ReturnStmt {
    operand: Expr
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Var_Decl")]
struct VarDecl {
    id: String,
    name: String
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Integer_Cst")]
struct IntegerCst {
    #[serde(rename = "$value")]
    value: i32
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
        feature_by_id.insert(x.get_id().clone(), x);
    }

}
