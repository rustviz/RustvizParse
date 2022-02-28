// #[macro_use]
extern crate quote;
extern crate syn;
extern crate clap;
extern crate proc_macro2;

// use core::fmt;
// use std::error::String;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use log::{debug};
// use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::io::Write;
use clap::{Arg, App};

mod syn_parse;
use crate::syn_parse::{syn_parse, header_gen_str, asource_gen};

fn main() {
  let file_name = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/examples/mutable_borrow");
  let mainfname = file_name.join("main.rs");
  let sourcefname = file_name.join("source.rs");
  // let file_name = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/examples/struct_rect/source.rs");
  // let (contents, line_num, mut var_map) = syn_parse::syn_parse(&mainfname);
  println!("{:?}", sourcefname);
  let parse_res = syn_parse(&sourcefname);
  match parse_res {
    Ok((rap, color_info)) => {
      // println!("{}", header_gen_str(&rap));
      println!("{:?}", rap);
      println!("{:?}", color_info);
      // let res = asource_gen(&file_name, &color_info,);
    }
    Err(e) => println!("error parsing header: {:?}", e),
  }
}

// fn main() {
//   env_logger::init();
//   let matches = App::new("Rustviz Parse")
//                       //   .version("1.0")
//                       //   .author("Kevin K. <kbknapp@gmail.com>")
//                         .about("Parse Owners and Functions")
//                         .arg(Arg::with_name("target")
//                           //    .short("t")
//                           //    .long("target")
//                           //    .value_name("FILE")
//                              .help("Target file for parsing")
//                              .required(true)
//                              .takes_value(true))
//                       //   .arg(Arg::with_name("INPUT")
//                       //        .help("Sets the input file to use")
//                       //        .required(true)
//                       //        .index(1))
//                         .get_matches();
//   // Create a file with header and original content
//   let mut file_name = PathBuf::from(matches.value_of("target").unwrap());
//   println!("{:?}", file_name);
//   let parse_res = syn_parse::syn_parse(&file_name);
//   match parse_res {
//     Ok(v) => {
//       let header = syn_parse::header_gen_str(&v.0);
//       let origin_contents = fs::read_to_string(&file_name);
//       file_name.pop();
//       file_name.push("main.rs");
//       let mut f = File::create(file_name).unwrap();
//       f.write_all(header.as_bytes());
//       f.write_all(origin_contents.unwrap().as_bytes());
//     },
//     Err(e) => println!("syn_parse ERROR: {:?}", e),
//   }
// }