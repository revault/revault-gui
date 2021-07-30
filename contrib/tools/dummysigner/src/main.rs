mod app;
mod server;

fn main() {
    if let Err(e) = app::run() {
        println!("{}", e);
    }
}
