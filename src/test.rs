fn main() {
    let mut b = 5;
    let c = &b;
    let &_a = c;
    b = 2;
    let s = String::from("hello");
    let len1 = String::len(&s);
    let len2 = s.len(); // shorthand for the above
    assert!(b!=3);
    println!("len1 = {} = len2 = {}", len1, len2);
    let x = String::from("hello");
    let _z = {
        let y = x;
        println!("{}", y);
        // ...
    };
    println!("Hello, world!");
}