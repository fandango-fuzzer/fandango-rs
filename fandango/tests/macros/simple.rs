use fandango::Fandango;

// note: path is weird here because we are in a deeply nested directory
// hopefully this doesn't change too much :)))
#[derive(Fandango)]
#[grammar = "../../../../fandango/tests/macros/simple.fan"]
struct Simple;

fn main() {}
