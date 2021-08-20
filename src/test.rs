// fn main() {
//     let s = String::from("hello");
//     takes_ownership(s);
//     let mut x = 5;
//     let y = x;
//     x = 6;
// }

// fn takes_ownership(some_string: String) {
//     println!("{}", some_string);
// }

fn main() {
    fn test_str() {
        println!("test_str called");
    }
    test_str();
}