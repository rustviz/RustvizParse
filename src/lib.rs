pub mod syn_parse;

#[cfg(test)]
mod tests {
    use crate::syn_parse::{syn_parse};
    use std::path::PathBuf;

    #[test]
     fn test() {
        // let file_source = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/examples/struct_rect/source.rs");
        // let annotated_source = PathBuf::from("/Users/haochenz/Desktop/rustviz/src/examples/struct_rect/input/annotated_source.rs");
        let fname =  PathBuf::from("");
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