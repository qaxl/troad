use troad_protocol::chat::{Chat, Color};

fn main() {
    let chat = Chat::new()
        .text("Hello, world!")
        .bold()
        .color(Color::Pink)
        // .extra()
        .text("I am inside you.")
        .reset()
        .obfuscated()
        .text("ALALALALALAL")
        .click_url("https://google.com")
        .underlined()
        .finish();

    println!("{chat}");
}
