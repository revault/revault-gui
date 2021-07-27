mod app;

fn main() {
    if let Err(e) = app::run() {
        println!("{}", e);
    }
}
