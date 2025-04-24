// src/lib.rs

pub mod error;
pub mod parser;
pub mod vm;

pub use error::XplError;

/// Run an XPL script from the given file path, returning printed outputs
pub fn run_file(path: &str) -> Result<Vec<String>, XplError> {
    let program = parser::parse_file(path)?;
    // If no main function, treat as empty output
    if !program.functions.contains_key("main") {
        return Ok(Vec::new());
    }
    let mut vm = vm::VM::new(path.to_string());
    let outputs = vm.run(&program)?;
    Ok(outputs)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runs_empty_program() {
        let tmp = "<program name=\"empty\" version=\"1.0\"></program>";
        let path = std::env::temp_dir().join("empty.xpl");
        std::fs::write(&path, tmp).unwrap();
        let outputs = run_file(path.to_str().unwrap()).unwrap();
        assert!(outputs.is_empty());
    }

    #[test]
    fn runs_hello_example() {
        let path = "examples/hello.xpl";
        let outputs = run_file(path).unwrap();
        assert_eq!(
            outputs,
            vec![
                "Hello, World!".to_string(),
                "The result of 5 + 3 is: ".to_string(),
                "8".to_string(),
            ]
        );
    }

    #[test]
    fn runs_test_example() {
        let path = "examples/test.xpl";
        let outputs = run_file(path).unwrap();
        assert_eq!(
            outputs,
            vec![
                "Hello, World!".to_string(),
                "This is a test program.".to_string(),
                "Testing the XPL language.".to_string(),
            ]
        );
    }

    #[test]
    fn runs_conditional_example() {
        let path = "examples/conditional.xpl";
        let outputs = run_file(path).unwrap();
        assert_eq!(outputs, vec!["x minus 5 is zero".to_string()]);
    }

    #[test]
    fn undefined_variable_error() {
        let tmp = "<program name=\"err\" version=\"1.0\"><function name=\"main\"><body><print> y </print></body></function></program>";
        let path = std::env::temp_dir().join("err.xpl");
        std::fs::write(&path, tmp).unwrap();
        let err = run_file(path.to_str().unwrap()).unwrap_err().to_string();
        assert!(err.contains("Undefined variable y"));
        assert!(err.contains(path.to_str().unwrap()));
    }

    #[test]
    fn undefined_function_error() {
        let tmp = "<program name=\"errf\" include=\"examples/math.xpl\" version=\"1.0\"><function name=\"main\"><body><call function=\"none\"><param>1</param></call></body></function></program>";
        let path = std::env::temp_dir().join("errf.xpl");
        std::fs::write(&path, tmp).unwrap();
        let err = run_file(path.to_str().unwrap()).unwrap_err().to_string();
        assert!(err.contains("Undefined function none"));
        assert!(err.contains(path.to_str().unwrap()));
    }
}
