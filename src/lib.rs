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
pub enum WorkspaceEventType {
    MoveWindow,
    Focus,
    Cycle,
}
#[derive(Debug, Clone)]
pub struct WindowEvent {
    pub window_id: String,
    pub event_type: WindowEventType,
}

#[derive(Debug, Clone)]
pub struct WorkspaceEvent {
    pub workspace: usize,
    pub event_type: WorkspaceEventType,
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
        let event_type = match ev_str_parts[0].as_str() {
            "WS_FOCUS" => WorkspaceEventType::Focus,
            "WS_MOVE" => WorkspaceEventType::MoveWindow,
            "WS_CYCLE" => {
                return Event::Workspace(WorkspaceEvent {
                    workspace: 0,
                    event_type: WorkspaceEventType::Cycle,
                })
            }
            _ => unreachable!(),
        };
        let workspace: usize = ev_str_parts[1].parse().expect("Invalid workspace event");
        if workspace > 0 && workspace <= workspace_count {
            Event::Workspace(WorkspaceEvent {
                workspace: workspace - 1,
                event_type,
            })
        } else {
            Event::Unknown
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

pub fn get_workspace_size(workspace_id: usize) -> (usize, usize) {
    (500, 300)
}

pub fn tile_workspace(workspace: &Vec<String>, gap: usize) {
    let (wsw, wsh) = get_workspace_size(0);

    match workspace.len() {
        0 => return,
        1 => {
            let full_w = wsw - 2 * gap;
            let full_h = wsh - 2 * gap;
            move_window(&workspace[0], gap, gap, full_w, full_h);
        }
        n => {
            let half_w = (wsw - 3 * gap) / 2;
            let left_n = n / 2;
            let right_n = n - left_n;
            let left_h = (wsh - (left_n + 1) * gap) / left_n;
            let right_h = (wsh - (right_n + 1) * gap) / right_n;
            let mut left_strip = workspace.clone();
            let right_strip = left_strip.split_off(left_n);

            for (i, wid) in left_strip.iter().enumerate() {
                move_window(wid, gap, gap * (i + 1) + left_h * i, half_w, left_h);
            }
            for (i, wid) in right_strip.iter().enumerate() {
                move_window(
                    wid,
                    half_w + gap * 2,
                    gap * (i + 1) + right_h * i,
                    half_w,
                    right_h,
                );
            }
        }
    }
}
