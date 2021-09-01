use syn::{Stmt, Expr, Pat, Item, FnArg, Type};
use log::{debug, info};
use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use syn::spanned::Spanned;

// TODO: Global variable? Stack allocated...
// static mut var_def: HashSet<RAP> = HashSet::new(); 
// static mut color_info: Vec<HashMap<String, Vec<Infoitem>>> = Vec::new();
static mut hash_num : i32 = 0;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct OwnerInfo {
    name: Option<String>, 
    is_ref: bool,
    ref_mut: bool, 
    field: Option<Vec<String>>, // for structs only
    hash_id: i32,
}

// impl OwnerInfo {
//     fn get_identstr(&self) -> String {
//         self.name.unwrap()
//     }
// }

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FuncInfo {
    name: Option<String>,
    hash_id: i32,
}

// impl FuncInfo {
//     fn get_indentstr(&self) -> String {
//         self.name.unwrap()
//     }
// }

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum RAP {
    Owner(OwnerInfo),
    MutRef(OwnerInfo),
    StaticRef(OwnerInfo),
    Struct(OwnerInfo),
    Function(FuncInfo),
}

fn get_identstr(rap: &RAP) -> String {
    match rap {
        RAP::Owner(OwnerInfo) => {
            OwnerInfo.name.unwrap()
        }
        RAP::MutRef(OwnerInfo) => {
            OwnerInfo.name.unwrap()
        }
        RAP::StaticRef(OwnerInfo) => {
            OwnerInfo.name.unwrap()
        }
        RAP::Struct(OwnerInfo) => {
            OwnerInfo.name.unwrap()
        }
        RAP::Function(FuncInfo) => {
            FuncInfo.name.unwrap()
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct StackItem <'a> {
    SynInfo: Infoitem,
    ItemOrig: Option<&'a RAP>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Infoitem {
    Struct(syn::ItemStruct), // struct definition
    Func(syn::ItemFn),
    FnArg(syn::FnArg),
    Local(syn::Local), //let a = 5;
    Call(syn::ExprCall), // func_cal();
    MethodCall(syn::ExprMethodCall), // a.to_string();
    Reference(syn::ExprReference), // &a;
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

pub fn syn_parse(FileName : &PathBuf) -> Result<(HashMap<String, Vec<RAP>>, Vec<HashMap<String, Vec<StackItem>>>), Box<Error>> {    
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    let mut file = File::open(FileName)?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);
    let mut color_info: Vec<HashMap<String, Vec<StackItem>>> = Vec::new();
    let mut var_def: HashMap<String, Vec<RAP>> = HashMap::new(); 
    color_info.push(HashMap::new());
    parse_item(&ast.items, &mut color_info, &mut var_def, 0);
    // color_gen(&color_info);
    Ok((var_def, color_info))
}

fn color_gen(color_info: &Vec<HashMap<String, Vec<Infoitem>>>) {
    // For debug purposes:
    // for i in color_info {
    //     println!("In Scope: ");
    //     println!("----------");
    //     for (key, value) in i {
    //         println!("Ident: ");
    //         println!("  {}", key);
    //         println!("Type/Content : ");
    //         for i in value {
    //             println!("{:?}", i);
    //             println!("");
    //         }
    //         println!("")
    //     }
    //     println!("----------");
    // }
    
}

pub fn header_gen_str(var_def: &HashSet<RAP>) -> String {
    // generate header lines 
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
                for i in stut.field.as_ref().unwrap() {
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

fn var_allo_insert<'a>(ident: String, ident_info: StackItem, 
    target_rap: RAP,
    color_info: &'a mut Vec<HashMap<String, Vec<StackItem>>>,
    var_def: &'a HashMap<String, Vec<RAP>>,
    stack_num: usize) {
    // variable initialization happened -> 
    // look for RAP, if exist then add shadowing variable
    // if not then add Ident_info to color_info
    // called for the following InfoItem:
    // ----------------------------------
    // Struct(syn::ItemStruct)
    // Func(syn::ItemFn)
    // FnArg(syn::FnArg)
    // Local(syn::Local)
    // Macro(syn::ExprMacro) 
    // ----------------------------------
    ident_info.ItemOrig = Some(&target_rap);
    if var_def.contains_key(&get_identstr(&target_rap)) {
        // add shadow RAP
        var_def[&get_identstr(&target_rap)].push(target_rap);
    } else {
        // add RAP
        var_def.insert(get_identstr(&target_rap), vec![target_rap]);
    }
    // push into stack
    match color_info[stack_num].get_mut(&ident) {
        Some(var_map) => {
            var_map.push(ident_info);
        },
        None => {
            color_info[stack_num].insert(ident, vec![ident_info]);
        }
    }
}

fn non_allo_insert(ident: String, ident_info: StackItem, 
    target_rap: Option<RAP>,
    color_info: &mut Vec<HashMap<String, Vec<StackItem>>>, 
    var_def: &HashMap<String, Vec<RAP>>,
    stack_num: usize) {
    // variable initialization did not happen -> 
    // search stack for reference to RAP
    // search upon the nearest stack with the same ident -> 
    // if multiple item correspond to one ident then choose the last
    // not found then add RAP and call var_allo_insert()
    // called for the following InfoItem:
    // ----------------------------------
    // Call(syn::ExprCall), // func_cal();
    // MethodCall(syn::ExprMethodCall), // a.to_string();
    // Reference(syn::ExprReference), // &a;
    // ----------------------------------

}

//TODO: do I need to specify same lifetime for color_info and var_def
fn parse_item (items: &Vec<syn::Item>, 
    color_info: &mut Vec<HashMap<String, Vec<StackItem>>>, 
    var_def: &mut HashMap<String, Vec<RAP>>, 
    stack_num: usize) {
    for item in items {
        // All items refer to https://docs.rs/syn/1.0.72/syn/enum.Item.html
        // TODO: macros, const, enums, structs... we only consider functions and struct here
        // TODO: Add errors???

        // TODO: *item and func.clone / item and *func.clone 
        // match take ownership???
        // clone of a reference??? or of the variable itself?
        match item {
            Item::Fn(func) => {
                debug!("--------------");
                debug!("func found: {}", func.sig.ident);
                debug!("{:?}", func.span().start());
                debug!("{:?}", func.span().end());
                debug!("--------------");
                // register func into var_def
                let func_rap = RAP::Function(FuncInfo{name: Some(format!("{}", func.sig.ident))});
                // push stack and register func into color_info
                color_info.push(HashMap::new());
                var_allo_insert(format!("{}", func.sig.ident), 
                StackItem {
                    SynInfo: Infoitem::Func(func.clone()),
                    ItemOrig: None,
                }, func_rap,
                color_info, var_def, stack_num);

                if func.sig.inputs.len() != 0 {
                    // match arguments
                    // create new stack for func arg
                    for arg in &func.sig.inputs {
                        match arg {
                            FnArg::Typed(pat_type) => {
                                // match arg type
                                let mut func_arg = OwnerInfo{name: None, ref_mut: false, is_ref: false, field: None};
                                debug!("--------------");
                                // extract arg ident
                                match &*pat_type.pat {
                                    Pat::Ident(pat_ident) => {
                                        // push var into stack
                                        func_arg.name=Some(String::from(format!("{}", pat_ident.ident)));
                                        debug!("arg found: {:?}", func_arg.name);
                                    },
                                    _ => info!("function arg name not supported")
                                }
                                debug!("{:?}", pat_type.span().start());
                                debug!("{:?}", pat_type.span().end());
                                debug!("--------------");
                                // extract arg type
                                let arg_rap;
                                match &*pat_type.ty {
                                    Type::Reference(type_reference) => {
                                        func_arg.is_ref = true;
                                  
                                        if let Some(_mutability) = &type_reference.mutability {
                                            func_arg.ref_mut = true;
                                            arg_rap = RAP::MutRef(func_arg);

                                        } else {
                                            arg_rap = RAP::StaticRef(func_arg);
                                        }
                                    },
                                    Type::Path(_) => {
                                        arg_rap = RAP::Owner(func_arg);
                                    }
                                    _ => info!("function arg type not supported")
                                }
                                var_allo_insert(func_arg.name.unwrap(), 
                                StackItem {
                                    SynInfo: Infoitem::FnArg(arg.clone()),
                                    ItemOrig: None,
                                }, arg_rap,
                                color_info, var_def, stack_num+1);
                            },
                            _ => info!("syn::Receiver <self> not supported")
                        }
                    }
                }
                // parse function block
                for stmt in &func.block.stmts {
                    parse_stmt(&stmt, &mut color_info, var_def, stack_num+1);
                }
            },
            Item::Struct(ItemStruct) => {
                //TODO: take care of struct later 
                // push struct dec into stack
                // var_allo_insert(format!("{}", ItemStruct.ident), 
                // StackItem {
                //     SynInfo: Infoitem::FnArg(arg.clone()),
                //     ItemOrig: &arg_rap,
                // },
                // Infoitem::Struct(ItemStruct.clone()), 
                // color_info, stack_num);
            },
            _ => info!("syn::Item option not supported")
        }
    }
    // var_def
}

fn parse_stmt(stmt: &syn::Stmt, 
    color_info: &mut Vec<HashMap<String, Vec<StackItem>>>, 
    var_def: &mut HashMap<String, Vec<RAP>>, 
    stack_num: usize) {
    // local => let statement
    // Item => function definition, struct definition etc.
    // Expr => Expression without semicolon (return...)
    // Semi => Expression with semicolon
    debug!("--------------");
    debug!("stmt found");
    let mut local = OwnerInfo{name: None, ref_mut: false, is_ref: false, field: None};

    match stmt {
        Stmt::Local(loc) => {
            // let statement
            // save variable info
            match &loc.pat {
                //TODO: Need to consider object type after the eq sign
                // eg: let a = &5; -> a is a is_ref type
                // let a = 5 -> a is not a is_ref type
                // let &a = &&5;
                // let a = *&&5;
                // we assume that a is determined by the rhs of the eq whether it's an 'explict' is_ref
                Pat::Ident(pat_ident) => {
                    debug!("Owner found: {}, ref_mut: {:?}, ref: {:?}", pat_ident.ident, pat_ident.mutability, pat_ident.by_ref);
                    debug!("{:?}", pat_ident.ident.span().start());
                    debug!("{:?}", pat_ident.ident.span().end());
                    local.name = Some(String::from(format!("{}", pat_ident.ident)));
                    // var ref_mut not is_ref ref_mut
                    if let Some(_mutable) = &pat_ident.mutability {
                        local.ref_mut = true;
                    }
                },
                //TODO: Need to consider object type after the eq sign (is this common? where is this useful?)
                // eg: let &a = &5 -> a is not a is_ref type
                // CRYUS: let &a = &&5 ???
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
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        local.name = Some(String::from(format!("{}", pat_ident.ident)));
                    }
                    if let Type::Reference(type_ref) = &*pat_type.ty {
                        local.is_ref = true;
                        if let Some(_mutable) = type_ref.mutability {
                            local.ref_mut = true;
                        }
                    }
                    //TODO: add struct here
                },
                _ => info!("stmt not supported")
            }

            //if a value or a is_ref is assigned
            if let Some((_eq, expr)) = &loc.init {
                parse_expr(expr, &mut local, color_info, var_def, stack_num);
            }
            
            let local_def;
            match local.field {
                Some(_) => {
                    local_def = RAP::Struct(local)
                },
                _ => {
                    if local.is_ref {
                        if local.ref_mut {
                            local_def = RAP::MutRef(local);
                        } else {
                            local_def = RAP::StaticRef(local)
                        }
                    } else {
                        local_def = RAP::Owner(local);
                    }
                }
            }
            var_allo_insert(local.name.unwrap(), 
            StackItem {
                SynInfo: Infoitem::Local(loc.clone()),
                ItemOrig: None,
            },local_def,
            color_info, var_def, stack_num);
        },
        Stmt::Semi(exp, _) => {
            parse_expr(&exp, &mut local, color_info, var_def, stack_num);
            info!("{:?}", exp);
        }, 
        Stmt::Expr(exp) => {
            parse_expr(&exp, &mut local, color_info, var_def, stack_num);
            info!("{:?}", exp);
        },
        Stmt::Item(item) => {
            parse_item(&vec![item.clone()], color_info, var_def, stack_num);
        }
    }
    debug!("{:?}", stmt.span().start());
    debug!("{:?}", stmt.span().end());
    debug!("--------------");
}

fn parse_expr (expr: &syn::Expr, 
    local: &mut OwnerInfo, 
    color_info: &mut Vec<HashMap<String, Vec<StackItem>>>, 
    var_def: &mut HashMap<String, Vec<RAP>>, 
    stack_num: usize) {

    debug!("--------------");
    debug!("expr found");

    match expr {
        Expr::Call(exprcall) => {
            if let Expr::Path(exprpath) = &*exprcall.func {
                // println!("func found: {:?}", exprpath);
                let call_rap = RAP::Function(FuncInfo{name: Some(format!("{}", path_fmt(&exprpath)))});
                non_allo_insert(format!("{}", path_fmt(&exprpath)),
                StackItem {
                    SynInfo: Infoitem::Call(exprcall.clone()),
                    ItemOrig: None,
                }, color_info, var_def, stack_num);
            }
        },
        Expr::MethodCall(exprm_call) => {
            let m_call = String::from(format!("{}", exprm_call.method));
            debug!("func found: {}",  m_call);
            let mcall_rap = RAP::Function(FuncInfo{name: Some(format!("{}",  m_call))});
            non_allo_insert(format!("{}", m_call),
            StackItem {
                SynInfo: Infoitem::MethodCall(exprm_call.clone()),
                ItemOrig: Some(mcall_rap),
            }, color_info, var_def, stack_num);
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
                non_allo_insert(format!("{}", path_fmt(&exprpath)),
                StackItem {
                    SynInfo: Infoitem::Reference(expred.clone()),
                    ItemOrig: None,
                }, color_info, var_def, stack_num);
            }
        },
        Expr::Block(expr_block) => {
            debug!("found block");
            color_info.push(HashMap::new());
            for stmt in &expr_block.block.stmts {
                parse_stmt(&stmt, color_info, var_def, stack_num+1);
            }
        },
        Expr::Struct(expr_struct) => {
            //TODO: take care of struct later
            // debug!("found struct");
            // let mut field_vec = Vec::new();
            // for i in &expr_struct.fields {
            //     match &i.member {
            //         syn::Member::Named(Ident) => {
            //             field_vec.push(format!("{}",Ident));
            //         }
            //         _ => {
            //             info!("struct type not supported")
            //         }
            //     }
            // }
            // local.field = Some(field_vec);
            // color_insert(format!("{}", expr_struct.path.segments[expr_struct.path.segments.len()-1].ident), 
            // Infoitem::ExprStruct(expr_struct.clone()), color_info, stack_num);
        },
        // TODO: println does not exactly take up resource (for type that implemented copy trait)
        Expr::Macro(_macro) => {
            debug!("found macro");
            let macro_path = &_macro.mac.path;
            if let Some(macro_func) = macro_path.segments.first() {
                debug!("found {}", macro_func.ident);
                //only consider Println and assert here
                var_allo_insert(format!("{}", macro_func.ident),
                StackItem {
                    SynInfo: Infoitem::Macro(_macro.clone()),
                    ItemOrig: None,
                }, color_info, var_def, stack_num);

                if macro_func.ident.to_string() == "println" {
                    let mut tokentree_buff = Vec::new();
                    let mut first_lit = false;
                    for item in _macro.mac.tokens.clone() {
                        debug!("{:?}",item);
                        match item {
                            proc_macro2::TokenTree::Punct(punct) => {
                                if !first_lit {
                                    // dump "{:?...}" in println!()
                                    tokentree_buff.clear();
                                    first_lit = true;
                                } else {
                                    let mut tokenstream_buff = proc_macro2::TokenStream::new();
                                    tokenstream_buff.extend(tokentree_buff);
                                    let res: Result<syn::Expr, syn::Error> = syn::parse2(tokenstream_buff);
                                    match res {
                                        Ok(exp) => parse_expr(&exp, local, color_info, var_def, stack_num),
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
                        Ok(exp) => parse_expr(&exp, local, color_info, var_def, stack_num),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                } else if macro_func.ident.to_string() == "assert" {
                    let res: Result<syn::Expr, syn::Error> = syn::parse2(_macro.mac.tokens.clone());
                    match res {
                        Ok(exp) => parse_expr(&exp, local, color_info, var_def, stack_num),
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