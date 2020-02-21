use std::io::Result;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEventType {
    EnterNotify,
    CreateNotify,
    DestroyNotify,
    MapNotify,
    FocusIn,
}

#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    MoveWindow(usize),
    Focus(usize),
    Cycle,
}
#[derive(Debug, Clone)]
pub struct WindowEvent {
    pub window_id: String,
    pub event_type: WindowEventType,
}

#[derive(Debug, Clone)]
pub enum Event {
    Window(WindowEvent),
    Workspace(WorkspaceEvent),
    Unknown,
}

pub fn parse_event(ev_str: Result<String>, workspace_count: usize) -> Event {
    let ev_str_parts: Vec<String> = ev_str
        .unwrap()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect();
    if ["ENTER", "CREATE", "DESTROY", "MAP", "FOCUS_IN"].contains(&ev_str_parts[0].as_str()) {
        let event_type = match ev_str_parts[0].as_str() {
            "ENTER" => WindowEventType::EnterNotify,
            "CREATE" => WindowEventType::CreateNotify,
            "DESTROY" => WindowEventType::DestroyNotify,
            "MAP" => WindowEventType::MapNotify,
            "FOCUS_IN" => WindowEventType::FocusIn,
            _ => unreachable!(),
        };
        Event::Window(WindowEvent {
            window_id: ev_str_parts[1].to_owned(),
            event_type,
        })
    } else if ["WS_FOCUS", "WS_MOVE", "WS_CYCLE"].contains(&ev_str_parts[0].as_str()) {
        let workspace: Option<usize> = if ev_str_parts.len() < 2 {
            None
        } else {
            ev_str_parts[1].parse().ok()
        };
        match ev_str_parts[0].as_str() {
            "WS_FOCUS" => Event::Workspace(WorkspaceEvent::Focus(
                workspace.expect("WS_FOCUS event takes workspace argument") - 1,
            )),
            "WS_MOVE" => Event::Workspace(WorkspaceEvent::MoveWindow(
                workspace.expect("WS_MOVE event takes workspace argument") - 1,
            )),
            "WS_CYCLE" => Event::Workspace(WorkspaceEvent::Cycle),
            _ => unreachable!(),
        }
    } else {
        Event::Unknown
    }
}

pub fn focus_window(window_id: &str) {
    Command::new("wtf").arg(window_id).status().ok();
}

pub fn focused_window() -> Option<String> {
    let output = Command::new("pfw")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok();
    if let Some(wid) = output {
        if wid != "" {
            return Some(wid);
        }
    }
    None
}

pub fn map_window(window_id: impl Into<String>) {
    Command::new("mapw")
        .arg("-m")
        .arg(window_id.into())
        .status()
        .ok();
}

pub fn unmap_window(window_id: impl Into<String>) {
    Command::new("mapw")
        .arg("-u")
        .arg(window_id.into())
        .status()
        .ok();
}

pub fn border_window(window_id: impl Into<String>, color: impl Into<String>) {
    Command::new("chwb")
        .arg("-c")
        .arg(color.into())
        .arg("-s")
        .arg("3")
        .arg(window_id.into())
        .status()
        .ok();
}

pub fn move_window(window_id: impl Into<String>, x: usize, y: usize, w: usize, h: usize) {
    Command::new("wtp")
        .args(
            vec![x, y, w, h]
                .iter()
                .map(|i| format!("{}", i))
                .collect::<Vec<String>>(),
        )
        .arg(window_id.into())
        .status()
        .ok();
}
