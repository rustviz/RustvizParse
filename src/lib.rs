pub mod syn_parse;

#[cfg(test)]
mod tests {
    #[test]
     fn test() {
        let file_name = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/RustvizParse/src/test.rs");
        // println!("{:?}", FileName);
        let parse_res = parse::parse(&file_name);
        println!("{}", parse_res.unwrap());
    }
}