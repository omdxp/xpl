fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <script.xpl>", args[0]);
        std::process::exit(1);
    }
    let outputs = xpl::run_file(&args[1])?;
    for line in outputs {
        println!("{}", line);
    }
    Ok(())
}
