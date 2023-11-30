use gloo::history::{History, MemoryHistory};

fn main() {
    let history = MemoryHistory::new();
    history.push("/home");
    history.push("/about");
    history.push("/contact");
    history.go(-2);
    assert_eq!(history.location().path(), "/home");
}
