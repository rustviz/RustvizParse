pub mod syn_parse;

#[cfg(test)]
mod tests {
    use crate::syn_parse::{syn_parse};
    use std::path::PathBuf;

    #[test]
     fn test() {
        let file_name = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/examples/hatra1/main.rs");
        println!("{:?}", FileName);
        let parse_res = syn_parse(&file_name);
        match parse_res {
            Ok((rap, color_info)) => {
                println!("{:?}", rap);
                println!("{:?}", color_info);
            }
            Err(e) => println!("error parsing header: {:?}", e),
        }
    }
}