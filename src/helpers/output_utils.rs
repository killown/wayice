use crate::core::state::{Backend, WayiceState};
use serde_json::{json, Value};
use smithay::output::Output;
use std::collections::HashMap;

pub fn get_monitor_info(output: &Output) -> HashMap<String, String> {
    let mut monitor_info = HashMap::new();

    monitor_info.insert("name".to_string(), output.name());
    monitor_info.insert("description".to_string(), output.description());

    if let Some(mode) = output.current_mode() {
        monitor_info.insert("width".to_string(), mode.size.w.to_string());
        monitor_info.insert("height".to_string(), mode.size.h.to_string());
        monitor_info.insert("refresh_rate".to_string(), format!("{} Hz", mode.refresh));
    }

    monitor_info
}

impl<BackendData: Backend> WayiceState<BackendData> {
    pub fn list_all_outputs(&mut self) -> String {
        let monitors: Vec<Output> = self.space.outputs().cloned().collect();
        let monitors_info: Vec<Value> = monitors
            .iter()
            .map(|output| {
                let mode = output.current_mode().unwrap();
                let size = mode.size;
                let refresh_rate = mode.refresh;
                json!({
                    "name": output.name(),
                    "size": { "width": size.w, "height": size.h },
                    "refresh_rate": refresh_rate
                })
            })
            .collect();

        let json_output = json!(monitors_info);
        json_output.to_string()
    }
}
