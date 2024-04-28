use tlua::runtime::Runtime;

#[tokio::main]
async fn main() {
    let rt = Runtime::new("tlua");

    rt.eval::<()>(
        r#"
function f()
    print("before sleep")
    tlua.task.sleep(1)
    print("after sleep")
    tlua.exit(69)
end
tlua.task.spawn(f)
    "#,
    )
    .unwrap();

    loop {
        rt.run_frame(None);
    }
}
