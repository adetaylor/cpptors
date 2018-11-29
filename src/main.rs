extern crate clap; 
extern crate serde;
extern crate serde_xml_rs;
#[macro_use]
extern crate serde_derive;
extern crate tempfile;

use clap::App;
use clap::Arg;
use std::fs::File;
use std::collections::HashMap;
use std::process::Command;

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
    dump: Option<Dump>
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
    version: String,
    #[serde(rename = "$value")]
    body: Body
}

#[derive(Deserialize, Debug)]
struct Body {
    #[serde(rename = "$value")]
    statement_list: StatementList
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Statement_List")]
struct StatementList {
    #[serde(rename = "$value")]
    statements: Vec<Statement>
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
    ModifyExpr(ModifyExpr),
    #[serde(rename = "Result_Decl")]
    ResultDecl(ResultDecl)
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Modify_Expr")]
struct ModifyExpr {
    #[serde(rename = "$value")]
    operands: Vec<Expr>
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Return_Stmt")]
struct ReturnStmt {
    #[serde(rename = "$value")]
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

#[derive(Deserialize, Debug)]
#[serde(rename = "Result_Decl")]
struct ResultDecl {
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
    let tmpfile : Option<tempfile::NamedTempFile>;
    let properfile : Option<File>;
    let f : &File = if !input.ends_with("xml") {
        tmpfile = Some(tempfile::NamedTempFile::new().expect("Unable to create temporary file"));
        let x = tmpfile.as_ref().unwrap();
        let tmp_path = x.path().to_str().expect("Invalid temp file name");
        Command::new("C:\\dev\\rust\\cpptors\\gccxml_cc1plus.exe")
            .arg(format!("-fxml={}", tmp_path))
            .arg(input)
            .output()
            .expect("Failed to run gccxml");
        tmpfile.as_ref().unwrap().as_file()
    } else {
        properfile = Some(File::open(input).unwrap());
        &properfile.as_ref().unwrap()
    };
    let program: GccXml = serde_xml_rs::deserialize(f).unwrap();
    println!("Program is: {:?}", program);
    let mut feature_by_id = HashMap::new();
    for x in program.features {
        feature_by_id.insert(x.get_id().clone(), x);
    }

}
