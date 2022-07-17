use std::collections::HashMap;

use anyhow::Result;

mod bencode;

struct User {
    name: String,
    age: u32,
}


fn main() -> Result<()> {
    let user = User {name: "xiaohui".to_string(), age: 18};
    // d4:name7:xiaohui3:agei18ee
    Ok(())
}
