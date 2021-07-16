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
use log::{debug};
// use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::io::Write;
use clap::{Arg, App};

mod parse;

fn parse(FileName : &PathBuf) -> Result<String, Box<Error>> {    
    // let mut file = File::open("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs")?;
    let mut file = File::open(FileName)?;
    // let mut file = File::open("/Users/haochenz/Desktop/playgroud/parse/src/test.rs")?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let ast = syn::parse_file(&content)?;
    debug!("{:#?}", ast);
    let header = parse::str_gen(ast);
    println!("{}", header);
    Ok(header)
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
    let file_name = PathBuf::from(matches.value_of("target").unwrap());
    // println!("{:?}", FileName);
    let parse_res = parse(&file_name);
    println!("{}", parse_res.unwrap());
}