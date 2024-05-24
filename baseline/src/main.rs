use otils;

fn main() {
    let mut v: Vec<i64> = (0..128).rev().collect();
    otils::osort(&mut v[..], 8);
    println!("{:?}", v);
}
