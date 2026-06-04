use std::sync::OnceLock;

pub const NODE_PATH: &str = "C:\\Program Files\\nodejs\\node.exe";

static BRIDGE: OnceLock<String> = OnceLock::new();

pub fn set_bridge_path(path: String) {
    let _ = BRIDGE.set(path);
}

pub fn bridge_path() -> &'static str {
    BRIDGE.get().expect("bridge_path not initialized")
}
