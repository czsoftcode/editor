use extism_pdk::*;

#[plugin_fn]
pub fn hello(input: String) -> FnResult<String> {
    Ok(format!("Hello, {}! This is a message from a PolyCredo WASM Plugin.", input))
}
