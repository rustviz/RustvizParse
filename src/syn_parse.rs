use syn::{Stmt, Expr, Pat, Item, FnArg, Type};
use log::{debug, info};
use std::collections::{HashSet, HashMap, BTreeMap};
use std::error::Error;
use std::fs::File;
use std::io::{Read, BufReader, BufRead};
use std::path::PathBuf;
use std::ptr;
use syn::spanned::Spanned;
use std::sync::Arc;
use rustviz_lib::data::{ResourceAccessPoint, 
    Owner, 
    Struct,
    MutRef,
    StaticRef,
    Function};

struct data_pkg {
    color_info: Vec<HashMap<String, Vec<StackItem>>>,
    var_alloc: HashMap<String, Vec<Arc<ResourceAccessPoint>>>,
    var_def: HashMap<String, HashMap<String, Arc<ResourceAccessPoint>>> // struct only
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct StackItem {
    SynInfo: Infoitem,
    ItemOrig: Arc<ResourceAccessPoint>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Infoitem {
    Struct(syn::Field), // struct definition
    Func(syn::ItemFn),
    FnArg(syn::FnArg),
    Local(syn::PatIdent), //let a = 5;
    Call(syn::ExprPath), // func_cal();
    MethodCall(syn::ExprMethodCall), // a.to_string();
    Reference(syn::ExprReference), // &a;
    ExprStruct(syn::Ident), // struct literal expression
    Macro(syn::PathSegment)
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

pub fn syn_parse(FileName : &PathBuf) 
    -> Result<(HashMap<String, Vec<Arc<ResourceAccessPoint>>>, Vec<HashMap<String, Vec<StackItem>>>), Box<Error>> {    
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    let mut file = File::open(FileName)?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);
    let mut data_pkg = data_pkg {
        color_info: Vec::new(),
        var_alloc: HashMap::new(),
        var_def: HashMap::new(),
    };
    data_pkg.color_info.push(HashMap::new());
    let mut hash_num: u64 = 1;
    parse_item(&ast.items, &mut data_pkg, &mut hash_num, 0);
    // color_gen(&color_info);
    Ok((data_pkg.var_alloc, data_pkg.color_info))
}

pub fn asource_gen(FileName : &PathBuf, color_info: &Vec<HashMap<String, Vec<StackItem>>>) -> Result<String, Box<Error>>{
    let mut insert_holder: BTreeMap<usize, BTreeMap<usize, String>> = BTreeMap::new();
    fn insert(insert_holder: &mut BTreeMap<usize, BTreeMap<usize, String>>, row: usize, col:usize, content: String) {
        match insert_holder.get_mut(&row) {
            Some(col_map) => {
                col_map.insert(col, content);
            },
            None => {
                let mut col_map = BTreeMap::new();
                col_map.insert(col, content);
                insert_holder.insert(row, col_map);
            }
        }
    }
    for i in color_info {
        for (key, val) in i {
            for item in val {
                let hash_id = item.ItemOrig.hash();
                match &item.SynInfo {
                    Infoitem::Struct(itemstruct) => {
                        let tag = format!("<tspan data-hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemstruct.ident.span().start().line,
                            itemstruct.ident.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemstruct.ident.span().end().line,
                            itemstruct.ident.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::Func(itemfunc) => {
                        let tag = format!("<tspan class=\"fn\" data-hash=\"0\" hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemfunc.sig.ident.span().start().line,
                            itemfunc.sig.ident.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemfunc.sig.ident.span().end().line,
                            itemfunc.sig.ident.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::FnArg(itemarg) => {
                        let tag = format!("<tspan data-hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemarg.span().start().line,
                            itemarg.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemarg.span().end().line,
                            itemarg.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::Local(itemlocal) => {
                        let tag = format!("<tspan data-hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemlocal.span().start().line,
                            itemlocal.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemlocal.span().end().line,
                            itemlocal.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::Call(itemcall) => {
                        let tag = format!("<tspan data-hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemcall.span().start().line,
                            itemcall.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemcall.span().end().line,
                            itemcall.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::MethodCall(itemmcall) => {
                        // println!("--------------");
                        // println!("MethodCall found: {:?}", itemmcall);
                        // println!("{:?}", itemmcall.span().start());
                        // println!("{:?}", itemmcall.span().end());
                        // println!("--------------");
                    },
                    Infoitem::Reference(itemref) => {
                        // println!("--------------");
                        // println!("Reference found: {:?}", itemref);
                        // println!("{:?}", itemref.span().start());
                        // println!("{:?}", itemref.span().end());
                        // println!("--------------");
                    },
                    Infoitem::ExprStruct(itemstuexp) => {
                        let tag = format!("<tspan data-hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemstuexp.span().start().line,
                            itemstuexp.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemstuexp.span().end().line,
                            itemstuexp.span().end().column,
                            String::from("</tspan>"));
                    },
                    Infoitem::Macro(itemmacro) => {
                        let tag = format!("<tspan class=\"fn\" data-hash=\"0\" hash=\"{}\">", hash_id);
                        insert(&mut insert_holder,
                            itemmacro.ident.span().start().line,
                            itemmacro.ident.span().start().column,
                            tag);
                        insert(&mut insert_holder,
                            itemmacro.ident.span().end().line,
                            itemmacro.ident.span().end().column,
                            String::from("</tspan>"));
                    }
                }
            }
        }
    }
    // write into file
    let mut output = String::new();
    let file = File::open(FileName)?;
    let reader = BufReader::new(file);
    let mut line_num = 1;
    let mut cursor = 0;
    for line in reader.lines() {
        match line {
            Ok(ref v) => {
                match insert_holder.get(&line_num) {
                    Some(col_map) => {
                        for (ky, val) in col_map {
                            let word = &v[cursor..*ky];
                            output.push_str(word);
                            output.push_str(val);
                            cursor=*ky;
                        }
                        output.push_str(&v[cursor..])
                    },
                    _ => {
                        output.push_str(v);
                    },
                }
            },
            Err(ref e) => {println!("error parsing header: {:?}", e)},
        }
        cursor = 0;
        output.push_str("\n");
        line_num+=1;
    }
    println!("{}", output);
    Ok(output)
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

pub fn header_gen_str(var_alloc: &HashMap<String, Vec<Arc<ResourceAccessPoint>>>) -> String {
    // generate header lines 
    let mut header = String::new();
    let mut struct_store: Vec<Struct> = Vec::new();

    header.push_str("/* --- BEGIN Variable Definitions ---\n");
    for (_, value) in var_alloc {
        for i in value {
            header.push_str(&i.rap_header(&mut struct_store));
        }
    }
    // deal with structs
    let mut struct_owner: HashMap<u64, String> = HashMap::new();
    let mut struct_member: HashMap<u64, Vec<String>> = HashMap::new();

    for i in struct_store {
        if i.is_member {
            match struct_member.get_mut(&i.owner) {
                Some(member_vec) => {
                    member_vec.push(i.name);
                },
                None => {
                    struct_member.insert(i.owner, vec![i.name]);
                }
            }
        } else {
            struct_owner.insert(i.hash, i.name);
        }
    }

    for (key, val) in struct_owner {
        header.push_str(&format!("Struct {}{{", val));
        match struct_member.get(&key) {
            Some(member_vec) => {
                for i in member_vec {
                    header.push_str(&format!("{},",i));
                }
            }
            None => {
                //Error!
            }
        }
        header.pop();
        header.push_str("};\n");
    }
    header.push_str("--- END Variable Definitions --- */\n");
    header
}

fn struct_field_insert(syn_info: Infoitem,
    struct_type: String,
    mut target_rap: ResourceAccessPoint,
    data: &mut data_pkg,
    stack_num: usize) {

        let mut rap_arc: Option<Arc<ResourceAccessPoint>> = None;
        match data.var_def.get(&struct_type) {
            Some(field) => {
                match field.get(target_rap.name()) {
                    Some(res) => {
                        target_rap.hash_mod(res.hash().clone());
                        rap_arc = Some(res.clone());
                    },
                    _ => {
                        //ERROR!
                    }
                }
            },
            _ => {
                //ERROR!
            }
        }

        if data.var_alloc.contains_key(target_rap.name()) {
            // TODO: add shadow RAP
            // var_def[&get_identstr(&target_rap)].push(target_rap);
        } else {
            // add RAP
            data.var_alloc.insert(target_rap.name().clone(), vec![Arc::new(target_rap.clone())]);
        }

        let stack_item = StackItem {
            SynInfo: syn_info,
            ItemOrig: Arc::clone(&rap_arc.unwrap()),
        };
        // push into stack
        match data.color_info[stack_num].get_mut(target_rap.name()) {
            Some(var_map) => {
                var_map.push(stack_item);
            },
            None => {
                data.color_info[stack_num].insert(target_rap.name().clone(), vec![stack_item]);
            }
        }
    }

fn struct_def_insert(syn_info: Infoitem,
    struct_type: String,
    target_rap: ResourceAccessPoint,
    data: &mut data_pkg,
    stack_num: usize) {
        let rap_arc = Arc::new(target_rap.clone());
        let stack_item = StackItem {
            SynInfo: syn_info,
            ItemOrig: Arc::clone(&rap_arc),
        };

        match data.var_def.get_mut(&struct_type) {
            Some(field) => {
                field.insert(target_rap.name().clone(), rap_arc);
            },
            _ => {
                let mut field = HashMap::new();
                field.insert(target_rap.name().clone(), rap_arc);
                data.var_def.insert(struct_type, field);
            }
        }
        // push into stack
        match data.color_info[stack_num].get_mut(target_rap.name()) {
            Some(var_map) => {
                var_map.push(stack_item);
            },
            None => {
                data.color_info[stack_num].insert(target_rap.name().clone(), vec![stack_item]);
            }
        }
    }

fn var_allo_insert(syn_info: Infoitem, 
    target_rap: ResourceAccessPoint,
    data: &mut data_pkg,
    stack_num: usize) {
    // variable initialization happened -> 
    // look for RAP, if exist then add shadowing RAP, else add RAP to var_def
    // push into color_info
    // called for the following InfoItem:
    // ----------------------------------
    // Func(syn::ItemFn)
    // FnArg(syn::FnArg)
    // Local(syn::Local)
    // ----------------------------------
    // is_def is true for struct declaraion
    // Struct(syn::ItemStruct)
    // ----------------------------------
    let rap_arc = Arc::new(target_rap.clone());
    let stack_item = StackItem {
        SynInfo: syn_info,
        ItemOrig: Arc::clone(&rap_arc),
    };

    if data.var_alloc.contains_key(target_rap.name()) {
        // TODO: add shadow RAP
        // var_def[&get_identstr(&target_rap)].push(target_rap);
    } else {
        // add RAP
        data.var_alloc.insert(target_rap.name().clone(), vec![rap_arc]);
    }
    
    // push into stack
    match data.color_info[stack_num].get_mut(target_rap.name()) {
        Some(var_map) => {
            var_map.push(stack_item);
        },
        None => {
            data.color_info[stack_num].insert(target_rap.name().clone(), vec![stack_item]);
        }
    }
}

fn non_allo_insert(ident: String,
    syn_info: Infoitem, 
    target_rap: Option<ResourceAccessPoint>,
    data: &mut data_pkg,
    hash_num: &mut u64,
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
    let mut rap_arc: Option<Arc<ResourceAccessPoint>> = None;
    let stack_item;

    if let Some(mut tar_rap) = target_rap {
        // ----------------------------------
        // Call(syn::ExprCall), // func_cal();
        // MethodCall(syn::ExprMethodCall), // a.to_string();
        // Macro(syn::ExprMacro) 
        // ----------------------------------
        match data.var_alloc.get(&ident) {
            Some(rap_vec) => {
                //TODO: shadowing variable
                rap_arc = Some(rap_vec[0].clone());
            },
            _ => {
                *hash_num+=1;
                let rap_alloc = Arc::new(tar_rap);
                rap_arc = Some(rap_alloc.clone());
                data.var_alloc.insert(ident.clone(), vec![rap_alloc]);
            }
        }
    } else {
        // ----------------------------------
        // Reference(syn::ExprReference), // &a;
        // ----------------------------------
        match data.var_alloc.get(&ident) {
            Some(rap_vec) => {
                //TODO: shadowing variable
                rap_arc = Some(rap_vec[0].clone());
            },
            _ => {
                //Error!!
                println!("undefined variable {} found", ident);
            }
        }
    }

    stack_item = StackItem {
        SynInfo: syn_info,
        ItemOrig: Arc::clone(&rap_arc.unwrap()),
    };

    // push into stack
    match data.color_info[stack_num].get_mut(&ident) {
        Some(var_map) => {
            var_map.push(stack_item);
        },
        None => {
            data.color_info[stack_num].insert(ident.clone(), vec![stack_item]);
        }
    }
}



//TODO: do I need to specify same lifetime for color_info and var_def
fn parse_item (items: &Vec<syn::Item>, 
    data: &mut data_pkg,
    hash_num: &mut u64,
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
                // register func into var_def
                let func_rap = ResourceAccessPoint::Function(Function{name: format!("{}", func.sig.ident), hash: hash_num.clone()});
                *hash_num+=1;
                debug!("--------------");
                debug!("func found: {:?}", func_rap);
                debug!("{:?}", func.span().start());
                debug!("{:?}", func.span().end());
                debug!("--------------");
                // push stack and register func into color_info
                data.color_info.push(HashMap::new());
                var_allo_insert(Infoitem::Func(func.clone()), 
                func_rap, data, stack_num);

                if func.sig.inputs.len() != 0 {
                    // match arguments
                    // create new stack for func arg
                    for arg in &func.sig.inputs {
                        match arg {
                            FnArg::Typed(pat_type) => {
                                // match arg type
                                let mut func_argname = String::new();
                                let mut is_mut = false;
                                debug!("--------------");
                                // extract arg ident
                                match &*pat_type.pat {
                                    Pat::Ident(pat_ident) => {
                                        // push var into stack
                                        func_argname = String::from(format!("{}", pat_ident.ident));
                                        if let Some(_mutability) = &pat_ident.mutability {
                                            is_mut = true;
                                        }
                                        // debug!("arg found: {:?}", func_arg.name);
                                    },
                                    _ => info!("function arg name not supported")
                                }
                                debug!("{:?}", pat_type.span().start());
                                debug!("{:?}", pat_type.span().end());
                                debug!("--------------");
                                // extract arg type
                                // TODO: fix this
                                let mut arg_rap = ResourceAccessPoint::Owner(Owner {name: String::new(), hash: 0, is_mut: false});
                                match &*pat_type.ty {
                                    Type::Reference(type_reference) => {                                  
                                        if let Some(_mutability) = &type_reference.mutability {
                                            arg_rap = ResourceAccessPoint::MutRef(MutRef {name: func_argname.clone(), hash: hash_num.clone(), is_mut: is_mut});
                                            *hash_num+=1;
                                        } else {
                                            arg_rap = ResourceAccessPoint::StaticRef(StaticRef {name: func_argname.clone(), hash: hash_num.clone(), is_mut: is_mut});
                                            *hash_num+=1;
                                        }
                                    },
                                    Type::Path(_) => {
                                        arg_rap = ResourceAccessPoint::Owner(Owner {name: func_argname.clone(), hash: hash_num.clone(), is_mut: is_mut});
                                        *hash_num+=1;
                                    }
                                    _ => info!("function arg type not supported")
                                }
                                var_allo_insert(Infoitem::FnArg(arg.clone()), 
                                arg_rap, data, stack_num+1);
                            },
                            _ => info!("syn::Receiver <self> not supported")
                        }
                    }
                }
                // parse function block
                for stmt in &func.block.stmts {
                    parse_stmt(&stmt, data, hash_num, stack_num+1);
                }
            },
            Item::Struct(itemstruct) => {
                // TODO: fix struct
                let struct_type = format!("{}", itemstruct.ident);
                match &itemstruct.fields {
                    syn::Fields::Named(named_field) => {
                        for i in &named_field.named {
                            let struct_rap = ResourceAccessPoint::Struct(
                                Struct {
                                    name: format!("{}", i.ident.clone().unwrap()),
                                    hash: hash_num.clone(),
                                    owner: hash_num.clone(), // no owner for struct declaration
                                    is_mut: false,
                                    is_member: false,
                                }
                            );
                            *hash_num+=1;
                            struct_def_insert(Infoitem::Struct(i.clone()), struct_type.clone(), struct_rap, data, stack_num);
                        }
                    },
                    _ => {
                        println!("not supported");
                    }
                }      
            },
            _ => info!("syn::Item option not supported")
        }
    }
    // var_def
}

// used to derive info from expr parse after eq sign
struct expr_derive {
    name: String,
    var_mut: bool,
    is_ref: bool,
    ref_mut: bool,
    is_struct: bool,
    hash: u64,
}

fn parse_stmt(stmt: &syn::Stmt, 
    data: &mut data_pkg,
    hash_num: &mut u64,
    stack_num: usize) {
    // local => let statement
    // Item => function definition, struct definition etc.
    // Expr => Expression without semicolon (return...)
    // Semi => Expression with semicolon

    match stmt {
        Stmt::Local(loc) => {
            let mut expr_pass = expr_derive {
                name: String::from(""),
                var_mut: false,
                is_ref: false,
                ref_mut: false,
                is_struct: false,
                hash: hash_num.clone(),
            };
            *hash_num+=1;
            let mut location_item = None;
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
                    location_item = Some(pat_ident);
                    debug!("Owner found: {}, ref_mut: {:?}, ref: {:?}", pat_ident.ident, pat_ident.mutability, pat_ident.by_ref);
                    debug!("{:?}", pat_ident.ident.span().start());
                    debug!("{:?}", pat_ident.ident.span().end());
                    expr_pass.name = String::from(format!("{}", pat_ident.ident));
                    // var ref_mut not is_ref ref_mut
                    if let Some(_mutable) = &pat_ident.mutability {
                        expr_pass.var_mut = true;
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
                        location_item = Some(pat_ident);
                        expr_pass.name = String::from(format!("{}", pat_ident.ident));
                        if let Some(_mutability) = pat_ident.mutability {
                            expr_pass.var_mut = true;
                        }
                    }
                    if let Type::Reference(type_ref) = &*pat_type.ty {
                        expr_pass.is_ref = true;
                        if let Some(_mutable) = type_ref.mutability {
                            expr_pass.ref_mut = true;
                        }
                    }
                    //TODO: add struct here
                },
                _ => info!("stmt not supported")
            }

            //if a value or a is_ref is assigned
            if let Some((_eq, expr)) = &loc.init {
                parse_expr(expr, Some(&mut expr_pass), data, hash_num, stack_num);
            }
            
            let expr_rap;
            if expr_pass.is_struct {
                expr_rap = ResourceAccessPoint::Struct(
                    Struct {
                    name: expr_pass.name,
                    hash: expr_pass.hash.clone(),
                    owner: expr_pass.hash.clone(),
                    is_mut: expr_pass.var_mut,
                    is_member: false
                    }
                );
            } else {
                if expr_pass.is_ref {
                    if expr_pass.ref_mut {
                        expr_rap = ResourceAccessPoint::MutRef(
                            MutRef {
                            name: expr_pass.name,
                            hash: expr_pass.hash.clone(),
                            is_mut: expr_pass.var_mut,
                            }
                        );
                    } else {
                        expr_rap = ResourceAccessPoint::StaticRef( 
                            StaticRef {
                            name: expr_pass.name,
                            hash: expr_pass.hash.clone(),
                            is_mut: expr_pass.var_mut,
                            }
                        );
                    }
                } else {
                    expr_rap = ResourceAccessPoint::Owner(
                        Owner {
                        name: expr_pass.name,
                        hash: expr_pass.hash.clone(),
                        is_mut: expr_pass.var_mut, 
                        }
                    );
                }
            }
            var_allo_insert(Infoitem::Local(location_item.unwrap().clone()), expr_rap,
            data, stack_num);
        },
        Stmt::Semi(exp, _) => {
            parse_expr(&exp, None, data, hash_num, stack_num);
            info!("{:?}", exp);
        }, 
        Stmt::Expr(exp) => {
            parse_expr(&exp, None, data, hash_num, stack_num);
            info!("{:?}", exp);
        },
        Stmt::Item(item) => {
            parse_item(&vec![item.clone()], data, hash_num, stack_num);
        }
    }
    debug!("{:?}", stmt.span().start());
    debug!("{:?}", stmt.span().end());
    debug!("--------------");
}

fn parse_expr (expr: &syn::Expr, 
    stmt_pass: Option<&mut expr_derive>, 
    data: &mut data_pkg,
    hash_num: &mut u64,
    stack_num: usize) {

    debug!("--------------");
    debug!("expr found");

    match expr {
        Expr::Call(exprcall) => {
            if let Expr::Path(exprpath) = &*exprcall.func {
                let call_rap = ResourceAccessPoint::Function(Function{name: format!("{}", path_fmt(&exprpath)), hash: hash_num.clone()});
                non_allo_insert(format!("{}", path_fmt(&exprpath)),
                Infoitem::Call(exprpath.clone()), Some(call_rap),
                data, hash_num, stack_num);
            }
        },
        Expr::MethodCall(exprm_call) => {
            let m_call = String::from(format!("{}", exprm_call.method));
            debug!("func found: {}",  m_call);
            let mcall_rap = ResourceAccessPoint::Function(Function{name: format!("{}",  m_call), hash: hash_num.clone()});
            non_allo_insert(format!("{}", m_call),
            Infoitem::MethodCall(exprm_call.clone()),
            Some(mcall_rap), data, hash_num, stack_num);
        },
        Expr::Reference(expred) => {
            debug!("Owner's a is_ref: {:?}", expred.mutability);
            if let Some(stmt_derive) = stmt_pass {
                stmt_derive.is_ref = true;
                if let Some(_mutable) = &expred.mutability {
                    stmt_derive.ref_mut = true;
                }
            }
            
            if let Expr::Path(exprpath) = &*expred.expr {
                // println!("Ref target: {:?}", exprpath);
                debug!(" Ref target: {}", path_fmt(&exprpath));
                non_allo_insert(format!("{}", path_fmt(&exprpath)),
                Infoitem::Reference(expred.clone()),
                None, data, hash_num, stack_num);
            }
        },
        Expr::Block(expr_block) => {
            debug!("found block");
            data.color_info.push(HashMap::new());
            for stmt in &expr_block.block.stmts {
                parse_stmt(&stmt, data, hash_num, stack_num+1);
            }
        },
        Expr::Struct(expr_struct) => {
            //TODO: take care of struct later
            debug!("found struct");
            let struct_type = format!("{}", expr_struct.path.segments[expr_struct.path.segments.len()-1].ident);
            if let Some(stmt_derive) = stmt_pass {
                stmt_derive.is_struct = true;
                let owner_hash = stmt_derive.hash.clone();
                for i in &expr_struct.fields {
                    match &i.member {
                        syn::Member::Named(Ident) => {
                            let mut field = ResourceAccessPoint::Struct(
                                Struct {
                                name: format!("{}",Ident),
                                //TODO: is it fine without clone?
                                hash: 0,
                                owner: owner_hash,
                                is_mut: false,
                                is_member: true,
                                }
                            );
                            struct_field_insert(Infoitem::ExprStruct(Ident.clone()),
                            struct_type.clone(), field, data, stack_num);
                        }
                        _ => {
                            info!("struct type not supported")
                        }
                    }
                }
            }
        },

        Expr::Macro(_macro) => {
            debug!("found macro");
            let macro_path = &_macro.mac.path;
            if let Some(macro_func) = macro_path.segments.first() {
                //only consider Println and assert here
                let macro_rap = ResourceAccessPoint::Function(Function{name: format!("{}", macro_func.ident), hash: hash_num.clone()});
                non_allo_insert(format!("{}", macro_func.ident),
                Infoitem::Macro(macro_func.clone()),
                Some(macro_rap), data, hash_num, stack_num);

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
                                        Ok(exp) => parse_expr(&exp, None, data, hash_num, stack_num),
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
                        Ok(exp) => parse_expr(&exp, None, data, hash_num, stack_num),
                        Err(_) => debug!("Assert macro parse error"),
                    }
                } else if macro_func.ident.to_string() == "assert" {
                    let res: Result<syn::Expr, syn::Error> = syn::parse2(_macro.mac.tokens.clone());
                    match res {
                        Ok(exp) => parse_expr(&exp, None, data, hash_num, stack_num),
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