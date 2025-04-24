fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <script.xpl>", args[0]);
        std::process::exit(1);
    }
    match xpl::run_file(&args[1]) {
        Ok(outputs) => {
            for line in outputs {
                println!("{}", line);
            }
        }
        Err(e) => {
            e.pretty_print();
            std::process::exit(1);
        }
    }
}
