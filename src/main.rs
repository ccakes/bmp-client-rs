use std::io::Error;

fn main() {
    let _ = bmp_client::run()
        .or_else::<Error, _>(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
}
