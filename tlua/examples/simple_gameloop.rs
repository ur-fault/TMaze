use tlua::runtime::Runtime;

fn main() {
    let rt = Runtime::new("tlua");

    let res = rt.eval::<()>(
        r#"
print("Hello, world!")
    "#,
    );

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    } else {
        println!("Success!");
    }
}
