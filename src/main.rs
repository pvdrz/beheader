fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    beheader::preprocess_file(&args[1]).unwrap();
}
