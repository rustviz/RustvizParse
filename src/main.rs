#[macro_use]
extern crate quote;
extern crate syn;

use core::fmt;
use std::{error::Error, string};
use std::fs::File;
use std::io::Read;
use log::{debug, error, log_enabled, info, Level};
// use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::io::Write;

use syn::Item;
use syn::Stmt;
use syn::Pat;
use syn::Expr;
use syn::FnArg;
use syn::Type;

#[derive(Debug, Hash, PartialEq, Eq)]
struct OwnerInfo {
    Name: Option<String>,
    Reference: bool,
    Mutability: bool
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct FuncInfo {
    Name: Option<String>
}


#[derive(Debug, Hash, PartialEq, Eq)]
enum RAP {
    Owner(OwnerInfo),
    MutRef(OwnerInfo),
    StaticRef(OwnerInfo),
    // Struct(),
    Function(FuncInfo),
}

fn path_fmt(exprpath : &syn::ExprPath) -> String {
    let mut pathname = "".to_owned();
    for seg in exprpath.path.segments.iter() {
        pathname.push_str(&seg.ident.to_string());
        pathname.push_str(&String::from("::"));
        // println!("{:?}", seg);
    }
    pathname[0..pathname.len()-2].to_string()
}

fn run() -> Result<(), Box<Error>> {    
    let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra2/main.rs")?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);

    let var_map = get_info(&ast);
    for i in &var_map {
        match i {
            RAP::Function(func) => {
                if func.Name != Some(String::from("main")) {
                    println!("Function {}()", func.Name.as_ref().unwrap());
                }
            },
            RAP::StaticRef(statref) => {
                println!("StaticRef {}", statref.Name.as_ref().unwrap());
            },
            RAP::MutRef(mutref) => {
                println!("MutRef {}", mutref.Name.as_ref().unwrap());
            },
            RAP::Owner(owner) => {
                if owner.Mutability {
                    println!("Owner mut {}", owner.Name.as_ref().unwrap());
                } else {
                    println!("Owner {}", owner.Name.as_ref().unwrap());
                }
            },
            _ => {
                println!("not implemented")
            }
        }
    }
    // create log file
    // if !Path::new("./parse_test.txt").exists() {
    //     println!("!exist");
    //     let mut f = File::create("./parse_test.txt")?;
    //     let ast_lit = format!("{:#?}", ast);
    //     f.write_all(ast_lit.as_bytes())?;
    // } else {
    //     let mut f = File::open("./parse_test.txt")?;
    //     f.set_len(0)?;
    //     // let ast_lit = format!("{:#?}", ast);
    //     // f.write_all(ast_lit.as_bytes())?;
    // }
    Ok(())
}

fn get_info(ast: &syn::File) -> HashSet<RAP> {
    let mut var_def = HashSet::new();
    //TODO: Scope analysis (ex. same variable name different scope)
    //TODO: separate different methods for parsing ownership in arguments and block expression??? draw diagrams
    for item in &ast.items {
        // TODO: rust's auto=deref...
        // options refer to https://docs.rs/syn/1.0.72/syn/enum.Item.html
        // TODO: macros, const, enums, structs... we only consider functions here
        match item {
            Item::Fn(func) => {
                debug!("func found: {}", func.sig.ident);
                var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}", func.sig.ident))}));
                if func.sig.inputs.len() != 0 {
                    for arg in &func.sig.inputs {
                        // info!("{:?}", arg); 
                        match arg {
                            FnArg::Typed(PatType) => {
                                let mut funcArg = OwnerInfo{Name: None, Mutability: false, Reference: false};
                                match &*PatType.pat {
                                    Pat::Ident(PatIdent) => {
                                    // TODO: We are assuming that the reference and mutability info are after colon??
                                        funcArg.Name=Some(String::from(format!("{}", PatIdent.ident)))
                                    },
                                    _ => info!("function arg name not supported")
                                }
                                //TODO: experiment on enum ownership... and if let ownership...
                                match &*PatType.ty {
                                    Type::Reference(TypeReference) => {
                                        funcArg.Reference = true;
                                        if let Some(mutability) = &TypeReference.mutability {
                                            funcArg.Mutability = true;
                                            var_def.insert(RAP::MutRef(funcArg));
                                        } else {
                                            var_def.insert(RAP::StaticRef(funcArg));
                                        }
                                    },
                                    Type::Path(_) => {
                                        var_def.insert(RAP::Owner(funcArg));
                                    }
                                    _ => info!("function arg type not supported")
                                }
                            },
                            _ => info!("receiver not supported")
                        }
                    }
                }
                //TODO: add function argument
                // func.sig.inputs
                for stmt in &func.block.stmts {
                    // local => let statement
                    // Item => function definition, struct definition etc.
                    // Expr => Expression without semicolon (return...)
                    // Semi => Expression with semicolon
                    match stmt {
                        Stmt::Local(loc) => {
                            // let statement
                            // save variable info
                            let mut local = OwnerInfo{Name: None, Mutability: false, Reference: false};

                            match &loc.pat {
                                Pat::Ident(patident) => {
                                    debug!("Owner found: {}, mutability: {:?}, ref: {:?}", patident.ident, patident.mutability, patident.by_ref);
                                    local.Name = Some(String::from(format!("{}", patident.ident)));
                                    // assume no 'ref' used here
                                    if let Some(mutable) = &patident.mutability {
                                        local.Mutability = true;
                                    }
                                },
                                Pat::Reference(PatReference) => {
                                    if let Pat::Ident(PatIdent) = &*PatReference.pat {
                                        debug!("Reference found: {}, mutability: {:?}", PatIdent.ident, PatReference.mutability);
                                        local.Name = Some(String::from(format!("{}", PatIdent.ident)));
                                        if let Some(mutable) = &PatReference.mutability {
                                            local.Mutability = true;
                                        }
                                    }
                                }
                                //TODO: add Struct(PatStruct),
                                _ => info!("stmt not supported")
                            }
                            //if assigned
                            if let Some((eq, expr)) = &loc.init {
                                //TODO: only consider functions and refs
                                // what's the pattern?
                                // let mut num: () = expr;
                                match &**expr {
                                    Expr::Call(exprcall) => {
                                        if let Expr::Path(exprpath) = &*exprcall.func {
                                            //TODO: WTF is '&*'
                                            // println!("func found: {:?}", exprpath);
                                            debug!("func found: {}", path_fmt(&exprpath));
                                            var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}", path_fmt(&exprpath)))}));
                                        }
                                    },
                                    Expr::Reference(expred) => {
                                        debug!("Owner's a reference: {:?}", expred.mutability);
                                        local.Reference = true;
                                        if let Some(mutable) = &expred.mutability {
                                            local.Mutability = true;
                                        }
                                        if let Expr::Path(exprpath) = &*expred.expr {
                                            // println!("Ref target: {:?}", exprpath);
                                            debug!(" Ref target: {}", path_fmt(&exprpath));
                                        }
                                    },
                                    // do not care other right side experssion
                                    _ => info!("expr not supported")
                                }
                            }
                            if local.Reference {
                                if local.Mutability {
                                    var_def.insert(RAP::MutRef(local));
                                } else {
                                    var_def.insert(RAP::StaticRef(local));
                                }
                            } else {
                                var_def.insert(RAP::Owner(local));
                            }
                        },
                        Stmt::Semi(exp, semi) => {
                            info!("{:?}", exp);
                            info!("{:?}", semi);
                            // excution of function, no related owner/function info
                        }, 
                        _ => {
                            //TODO: expressions and extra item definition not supported, should be written recursively
                            info!("Expression (control flow) and Item definition not supported")
                        }
                    }
                }
            },
            _ => info!("Item definition not supported")
        }
    }
    var_def
}

fn main() {
    env_logger::init();
    let parse_res = run();
}