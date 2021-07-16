fn main() {
    // let mut b = 5;
    // let c = &b;
    // let &_a = c;
    // b = 2;
    // let s = String::from("hello");
    // let len1 = String::len(&s);
    // let len2 = s.len(); // shorthand for the above
    // assert!(call(a));
    // // println!("len1 = {} = len2 = {}", len1, len2);
    // let x = String::from("hello");
    // let _z = {
    //     let y = x;
    //     // println!("{}", y);
    //     // ...
    // };
    // // println!("Hello, world!");
    println!("{hello {:b} loll }", compare_string(a,b), same_string(a,b));
    assert!(compare_two_strings(a,b));
    // println!("{0}, this is {1}. {1}, this is {0}", "Alice", "Bob");
}