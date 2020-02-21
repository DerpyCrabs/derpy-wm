use ::derpywm::{
    focus_window, focused_window, parse_event, Event, WindowEventType, WorkspaceEvent,
};
use std::io::{self, BufRead};

mod workspaces;

use workspaces::Workspaces;

const GAP: usize = 5;
const WORKSPACES: usize = 3;

fn main() {
    let mut workspaces = Workspaces::new(WORKSPACES);
    let mut last_event = Event::Unknown;

    for event in io::stdin()
        .lock()
        .lines()
        .map(|ev| parse_event(ev, WORKSPACES))
    {
        let event_clone = event.clone();
        println!("{:#?}", event);
        println!("{:#?}", workspaces);
        match event {
            Event::Window(event) => match event.event_type {
                WindowEventType::FocusIn => {}
                WindowEventType::MapNotify => {
                    if let Event::Window(last_event) = last_event {
                        if last_event.window_id == event.window_id
                            && last_event.event_type == WindowEventType::CreateNotify
                        {
                            focus_window(event.window_id.as_str());
                        }
                    }
                    &workspaces.focused().tile(GAP);
                }
                WindowEventType::CreateNotify => {
                    workspaces.add_window(event.window_id);
                }
                WindowEventType::DestroyNotify => {
                    let changed_workspace = workspaces.delete_window(event.window_id);

                    if changed_workspace == workspaces.focused_workspace {
                        if let Some(window_id) = workspaces.focused().workspace.iter().last() {
                            focus_window(window_id.as_str());
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
                WorkspaceEvent::MoveWindow(workspace) => {
                    if workspace != workspaces.focused_workspace {
                        if let Some(focused_wid) = focused_window() {
                            workspaces.move_window(focused_wid, workspace);
                            &workspaces.focused().tile(GAP);
                            if let Some(window_id) = workspaces.focused().workspace.iter().last() {
                                focus_window(window_id.as_str());
                            }
                        }
                    }
                }
                WorkspaceEvent::Cycle => {
                    if workspaces.focused().workspace.len() > 0 {
                        workspaces.focused_mut().workspace.rotate_right(1);
                        &workspaces.focused().tile(GAP);
                    }
                }
            },
            Event::Unknown => {}
        }
        last_event = event_clone;
    }
}
