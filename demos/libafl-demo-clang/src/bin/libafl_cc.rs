//! CC
use std::env;

use libafl_cc::{ClangWrapper, CompilerWrapper, ToolWrapper};

/// Run the CC wrapper
///
/// # Panics
///
/// Panics if the compiler fails
pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut dir = env::current_exe().unwrap();
        let wrapper_name = dir.file_name().unwrap().to_str().unwrap();

        let is_cpp = match wrapper_name[wrapper_name.len() - 2..]
            .to_lowercase()
            .as_str()
        {
            "cc" => false,
            "++" | "pp" | "xx" => true,
            _ => panic!(
                "Could not figure out if c or c++ wrapper was called. Expected {} to end with c or cxx",
                dir.display()
            ),
        };

        dir.pop();

        let mut cc = ClangWrapper::new();
        let cc = cc
            .cpp(is_cpp)
            // silence the compiler wrapper output, needed for some configure scripts.
            .silence(true)
            .parse_args(&args)
            .expect("Failed to parse the command line");

        cc.link_staticlib(&dir, "libafl_example");

        if let Some(code) = cc
            .add_arg("-fsanitize-coverage=trace-pc-guard")
            .run()
            .expect("Failed to run the wrapped compiler")
        {
            std::process::exit(code);
        }
    } else {
        panic!("LibAFL CC: No Arguments given");
    }
}
