fn main() {
    let mut i = 0;
    loop {
        println!("iter {i}");
        i += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
