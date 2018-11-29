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
use std::path::{Path, PathBuf};

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
    let tmpdir : Option<tempfile::TempDir>;
    let xml_path : PathBuf = if input.ends_with("xml") {
        Path::new(input).to_path_buf()
    } else {
        tmpdir = Some(tempfile::tempdir().expect("Failed to create temp dir"));
        let tmp_path = tmpdir.as_ref().unwrap().path().join("out.xml");
        {
            let tmp_path_str = tmp_path.as_path().to_str().expect("Temp path made no sense");
            let output = Command::new("C:\\dev\\rust\\cpptors\\gccxml_cc1plus.exe")
                .arg(format!("-fxml={}", tmp_path_str))
                .arg(input)
                .output()
                .expect("Failed to run gccxml");
            println!("stdout was {:?}", output);
            println!("XML stored at {}", tmp_path_str);
        }
        tmp_path
    };
    let f = File::open(xml_path).unwrap();
    let program: GccXml = serde_xml_rs::deserialize(f).unwrap();
    println!("Program is: {:?}", program);
    let mut feature_by_id = HashMap::new();
    for x in &program.features {
        feature_by_id.insert(x.get_id().clone(), x);
    }
    for x in &program.features {
        match x {
            CodeFeature::Function(f) => dump_function(&f, &feature_by_id),
            _ => {}
        }
    }
}

fn dump_function(f: &Function, feature_directory : &HashMap<String, &CodeFeature>) {
    println!("fn {}() -> {}", f.name, dump_type(&feature_directory[&f.returns]));
    println!("{{");
    for stmt in f.dump.as_ref().expect("No dump").body.statement_list.statements.iter() {
        println!("    {}", dump_statement(stmt, feature_directory));
    }
    println!("}}");
}

fn dump_statement<'a>(s: &'a Statement, _feature_directory : &HashMap<String, &CodeFeature>) -> &'a str {
    match s {
        Statement::ReturnStmt(_) => "return",
        _ => ""
    }
}

fn dump_type(cf: &CodeFeature) -> &str {
    match &cf {
        CodeFeature::FundamentalType(ft) => {
            &ft.name
        },
        _ => ""
    }
}