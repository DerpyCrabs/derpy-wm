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
    FocusWindow(String),
    FullscreenToggle,
    Cycle,
}

#[derive(Debug, Clone)]
pub enum ScratchpadEvent {
    AddWindow(String),
    RemoveWindow(String),
    ToggleWindow(String),
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
    Scratchpad(ScratchpadEvent),
    Unknown,
}

pub fn parse_event(ev_str: Result<String>) -> Event {
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
    } else if [
        "WS_FOCUS",
        "WS_MOVE",
        "WS_CYCLE",
        "WS_FULLSCREEN",
        "WS_FOCUS_WINDOW",
    ]
    .contains(&ev_str_parts[0].as_str())
    {
        if ev_str_parts[0].as_str() == "WS_FOCUS_WINDOW" {
            return Event::Workspace(WorkspaceEvent::FocusWindow(ev_str_parts[1].clone()));
        }
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
            "WS_FULLSCREEN" => Event::Workspace(WorkspaceEvent::FullscreenToggle),
            _ => unreachable!(),
        }
    } else if ["SP_ADD", "SP_REMOVE", "SP_TOGGLE"].contains(&ev_str_parts[0].as_str()) {
        match ev_str_parts[0].as_str() {
            "SP_ADD" => Event::Scratchpad(ScratchpadEvent::AddWindow(ev_str_parts[1].clone())),
            "SP_REMOVE" => {
                Event::Scratchpad(ScratchpadEvent::RemoveWindow(ev_str_parts[1].clone()))
            }
            "SP_TOGGLE" => {
                Event::Scratchpad(ScratchpadEvent::ToggleWindow(ev_str_parts[1].clone()))
            }
            _ => unreachable!(),
        }
    } else {
        Event::Unknown
    }
}

pub fn focus_window(window_id: impl Into<String>) {
    Command::new("wtf").arg(window_id.into()).status().ok();
}

pub fn fullscreen_window(window_id: impl Into<String> + Clone, (w, h): (usize, usize)) {
    Command::new("chwb")
        .arg("-s")
        .arg("0")
        .arg(window_id.clone().into())
        .status()
        .ok();
    Command::new("wtp")
        .args(&["0", "0", &w.to_string(), &h.to_string()])
        .arg(window_id.into())
        .status()
        .ok();
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

pub fn window_type(window_id: impl Into<String>) -> Option<String> {
    let output = Command::new("xprop")
        .arg("-id")
        .arg(window_id.into())
        .arg("_NET_WM_WINDOW_TYPE")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .ok();
    if let Some(typ) = output {
        let parts: Vec<String> = typ.split("=").map(|s| s.to_string()).collect();
        if parts.len() != 2 {
            return None;
        }
        return Some(parts[1].trim().to_string());
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
