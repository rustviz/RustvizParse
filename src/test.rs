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
    let a = 5;
    let b = 5;
    fn test() {
        let c = 5;
        let d = 5;
        let e = 10;
        {
            let e = 5;
            let f = 5;
            fn e() {
            }
            
        }
    }
}