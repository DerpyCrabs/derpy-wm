mod reconciler;

use ::derpywm::{parse_event, Event, ScratchpadEvent, WindowEventType, WorkspaceEvent};
use reconciler::{actualize_screen, WMState};

use std::io::{self, BufRead};

const WORKSPACES: usize = 3;

fn main() {
    let mut last_event = Event::Unknown;
    let mut last_state = WMState::new(WORKSPACES);

    for event in io::stdin().lock().lines().map(parse_event) {
        let mut now = last_state.clone();
        let event_clone = event.clone();

        dbg!(event.clone());

        match event {
            Event::Window(event) => match event.event_type {
                WindowEventType::CreateNotify => {
                    now.workspaces[now.focused_workspace]
                        .windows
                        .push(event.window_id.clone());
                }
                WindowEventType::MapNotify => {
                    if let Event::Window(last_event) = last_event {
                        if last_event.window_id == event.window_id
                            && last_event.event_type == WindowEventType::CreateNotify
                        {
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
                WorkspaceEvent::FocusWindow(direction) => {}
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

        now.focused_window = actualize_screen(&last_state, &now);
        last_event = event_clone;
        last_state = now;
    }
}
