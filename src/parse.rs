use syn::{Stmt, Expr, Pat, Item, FnArg, Type};
use log::{debug, info};
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use syn::spanned::Spanned;

static mut var_def: HashSet<RAP> = HashSet::new(); 
static mut color_info: Vec<HashMap<String, Vec<Infoitem>>> = Vec::new();

#[derive(Debug, Hash, PartialEq, Eq)]
struct OwnerInfo {
    name: Option<String>, 
    is_ref: bool,
    ref_mut: bool, 
    field: Option<Vec<String>>, // for structs only
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct FuncInfo {
    name: Option<String>
}

#[derive(Debug, Hash, PartialEq, Eq)]
enum RAP {
    Owner(OwnerInfo),
    MutRef(OwnerInfo),
    StaticRef(OwnerInfo),
    Struct(OwnerInfo),
    Function(FuncInfo),
}

enum Infoitem {
    Struct(syn::ItemStruct), // struct definition
    Func(syn::ItemFn),
    FnArg(syn::FnArg),
    Local(syn::Local),
    Call(syn::ExprCall),
    MethodCall(syn::ExprMethodCall),
    is_ref(syn::ExprReference),
    ExprStruct(syn::ExprStruct), // struct literal expression
    Macro(syn::ExprMacro)
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

pub fn parse(FileName : &PathBuf) -> Result<String, Box<Error>> {    
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    let mut file = File::open(FileName)?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);
    let header = str_gen(ast);
    Ok(header)
}

fn color_gen() {
}

fn str_gen(ast: syn::File) -> String {
    // generate header lines 
    color_info.push(HashMap::new());
    parse_item(&ast.items, 0);
    let mut header = String::new();
    header.push_str("/* --- BEGIN Variable Definitions ---\n");
    for i in var_def {
        match i {
            RAP::Function(func) => {
                if func.name != Some(String::from("main")) {
                    header.push_str(&format!("Function {}();\n", func.name.as_ref().unwrap()));
                }
            },
            RAP::StaticRef(statref) => {
                header.push_str(&format!("StaticRef {};\n", statref.name.as_ref().unwrap()));
            },
            RAP::MutRef(mutref) => {
                header.push_str(&format!("MutRef {};\n", mutref.name.as_ref().unwrap()));
            },
            RAP::Owner(owner) => {
                if owner.ref_mut {
                    header.push_str(&format!("Owner mut {};\n", owner.name.as_ref().unwrap()));
                } else {
                    header.push_str(&format!("Owner {};\n", owner.name.as_ref().unwrap()));
                }
            },
            RAP::Struct(stut) => {
                // Struct r{w, h};
                header.push_str(&format!("Struct {}{{", stut.name.as_ref().unwrap()));
                for i in stut.field.unwrap() {
                    header.push_str(&format!("{},",i));
                }
                header.pop();
                header.push_str("};\n");
            }
            _ => {
                println!("not implemented")
            }
        }
    }
    header.push_str("--- END Variable Definitions --- */\n");
    header
}

fn color_insert(ident: String, ident_info: Infoitem, stack_num: usize) {
    match color_info[stack_num].get_mut(&ident) {
        Some(var_map) => {
            var_map.push(ident_info);
        },
        None => {
            color_info[stack_num].insert(ident, vec![ident_info]);
        }
    }
}

fn parse_item (items: &Vec<syn::Item>, stack_num: usize) {
    for item in items {
        // All items refer to https://docs.rs/syn/1.0.72/syn/enum.Item.html
        // TODO: macros, const, enums, structs... we only consider functions and struct here
        // TODO: Add errors???
        match *item {
            Item::Fn(func) => {
                debug!("--------------");
                debug!("func found: {}", func.sig.ident);
                debug!("{:?}", func.span().start());
                debug!("{:?}", func.span().end());
                debug!("--------------");
                // push stack and register func into color_info
                color_info.push(HashMap::new());
                color_insert(format!("{}", func.sig.ident), Infoitem::Func(func), stack_num);
                // register func into var_def
                var_def.insert(RAP::Function(FuncInfo{name: Some(format!("{}", func.sig.ident))}));
                if func.sig.inputs.len() != 0 {
                    // match arguments
                    // create new stack for func arg
                    color_info.push(HashMap::new());
                    for arg in &func.sig.inputs {
                        match *arg {
                            FnArg::Typed(pat_type) => {
                                // match arg type
                                let mut func_arg = OwnerInfo{name: None, ref_mut: false, is_ref: false, field: None};
                                debug!("--------------");
                                match &*pat_type.pat {
                                    Pat::Ident(pat_ident) => {
                                        // push var into stack
                                        color_insert(format!("{}", pat_ident.ident), Infoitem::FnArg(arg), stack_num+1);
                                        // TODO: We are assuming that the is_ref and ref_mut info are after colon??
                                        func_arg.name=Some(String::from(format!("{}", pat_ident.ident)));
                                        debug!("arg found: {:?}", func_arg.name);
                                    },
                                    _ => info!("function arg name not supported")
                                }
                                debug!("{:?}", pat_type.span().start());
                                debug!("{:?}", pat_type.span().end());
                                debug!("--------------");
                                match &*pat_type.ty {
                                    Type::Reference(type_reference) => {
                                        func_arg.is_ref = true;
                                        if let Some(_mutability) = &type_reference.mutability {
                                            func_arg.ref_mut = true;
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
                            _ => info!("syn::Receiver <self> not supported")
                        }
                    }
                }
                // parse function block
                for stmt in &func.block.stmts {
                    parse_stmt(&stmt, stack_num+1);
                }
            },
            Item::Struct(ItemStruct) => {
                // push struct dec into stack
                color_insert(format!("{}", ItemStruct.ident), Infoitem::Struct(ItemStruct), stack_num);
            },
            _ => info!("syn::Item option not supported")
        }
    }
    // var_def
}

fn parse_stmt(stmt: &syn::Stmt, stack_num: usize) {
    // local => let statement
    // Item => function definition, struct definition etc.
    // Expr => Expression without semicolon (return...)
    // Semi => Expression with semicolon
    debug!("--------------");
    debug!("stmt found");
    let mut local = OwnerInfo{name: None, ref_mut: false, is_ref: false, field: None};
    // TODO: local is acting as a placeholder for Expr, Item and Semi, need more elegent impl for parse_expr()...
    match *stmt {
        Stmt::Local(loc) => {
            // let statement
            // save variable info
            match &loc.pat {
                //TODO: Need to consider object type after the eq sign
                // eg: let a = &5; -> a is a is_ref type
                // let a = 5 -> a is not a is_ref type
                // we assume that a is determined by the rhs of the eq whether it's an 'explict' is_ref
                Pat::Ident(pat_ident) => {
                    debug!("Owner found: {}, ref_mut: {:?}, ref: {:?}", pat_ident.ident, pat_ident.mutability, pat_ident.by_ref);
                    debug!("{:?}", pat_ident.ident.span().start());
                    debug!("{:?}", pat_ident.ident.span().end());
                    color_insert(format!("{}", pat_ident.ident), Infoitem::Local(loc), stack_num);
                    local.name = Some(String::from(format!("{}", pat_ident.ident)));
                    // var ref_mut not is_ref ref_mut
                    if let Some(_mutable) = &pat_ident.mutability {
                        local.ref_mut = true;
                    }
                },
                //TODO: Need to consider object type after the eq sign (is this common? where is this useful?)
                // eg: let &a = &5 -> a is not a is_ref type
                // let &a = &&5 ???
                // we could have let &mut a = &mut 5; let &mut mut a = &mut 5; let &a = &5
                // why can't we have let mut &a = &5
                // Pat::is_ref(pat_reference) => {
                //     if let Pat::Ident(pat_ident) = &*pat_reference.pat {
                //         debug!("is_ref found: {}, ref_mut: {:?}", pat_ident.ident, pat_reference.ref_mut);
                //         local.name = Some(String::from(format!("{}", pat_ident.ident)));
                //         color_insert(format!("{}", pat_ident.ident), Infoitem::Local(loc), stack_num);
                //         if let Some(_mutable) = &pat_reference.ref_mut {
                //             local.ref_mut = true;
                //         }
                //     }
                // },

                //TODO: Need to consider object type after the eq sign
                // eg: let mut a : &mut i32 = &mut 5; -> a is a is_ref type
                // let &mut a : &mut i32 = &mut 5; -> a is not a is_ref type
                // we assume that all let statement : let foo:i32 = 5;
                Pat::Type(pat_type) => {
                    if let Pat::Ident(pat_ident) = *pat_type.pat {
                        local.name = Some(String::from(format!("{}", pat_ident.ident)));
                        color_insert(format!("{}", pat_ident.ident), Infoitem::Local(loc), stack_num);
                    }
                    if let Type::Reference(type_ref) = *pat_type.ty {
                        local.is_ref = true;
                        if let Some(_mutable) = type_ref.mutability {
                            local.ref_mut = true;
                        }
                    }
                },
                _ => info!("stmt not supported")
            }

            //if a value or a is_ref is assigned
            if let Some((_eq, expr)) = &loc.init {
                parse_expr(expr, &mut local, stack_num);
            }

            match local.field {
                Some(_) => {
                    var_def.insert(RAP::Struct(local));
                },
                _ => {
                    if local.is_ref {
                        if local.ref_mut {
                            var_def.insert(RAP::MutRef(local));
                        } else {
                            var_def.insert(RAP::StaticRef(local));
                        }
                    } else {
                        var_def.insert(RAP::Owner(local));
                    }
                }
            }
        },
        Stmt::Semi(exp, _) => {
            parse_expr(&exp, &mut local, stack_num);
            info!("{:?}", exp);
            //TODO: deal with semis?
        }, 
        Stmt::Expr(exp) => {
            parse_expr(&exp, &mut local, stack_num);
            info!("{:?}", exp);
        },
        Stmt::Item(item) => {
            parse_item(vec![item], stack_num);
        }
    }
    debug!("{:?}", stmt.span().start());
    debug!("{:?}", stmt.span().end());
    debug!("--------------");
}

fn parse_expr (expr: &syn::Expr, local: &mut OwnerInfo, stack_num: usize) {
    debug!("--------------");
    debug!("expr found");
    match *expr {
        Expr::Call(exprcall) => {
            if let Expr::Path(exprpath) = &*exprcall.func {
                // println!("func found: {:?}", exprpath);
                var_def.insert(RAP::Function(FuncInfo{name: Some(format!("{}", path_fmt(&exprpath)))}));
                color_insert(format!("{}", path_fmt(&exprpath)), Infoitem::Call(exprcall), stack_num);
            }
        },
        Expr::MethodCall(exprm_call) => {
            if let m_call = String::from(format!("{}", exprm_call.method)) {
                debug!("func found: {}",  m_call);
                var_def.insert(RAP::Function(FuncInfo{name: Some(format!("{}",  m_call))}));
                color_insert(format!("{}", m_call), Infoitem::MethodCall(exprm_call), stack_num);
            }
        },
        Expr::Reference(expred) => {
            debug!("Owner's a is_ref: {:?}", expred.mutability);
            local.is_ref = true;
            if let Some(_mutable) = &expred.mutability {
                local.ref_mut = true;
            }
            if let Expr::Path(exprpath) = &*expred.expr {
                // println!("Ref target: {:?}", exprpath);
                debug!(" Ref target: {}", path_fmt(&exprpath));
            }
        },
        Expr::Block(expr_block) => {
            debug!("found block");
            for stmt in &expr_block.block.stmts {
                parse_stmt(&stmt, stack_num+1);
            }
        },
        Expr::Struct(expr_struct) => {
            debug!("found struct");
            let mut field_vec = Vec::new();
            for i in &expr_struct.fields {
                match &i.member {
                    syn::Member::Named(Ident) => {
                        field_vec.push(format!("{}",Ident));
                    }
                    _ => {
                        //TODO: do not take care of pair struct
                    }
                }
            }
            local.field = Some(field_vec);
            color_insert(format!("{}", expr_struct.path.segments[expr_struct.path.segments.len()-1].ident), Infoitem::ExprStruct(expr_struct), stack_num);
        },
        Expr::Macro(_macro) => {
            debug!("found macro");
            let macro_path = &_macro.mac.path;
            if let Some(macro_func) = macro_path.segments.first() {
                debug!("found {}", macro_func.ident);
                //TODO: only consider Println and assert here
                color_insert(format!("{}", macro_func.ident), Infoitem::Macro(_macro), stack_num);
                if macro_func.ident.to_string() == "println" {
                    var_def.insert(RAP::Function(FuncInfo{name: Some(format!("{}!", macro_func.ident))}));
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
                                        Ok(exp) => parse_expr(&exp, local, stack_num),
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
                        Ok(exp) => parse_expr(&exp, local, stack_num),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                } else if macro_func.ident.to_string() == "assert" {
                    let res: Result<syn::Expr, syn::Error> = syn::parse2(_macro.mac.tokens.clone());
                    match res {
                        Ok(exp) => parse_expr(&exp, local, stack_num),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                    // parse_expr(res, local, var_def);
                } else {
                    info!("macro type not supported")
                }
            }
        },
        // do not care other right side experssion
        _ => info!("expr not supported")
    }
    debug!("{:?}", expr.span().start());
    debug!("{:?}", expr.span().end());
    debug!("--------------");
}