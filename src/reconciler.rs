use ::derpywm::{
    border_window, focus_window, foreground_window, fullscreen_window, map_window, move_window,
    tile_windows, unmap_window,
};

const GAP: usize = 10;
const FOCUSED: &str = "0xff0000";
const UNFOCUSED: &str = "0x888888";
// TODO get size for every workspace
const SIZE: (usize, usize) = (500, 300);
const SCRATCHPAD_SIZE: (usize, usize) = (150, 100);
const PANEL: usize = 16;

pub type ScratchpadName = String;
pub type WindowId = String;

#[derive(Debug, Clone)]
pub struct ScratchpadState {
    pub windows: Vec<(ScratchpadName, WindowId)>,
    pub shown: Option<ScratchpadName>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub windows: Vec<WindowId>,
    pub focus_history: Vec<WindowId>,
    pub fullscreen: Option<WindowId>,
}

#[derive(Debug, Clone)]
pub struct WMState {
    pub workspaces: Vec<WorkspaceState>,
    pub scratchpad: ScratchpadState,
    pub focused_workspace: usize,
    pub focused_window: Option<WindowId>,
}

impl WMState {
    pub fn new(workspace_count: usize) -> WMState {
        WMState {
            workspaces: vec![
                WorkspaceState {
                    windows: Vec::new(),
                    focus_history: Vec::new(),
                    fullscreen: None
                };
                workspace_count
            ],
            scratchpad: ScratchpadState {
                windows: Vec::new(),
                shown: None,
            },
            focused_workspace: 0,
            focused_window: None,
        }
    }
}

pub fn actualize_screen(before: &WMState, now: &WMState) -> Option<WindowId> {
    // Focus workspace if focused_workspace changed
    if before.focused_workspace != now.focused_workspace {
        before.workspaces[before.focused_workspace]
            .windows
            .iter()
            .for_each(unmap_window);
        now.workspaces[now.focused_workspace]
            .windows
            .iter()
            .for_each(map_window);
    }

    // Show scratchpad
    if before.scratchpad.shown != now.scratchpad.shown {
        if let None = before.scratchpad.shown {
            // Show new window
            let shown = now.scratchpad.shown.as_ref().unwrap();
            let shown_wid = now
                .scratchpad
                .windows
                .iter()
                .find(|(name, _)| name == shown.as_str())
                .unwrap()
                .1
                .as_str();

            map_window(shown_wid);
            move_window(
                shown_wid,
                (SIZE.0 - SCRATCHPAD_SIZE.0) / 2,
                (SIZE.1 - SCRATCHPAD_SIZE.1) / 2,
                SCRATCHPAD_SIZE.0,
                SCRATCHPAD_SIZE.1,
            );
            foreground_window(shown_wid);
            border_window(shown_wid, FOCUSED);
        } else {
            // Need to hide shown before window
            let shown_before = before.scratchpad.shown.as_ref().unwrap();
            let shown_before_wid = before
                .scratchpad
                .windows
                .iter()
                .find(|(name, _)| name == shown_before.as_str())
                .unwrap()
                .1
                .as_str();
            unmap_window(shown_before_wid);

            // Show new window if needed
            if let Some(shown) = &now.scratchpad.shown {
                let shown_wid = now
                    .scratchpad
                    .windows
                    .iter()
                    .find(|(name, _)| name == shown.as_str())
                    .unwrap()
                    .1
                    .as_str();

                map_window(shown_wid);
                move_window(
                    shown_wid,
                    (SIZE.0 - SCRATCHPAD_SIZE.0) / 2,
                    (SIZE.1 - SCRATCHPAD_SIZE.1) / 2,
                    SCRATCHPAD_SIZE.0,
                    SCRATCHPAD_SIZE.1,
                );
                foreground_window(shown_wid);
                border_window(shown_wid, FOCUSED);
            }
        }
    }

    // Unmap moved from workspace window
    for window in &before.workspaces[now.focused_workspace].windows {
        if !now.workspaces[now.focused_workspace]
            .windows
            .contains(window)
        {
            unmap_window(window);
        }
    }
    // Map moved to workspace window
    for window in &now.workspaces[now.focused_workspace].windows {
        if !before.workspaces[now.focused_workspace]
            .windows
            .contains(window)
        {
            map_window(window);
        }
    }

    // Add border if window added to focused workspace
    if before.workspaces[now.focused_workspace].windows.len()
        < now.workspaces[now.focused_workspace].windows.len()
    {
        border_window(
            now.workspaces[now.focused_workspace]
                .windows
                .iter()
                .last()
                .unwrap(),
            UNFOCUSED,
        );
    }

    // Add border if window got unfullscreened
    if before.workspaces[now.focused_workspace]
        .fullscreen
        .is_some()
        && now.workspaces[now.focused_workspace].fullscreen.is_none()
    {
        let wid = before.workspaces[now.focused_workspace]
            .fullscreen
            .as_ref()
            .unwrap();
        border_window(wid.clone(), UNFOCUSED);
    }

    // Tile windows if focused workspace windows changed
    // or window got unfullscreened
    // or scratchpad windows changed
    if (before.workspaces[before.focused_workspace].windows
        != now.workspaces[now.focused_workspace].windows)
        || (before.workspaces[now.focused_workspace]
            .fullscreen
            .is_some()
            && now.workspaces[now.focused_workspace].fullscreen.is_none())
        || (now.scratchpad.windows != before.scratchpad.windows)
    {
        tile_windows(
            now.workspaces[now.focused_workspace].windows.clone(),
            GAP,
            SIZE,
            PANEL,
        );
    }

    // Show scratchpad if scratchpad window was unfullscreened
    if let Some(fullscreen) = &before.workspaces[now.focused_workspace].fullscreen {
        if let Some(scratchpad) = &now.scratchpad.shown {
            let scratchpad_wid = now
                .scratchpad
                .windows
                .iter()
                .find(|(name, _)| name == scratchpad.as_str())
                .unwrap()
                .1
                .as_str();
            if scratchpad_wid == fullscreen.as_str()
                && now.workspaces[now.focused_workspace].fullscreen.is_none()
            {
                map_window(scratchpad_wid);
                move_window(
                    scratchpad_wid,
                    (SIZE.0 - SCRATCHPAD_SIZE.0) / 2,
                    (SIZE.1 - SCRATCHPAD_SIZE.1) / 2,
                    SCRATCHPAD_SIZE.0,
                    SCRATCHPAD_SIZE.1,
                );
                foreground_window(scratchpad_wid);
                border_window(scratchpad_wid, FOCUSED);
            }
        }
    }

    // Show fullscreen window
    if before.workspaces[now.focused_workspace].fullscreen
        != now.workspaces[now.focused_workspace].fullscreen
        || before.focused_workspace != now.focused_workspace
    {
        if let Some(fullscreen) = &now.workspaces[now.focused_workspace].fullscreen {
            fullscreen_window(fullscreen, SIZE);
            foreground_window(fullscreen);
        }
    }

    // Actualize focus
    // Focus window if it became fullscreen or on workspace focus change
    if (before.workspaces[now.focused_workspace]
        .fullscreen
        .is_none()
        && now.workspaces[now.focused_workspace].fullscreen.is_some())
        || (before.focused_workspace != now.focused_workspace
            && now.workspaces[now.focused_workspace].fullscreen.is_some())
    {
        let fullscreen = now.workspaces[now.focused_workspace]
            .fullscreen
            .as_ref()
            .unwrap();
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, UNFOCUSED);
        }
        // Focus new window
        focus_window(fullscreen);
        return Some(fullscreen.to_string());
    }
    // No need to refocus fullscreen windows
    if now.workspaces[now.focused_workspace].fullscreen.is_some() {
        return now.focused_window.clone();
    }
    // Focus scratchpad window if shown
    if (before.scratchpad.shown != now.scratchpad.shown
        || before.focused_workspace != now.focused_workspace)
        && now.scratchpad.shown.is_some()
    {
        let shown = now.scratchpad.shown.as_ref().unwrap();
        let shown_wid = now
            .scratchpad
            .windows
            .iter()
            .find(|(name, _)| name == shown.as_str())
            .unwrap()
            .1
            .as_str();
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, UNFOCUSED);
        }
        // Focus new window
        border_window(shown_wid, FOCUSED);
        focus_window(shown_wid);
        return Some(shown_wid.to_string());
    }
    // No need to refocus scratchpad windows
    if now.scratchpad.shown.is_some() {
        return now.focused_window.clone();
    }
    // Focus last window in focus history
    if let Some(wid) = now.workspaces[now.focused_workspace]
        .focus_history
        .iter()
        .last()
    {
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, UNFOCUSED);
        }
        // Focus new window
        border_window(wid, FOCUSED);
        focus_window(wid);
        return Some(wid.to_string());
    }
    now.focused_window.clone()
}
