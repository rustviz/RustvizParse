// #[macro_use]
extern crate quote;
extern crate syn;
extern crate clap;
extern crate proc_macro2;

// use core::fmt;
// use std::error::String;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use log::{debug, info};
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
use std::iter::FromIterator;


use proc_macro2::TokenStream;

use clap::{Arg, App, SubCommand};

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

fn run(FileName : &Path) -> Result<(), Box<Error>> {    
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    let mut file = File::open(FileName)?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);

    let mut var_map = HashSet::new();
    get_info(&ast, &mut var_map);

    let path = Path::new("./test.txt");
    let display = path.display();
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };
    file.write_all("/* --- BEGIN Variable Definitions ---\n".as_bytes());

    for i in &var_map {
        match i {
            RAP::Function(func) => {
                if func.Name != Some(String::from("main")) {
                    println!("Function {}()", func.Name.as_ref().unwrap());
                    file.write_all(format!("Function {}();\n", func.Name.as_ref().unwrap()).as_bytes());
                }
            },
            RAP::StaticRef(statref) => {
                println!("StaticRef {}", statref.Name.as_ref().unwrap());
                file.write_all(format!("StaticRef {};\n", statref.Name.as_ref().unwrap()).as_bytes());
            },
            RAP::MutRef(mutref) => {
                println!("MutRef {}", mutref.Name.as_ref().unwrap());
                file.write_all(format!("MutRef {};\n", mutref.Name.as_ref().unwrap()).as_bytes());
            },
            RAP::Owner(owner) => {
                if owner.Mutability {
                    println!("Owner mut {}", owner.Name.as_ref().unwrap());
                    file.write_all(format!("Owner mut {};\n", owner.Name.as_ref().unwrap()).as_bytes());
                } else {
                    println!("Owner {}", owner.Name.as_ref().unwrap());
                    file.write_all(format!("Owner {};\n", owner.Name.as_ref().unwrap()).as_bytes());
                }
            },
            _ => {
                println!("not implemented")
            }
        }
    }
    file.write_all("--- END Variable Definitions --- */\n".as_bytes());


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

fn parse_expr (expr: &syn::Expr, local: &mut OwnerInfo, var_def: &mut HashSet<RAP>) {
    match expr {
        Expr::Call(exprcall) => {
            if let Expr::Path(exprpath) = &*exprcall.func {
                //TODO: WTF is '&*'
                // println!("func found: {:?}", exprpath);
                debug!("func found: {}", path_fmt(&exprpath));
                var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}", path_fmt(&exprpath)))}));
            }
        },
        Expr::MethodCall(exprm_call) => {
            if let m_call = String::from(format!("{}", exprm_call.method)) {
                debug!("func found: {}",  m_call);
                var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}",  m_call))}));
            }
        },
        Expr::Reference(expred) => {
            debug!("Owner's a reference: {:?}", expred.mutability);
            local.Reference = true;
            if let Some(_mutable) = &expred.mutability {
                local.Mutability = true;
            }
            if let Expr::Path(exprpath) = &*expred.expr {
                // println!("Ref target: {:?}", exprpath);
                debug!(" Ref target: {}", path_fmt(&exprpath));
            }
        },
        Expr::Block(expr_block) => {
            debug!("found block");
            for stmt in &expr_block.block.stmts {
                parse_stmt(&stmt, var_def);
            }
        },
        Expr::Macro(_macro) => {
            debug!("found macro");
            let macro_path = &_macro.mac.path;
            if let Some(macro_func) = macro_path.segments.first() {
                debug!("found {}", macro_func.ident);
                //TODO: only consider Println and assert here
                if (macro_func.ident.to_string() == "println") {
                    var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}", macro_func.ident))}));
                    let mut tokentree_buff = Vec::new();
                    let mut first_lit = false;
                    for item in _macro.mac.tokens.clone() {
                        debug!("{:?}",item);
                        match item {
                            proc_macro2::TokenTree::Punct(punct) => {
                                if (!first_lit) {
                                    // dump "{:?...}" in println!()
                                    tokentree_buff.clear();
                                    first_lit = true;
                                } else {
                                    let mut tokenstream_buff = proc_macro2::TokenStream::new();
                                    tokenstream_buff.extend(tokentree_buff);
                                    let res: Result<syn::Expr, syn::Error> = syn::parse2(tokenstream_buff);
                                    match res {
                                        Ok(exp) => parse_expr(&exp, local, var_def),
                                        Err(_) => debug!("Assert macro parse error"),
                                    }
                                    tokentree_buff = Vec::new();
                                }
                            }
                            _ => {
                                tokentree_buff.push(item);
                            }
                        }
                    }
                    let mut tokenstream_buff = proc_macro2::TokenStream::new();
                    tokenstream_buff.extend(tokentree_buff);
                    let res: Result<syn::Expr, syn::Error> = syn::parse2(tokenstream_buff);
                    match res {
                        Ok(exp) => parse_expr(&exp, local, var_def),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                } else {
                    let res: Result<syn::Expr, syn::Error> = syn::parse2(_macro.mac.tokens.clone());
                    match res {
                        Ok(exp) => parse_expr(&exp, local, var_def),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                    // parse_expr(res, local, var_def);
                }
            }
        },
        // do not care other right side experssion
        _ => info!("expr not supported")
    }
}

fn parse_stmt(stmt: &syn::Stmt, var_def: &mut HashSet<RAP>) {
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
                Pat::Ident(pat_ident) => {
                    debug!("Owner found: {}, mutability: {:?}, ref: {:?}", pat_ident.ident, pat_ident.mutability, pat_ident.by_ref);
                    local.Name = Some(String::from(format!("{}", pat_ident.ident)));
                    // assume no 'ref' used here
                    if let Some(_mutable) = &pat_ident.mutability {
                        local.Mutability = true;
                    }
                },
                Pat::Reference(pat_reference) => {
                    if let Pat::Ident(pat_ident) = &*pat_reference.pat {
                        debug!("Reference found: {}, mutability: {:?}", pat_ident.ident, pat_reference.mutability);
                        local.Name = Some(String::from(format!("{}", pat_ident.ident)));
                        if let Some(_mutable) = &pat_reference.mutability {
                            local.Mutability = true;
                        }
                    }
                }
                //TODO: add Struct(PatStruct),
                _ => info!("stmt not supported")
            }
            //if assigned
            if let Some((_eq, expr)) = &loc.init {
                //TODO: only consider functions and refs
                // what's the pattern?
                // let mut num: () = expr;
                parse_expr(expr, &mut local, var_def);
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
        Stmt::Semi(exp, _semi) => {
            // match exp {
            //     Expr::Macro(_macro) => {
            //         debug!("found macro");
            //         let macro_path = &_macro.mac.path;
            //         if let Some(macro_func) = macro_path.segments.first() {
            //             debug!("found {}", macro_func.ident);
            //             var_def.insert(RAP::Function(FuncInfo{Name: Some(format!("{}", macro_func.ident))}));
            //         }
            //         let input = _macro.mac.tokens.clone();
            //         let mut content = "fn main(){".to_owned();
            //         content.push_str(&input.to_string());
            //         content.push_str(&"}".to_owned());
            //         debug!("found macro statement: *{}*", content);
            //         let sub_ast = syn::parse_file(&content).unwrap();
            //         debug!("{:#?}", sub_ast);
            //         get_info(&sub_ast, var_def);
            //     }
            //     _ => {
            //         debug!("not supported");
            //     }
            // }
            let mut local = OwnerInfo{Name: None, Mutability: false, Reference: false};
            parse_expr(exp, &mut local, var_def);
            info!("{:?}", exp);
            //TODO: deal with semis?
        }, 
        Stmt::Expr(exp) => {
            let mut local = OwnerInfo{Name: None, Mutability: false, Reference: false};
            parse_expr(exp, &mut local, var_def);
            info!("{:?}", exp);
        },
        _ => {
            //TODO: expressions and extra item definition not supported, should be written recursively
            info!("Expression (control flow) and Item definition not supported")
        }
    }
}

fn get_info(ast: &syn::File, var_def: &mut HashSet<RAP>) {
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
                            FnArg::Typed(pat_type) => {
                                let mut func_arg = OwnerInfo{Name: None, Mutability: false, Reference: false};
                                match &*pat_type.pat {
                                    Pat::Ident(pat_ident) => {
                                    // TODO: We are assuming that the reference and mutability info are after colon??
                                        func_arg.Name=Some(String::from(format!("{}", pat_ident.ident)))
                                    },
                                    _ => info!("function arg name not supported")
                                }
                                //TODO: experiment on enum ownership... and if let ownership...
                                match &*pat_type.ty {
                                    Type::Reference(type_reference) => {
                                        func_arg.Reference = true;
                                        if let Some(_mutability) = &type_reference.mutability {
                                            func_arg.Mutability = true;
                                            var_def.insert(RAP::MutRef(func_arg));
                                        } else {
                                            var_def.insert(RAP::StaticRef(func_arg));
                                        }
                                    },
                                    Type::Path(_) => {
                                        var_def.insert(RAP::Owner(func_arg));
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
                    parse_stmt(&stmt, var_def);
                }
            },
            _ => info!("Item definition not supported")
        }
    }
    // var_def
}

fn main() {
    env_logger::init();
    let matches = App::new("Rustviz Parse")
                        //   .version("1.0")
                        //   .author("Kevin K. <kbknapp@gmail.com>")
                          .about("Parse Owners and Functions")
                          .arg(Arg::with_name("target")
                            //    .short("t")
                            //    .long("target")
                            //    .value_name("FILE")
                               .help("Target file for parsing")
                               .required(true)
                               .takes_value(true))
                        //   .arg(Arg::with_name("INPUT")
                        //        .help("Sets the input file to use")
                        //        .required(true)
                        //        .index(1))
                          .get_matches();
    let file_name = Path::new(matches.value_of("target").unwrap());
    // println!("{:?}", FileName);
    let parse_res = run(file_name);
}