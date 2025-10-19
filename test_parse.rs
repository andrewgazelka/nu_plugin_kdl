use kdl::KdlDocument;

fn main() {
    let input = include_str!("zellij-layout.kdl");
    match input.parse::<KdlDocument>() {
        Ok(doc) => println!("Success! Parsed: {:?}", doc),
        Err(e) => println!("Error: {}", e),
    }
}
