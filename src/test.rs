fn main() {
    let mut b = 5;
    let c = &b;
    let & a = c;
    let s = String::from("hello");
    let len1 = String::len(&s);
    let len2 = s.len(); // shorthand for the above
    println!("len1 = {} = len2 = {}", len1, len2);
}