mod app;
mod server;
mod sign;
mod view;

fn main() {
    if let Err(e) = app::run() {
        println!("{}", e);
    }
}
