use ::derpywm::{
    border_window, focus_window, focused_window, parse_event, window_type, Event, ScratchpadEvent,
    WindowEventType, WorkspaceEvent,
};
use std::io::{self, BufRead};

mod scratchpad;
mod workspaces;

use scratchpad::Scratchpad;
use workspaces::Workspaces;

const GAP: usize = 10;
const WORKSPACES: usize = 3;
const FOCUSED: &str = "0xff0000";
const UNFOCUSED: &str = "0x888888";
const IGNORED: &str = "_NET_WM_WINDOW_TYPE_DOCK";

fn main() {
    let mut workspaces = Workspaces::new(WORKSPACES);
    let mut last_event = Event::Unknown;
    let mut scratchpad = Scratchpad::new();

    for event in io::stdin().lock().lines().map(parse_event) {
        let event_clone = event.clone();
        println!("{:#?}", event);
        println!("{:#?}", workspaces);
        match event {
            Event::Window(event) => match event.event_type {
                WindowEventType::FocusIn => {
                    workspaces
                        .focused()
                        .workspace
                        .iter()
                        .for_each(|wid| border_window(wid, UNFOCUSED));
                    border_window(event.window_id.clone(), FOCUSED);
                    workspaces.focused_mut().focus_window(event.window_id);
                }
                WindowEventType::MapNotify => {
                    if let Event::Window(last_event) = last_event {
                        if last_event.window_id == event.window_id
                            && last_event.event_type == WindowEventType::CreateNotify
                        {
                            (|| {
                                if let Some(typ) = window_type(event.window_id.clone()) {
                                    dbg!(typ.clone());
                                    if typ == IGNORED {
                                        workspaces.delete_window(event.window_id.as_str());
                                        return;
                                    }
                                }
                                focus_window(event.window_id.as_str());
                                workspaces
                                    .focused()
                                    .workspace
                                    .iter()
                                    .for_each(|wid| border_window(wid, UNFOCUSED));
                                border_window(event.window_id.clone(), FOCUSED);
                                workspaces.focused_mut().focus_window(event.window_id);
                                &workspaces.focused().tile(GAP);
                            })();
                        }
                    }
                }
                WindowEventType::CreateNotify => {
                    workspaces.add_window(event.window_id.as_str());
                }
                WindowEventType::DestroyNotify => {
                    let changed_workspace = workspaces.delete_window(event.window_id);

                    if changed_workspace == workspaces.focused_workspace {
                        if let Some(window_id) = workspaces.focused().workspace.iter().last() {
                            focus_window(window_id.as_str());
                            border_window(window_id, FOCUSED);
                        }
                        &workspaces.focused().tile(GAP);
                    }
                }
                _ => (),
            },
            Event::Workspace(event) => match event {
                WorkspaceEvent::Focus(workspace) => {
                    workspaces.focus(workspace);
                }
                WorkspaceEvent::FullscreenToggle => {
                    if let Some(_) = workspaces.focused().fullscreen {
                        workspaces.focused_mut().unfullscreen_window();
                    } else {
                        if let Some(focused_wid) = focused_window() {
                            workspaces.focused_mut().fullscreen_window(focused_wid);
                        }
                    }
                }
                WorkspaceEvent::MoveWindow(workspace) => {
                    if workspace != workspaces.focused_workspace {
                        if let Some(focused_wid) = focused_window() {
                            workspaces.move_window(focused_wid, workspace);
                            &workspaces.focused().tile(GAP);
                            if let Some(window_id) = workspaces.focused().workspace.iter().last() {
                                focus_window(window_id.as_str());
                                border_window(window_id, FOCUSED);
                            }
                        }
                    }
                }
                WorkspaceEvent::FocusWindow(direction) => match direction.as_str() {
                    "up" => workspaces.focused().focus_up(),
                    "down" => workspaces.focused().focus_down(),
                    "right" => workspaces.focused().focus_right(),
                    "left" => workspaces.focused().focus_left(),
                    _ => (),
                },
                WorkspaceEvent::Cycle => {
                    if workspaces.focused().workspace.len() > 0 {
                        workspaces.focused_mut().workspace.rotate_right(1);
                        &workspaces.focused().tile(GAP);
                    }
                }
            },
            Event::Scratchpad(event) => match event {
                ScratchpadEvent::AddWindow(name) => {
                    if let Some(focused_wid) = focused_window() {
                        workspaces.delete_window(focused_wid.clone());
                        scratchpad.add_window(name, focused_wid);
                    }
                }
                ScratchpadEvent::RemoveWindow(_name) => {
                    // TODO implement removing window from scratchpad
                }
                ScratchpadEvent::ToggleWindow(name) => {
                    scratchpad.toggle_window(name);
                }
            },
            Event::Unknown => {}
        }
        last_event = event_clone;
    }
}
