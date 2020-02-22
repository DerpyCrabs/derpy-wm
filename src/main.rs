mod reconciler;

use ::derpywm::{
    parse_event, window_type, Config, Event, ScratchpadEvent, WindowEventType, WorkspaceEvent,
};
use reconciler::{actualize_screen, WMState};

use std::io::{self, BufRead};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Provide config path");
    }
    let config_file = std::fs::read_to_string(args[1].clone()).expect("Can't find config.toml");
    let config: Config = toml::from_str(config_file.as_str()).expect("Can't parse config file");

    let mut last_event = Event::Unknown;
    let mut last_state = WMState::new(config.workspaces);

    for event in io::stdin().lock().lines().map(parse_event) {
        let mut now = last_state.clone();
        let event_clone = event.clone();

        dbg!(event.clone());

        match event {
            Event::Window(event) => match event.event_type {
                WindowEventType::CreateNotify => {}
                WindowEventType::MapNotify => {
                    if let Event::Window(last_event) = &last_event {
                        if last_event.window_id == event.window_id
                            && last_event.event_type == WindowEventType::CreateNotify
                        {
                            if let Some(typ) = window_type(event.window_id.as_str()) {
                                if typ == "_NET_WM_WINDOW_TYPE_DOCK" {
                                    continue;
                                }
                            }
                            now.workspaces[now.focused_workspace]
                                .windows
                                .push(event.window_id.clone());
                            now.workspaces[now.focused_workspace]
                                .focus_history
                                .push(event.window_id);
                        }
                    }
                }
                WindowEventType::DestroyNotify => {
                    for workspace in &mut now.workspaces {
                        workspace
                            .focus_history
                            .retain(|wid| wid != event.window_id.as_str());
                        workspace
                            .windows
                            .retain(|wid| wid != event.window_id.as_str());
                        if let Some(wid) = &workspace.fullscreen {
                            if wid == event.window_id.as_str() {
                                workspace.fullscreen = None;
                            }
                        }
                    }
                    if let Some(shown) = &now.scratchpad.shown {
                        let (_, shown_wid) = now
                            .scratchpad
                            .windows
                            .iter()
                            .find(|(name, _)| name == shown.as_str())
                            .unwrap();
                        if shown_wid == event.window_id.as_str() {
                            now.scratchpad.shown = None;
                        }
                    }
                    now.scratchpad
                        .windows
                        .retain(|(_, wid)| wid != event.window_id.as_str());
                    if let Some(focused) = &now.focused_window {
                        if focused == event.window_id.as_str() {
                            now.focused_window = None
                        }
                    }
                }
                WindowEventType::FocusIn => {
                    let history = &mut now.workspaces[now.focused_workspace].focus_history;
                    if history.contains(&event.window_id) {
                        history.retain(|wid| wid != event.window_id.as_str());
                        history.push(event.window_id.clone());
                    } else {
                        now.focused_window = Some(event.window_id);
                    }
                }
                WindowEventType::FocusOut => continue,
            },
            Event::Workspace(event) => match event {
                WorkspaceEvent::Focus(ws) => {
                    if now.focused_workspace == ws {
                        continue;
                    }
                    now.focused_workspace = ws;
                }
                WorkspaceEvent::MoveWindow(ws) => {
                    if now.focused_workspace == ws {
                        continue;
                    }
                    if let Some(focused_wid) = &now.focused_window {
                        now.workspaces[now.focused_workspace]
                            .windows
                            .retain(|wid| wid != focused_wid.as_str());
                        now.workspaces[now.focused_workspace]
                            .focus_history
                            .retain(|wid| wid != focused_wid.as_str());
                        if let Some(fullscreen) = &now.workspaces[now.focused_workspace].fullscreen
                        {
                            if fullscreen == focused_wid.as_str() {
                                now.workspaces[now.focused_workspace].fullscreen = None;
                            }
                        }
                        now.workspaces[ws].windows.push(focused_wid.clone());
                        now.workspaces[ws]
                            .focus_history
                            .push(focused_wid.to_string());
                    } else {
                        continue;
                    }
                }
                WorkspaceEvent::FullscreenToggle => {
                    if let Some(_) = &now.workspaces[now.focused_workspace].fullscreen {
                        now.workspaces[now.focused_workspace].fullscreen = None;
                    } else {
                        if let Some(focused_wid) = &now.focused_window {
                            now.workspaces[now.focused_workspace].fullscreen =
                                Some(focused_wid.clone());
                        }
                    }
                }
                WorkspaceEvent::Cycle => {
                    let windows = &mut now.workspaces[now.focused_workspace].windows;
                    if windows.len() > 0 {
                        windows.rotate_right(1);
                    }
                }
                WorkspaceEvent::FocusWindow(direction) => {
                    if now.workspaces[now.focused_workspace].fullscreen.is_some()
                        || now.scratchpad.shown.is_some()
                    {
                        continue;
                    }
                    let windows = &now.workspaces[now.focused_workspace].windows;
                    if windows.len() < 2 {
                        continue;
                    }
                    let n = windows.len();
                    let left_n = n / 2;
                    let right_n = n - left_n;
                    let focused_wid = now.workspaces[now.focused_workspace]
                        .focus_history
                        .iter()
                        .last()
                        .unwrap()
                        .clone();
                    let focused_index = windows
                        .iter()
                        .position(|wid| wid == focused_wid.as_str())
                        .unwrap();
                    match direction.as_str() {
                        "LEFT" => {
                            if focused_index >= left_n {
                                let i = if right_n > focused_index {
                                    0
                                } else {
                                    focused_index - right_n
                                };
                                let prev_window =
                                    now.workspaces[now.focused_workspace].windows[i].clone();
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .retain(|wid| wid != prev_window.as_str());
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .push(prev_window);
                            }
                        }
                        "RIGHT" => {
                            if focused_index < left_n {
                                let prev_window = now.workspaces[now.focused_workspace].windows
                                    [focused_index + left_n]
                                    .clone();
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .retain(|wid| wid != prev_window.as_str());
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .push(prev_window);
                            }
                        }
                        "UP" => {
                            if focused_index != 0 && focused_index != left_n {
                                let prev_window = now.workspaces[now.focused_workspace].windows
                                    [focused_index - 1]
                                    .clone();
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .retain(|wid| wid != prev_window.as_str());
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .push(prev_window);
                            }
                        }
                        "DOWN" => {
                            if focused_index != (left_n - 1) && focused_index != (n - 1) {
                                let prev_window = now.workspaces[now.focused_workspace].windows
                                    [focused_index + 1]
                                    .clone();
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .retain(|wid| wid != prev_window.as_str());
                                now.workspaces[now.focused_workspace]
                                    .focus_history
                                    .push(prev_window);
                            }
                        }
                        _ => {}
                    }
                }
            },
            Event::Scratchpad(event) => {
                match event {
                    ScratchpadEvent::AddWindow(name) => {
                        if let Some(focused_wid) = &now.focused_window {
                            now.workspaces[now.focused_workspace]
                                .windows
                                .retain(|wid| wid != focused_wid.as_str());
                            now.workspaces[now.focused_workspace]
                                .focus_history
                                .retain(|wid| wid != focused_wid.as_str());
                            now.scratchpad.windows.push((name, focused_wid.to_string()));
                        }
                    }
                    ScratchpadEvent::RemoveWindow(name) => {
                        if let Some((_, wid)) = now
                            .scratchpad
                            .windows
                            .iter()
                            .find(|(wname, _)| wname == name.as_str())
                        {
                            now.workspaces[now.focused_workspace]
                                .windows
                                .push(wid.to_string());
                            now.workspaces[now.focused_workspace]
                                .focus_history
                                .push(wid.to_string());
                            now.scratchpad
                                .windows
                                .retain(|(wname, _)| wname != name.as_str());
                        }
                        if let Some(shown) = &now.scratchpad.shown.as_ref() {
                            if shown == &name.as_str() {
                                now.scratchpad.shown = None
                            }
                        }
                    }
                    ScratchpadEvent::ToggleWindow(name) => {
                        if let None = now
                            .scratchpad
                            .windows
                            .iter()
                            .find(|(wname, _)| wname == name.as_str())
                        {
                            continue;
                        }
                        if let Some(wid) = now.scratchpad.shown {
                            // Reset fullscreen if it's current scratchpad window
                            if let Some(fullscreen) =
                                &now.workspaces[now.focused_workspace].fullscreen
                            {
                                if fullscreen == wid.as_str() {
                                    now.workspaces[now.focused_workspace].fullscreen = None;
                                }
                            }
                            if wid == name.as_str() {
                                now.scratchpad.shown = None;
                            } else {
                                now.scratchpad.shown = Some(name);
                            }
                        } else {
                            now.scratchpad.shown = Some(name);
                        }
                    }
                }
            }
            Event::Unknown => continue,
        }

        now.focused_window = actualize_screen(&last_state, &now, &config);
        last_event = event_clone;
        last_state = now;
    }
}
