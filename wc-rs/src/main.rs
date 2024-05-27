fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 {
        let flag = args[1].as_str();
        let filepath = args[2].as_str();
        let content = std::fs::read_to_string(filepath).expect("Failed to read file");

        if flag == "-c" {
            println!("{} {}", content.bytes().count(), filepath);
        }
    }
}
