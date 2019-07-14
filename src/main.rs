use std::io::Error;

fn main() {
    let _ = flowrider_bmp::run()
        .or_else::<Error, _>(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
}