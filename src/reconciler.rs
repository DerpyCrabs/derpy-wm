use ::derpywm::{
    border_window, focus_window, foreground_window, fullscreen_window, map_window, move_window,
    tile_windows, unmap_window, Config,
};

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

pub fn actualize_screen(before: &WMState, now: &WMState, config: &Config) -> Option<WindowId> {
    let focused_workspace_before = &before.workspaces[now.focused_workspace];
    let focused_workspace = &now.workspaces[now.focused_workspace];
    // Focus workspace if focused_workspace changed
    if before.focused_workspace != now.focused_workspace {
        before.workspaces[before.focused_workspace]
            .windows
            .iter()
            .for_each(unmap_window);
        focused_workspace.windows.iter().for_each(map_window);
    }

    // Show scratchpad
    if before.scratchpad.shown != now.scratchpad.shown {
        if let None = before.scratchpad.shown {
            show_scratchpad(
                &now.scratchpad,
                config.workspace_size,
                config.scratchpad_size,
                config.focused_border.as_str(),
            );
        } else {
            // Need to hide shown before window
            let shown_before_wid = scratchpad_shown_wid(&before.scratchpad).unwrap();
            unmap_window(shown_before_wid);

            // Show new window if needed
            show_scratchpad(
                &now.scratchpad,
                config.workspace_size,
                config.scratchpad_size,
                config.focused_border.as_str(),
            );
        }
    }

    // Unmap moved from workspace window
    for window in &focused_workspace_before.windows {
        if !focused_workspace.windows.contains(window) {
            unmap_window(window);
        }
    }
    // Map moved to workspace window
    for window in &focused_workspace.windows {
        if !focused_workspace_before.windows.contains(window) {
            map_window(window);
        }
    }

    // Add border if window added to focused workspace
    if focused_workspace_before.windows.len() < focused_workspace.windows.len() {
        border_window(
            focused_workspace.windows.iter().last().unwrap(),
            config.unfocused_border.as_str(),
        );
    }

    // Add border if window got unfullscreened
    if focused_workspace_before.fullscreen.is_some() && focused_workspace.fullscreen.is_none() {
        let wid = focused_workspace_before.fullscreen.as_ref().unwrap();
        border_window(wid.as_str(), config.unfocused_border.as_str());
    }

    // Tile windows if focused workspace windows changed
    // or window got unfullscreened
    // or scratchpad windows changed
    if (before.workspaces[before.focused_workspace].windows != focused_workspace.windows)
        || (focused_workspace_before.fullscreen.is_some() && focused_workspace.fullscreen.is_none())
        || (now.scratchpad.windows != before.scratchpad.windows)
    {
        tile_windows(
            focused_workspace.windows.clone(),
            config.gaps,
            config.workspace_size,
            config.panel_width,
        );
    }

    // Show scratchpad if scratchpad window was unfullscreened
    if let Some(fullscreen) = &focused_workspace_before.fullscreen {
        if let Some(scratchpad_wid) = scratchpad_shown_wid(&now.scratchpad) {
            if scratchpad_wid == fullscreen.as_str() && focused_workspace.fullscreen.is_none() {
                show_scratchpad(
                    &now.scratchpad,
                    config.workspace_size,
                    config.scratchpad_size,
                    config.focused_border.as_str(),
                );
            }
        }
    }

    // Show fullscreen window
    if focused_workspace_before.fullscreen != focused_workspace.fullscreen
        || before.focused_workspace != now.focused_workspace
    {
        if let Some(fullscreen) = &focused_workspace.fullscreen {
            fullscreen_window(fullscreen, config.workspace_size);
            foreground_window(fullscreen);
        }
    }

    // Actualize focus
    // Focus window if it became fullscreen or on workspace focus change
    if (focused_workspace_before.fullscreen.is_none() && focused_workspace.fullscreen.is_some())
        || (before.focused_workspace != now.focused_workspace
            && focused_workspace.fullscreen.is_some())
    {
        let fullscreen = focused_workspace.fullscreen.as_ref().unwrap();
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, config.unfocused_border.as_str());
        }
        // Focus new window
        focus_window(fullscreen);
        return Some(fullscreen.to_string());
    }
    // No need to refocus fullscreen windows
    if focused_workspace.fullscreen.is_some() {
        return now.focused_window.clone();
    }
    // Focus scratchpad window if shown
    if (before.scratchpad.shown != now.scratchpad.shown
        || before.focused_workspace != now.focused_workspace)
        && now.scratchpad.shown.is_some()
    {
        let shown_wid = scratchpad_shown_wid(&now.scratchpad).unwrap();
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, config.unfocused_border.as_str());
        }
        // Focus new window
        border_window(shown_wid, config.focused_border.as_str());
        focus_window(shown_wid);
        return Some(shown_wid.to_string());
    }
    // No need to refocus scratchpad windows
    if now.scratchpad.shown.is_some() {
        return now.focused_window.clone();
    }
    // Focus last window in focus history
    if let Some(wid) = focused_workspace.focus_history.iter().last() {
        // Unfocus previous window
        if let Some(focused_window) = &now.focused_window {
            border_window(focused_window, config.unfocused_border.as_str());
        }
        // Focus new window
        border_window(wid, config.focused_border.as_str());
        focus_window(wid);
        return Some(wid.to_string());
    }
    now.focused_window.clone()
}

fn scratchpad_shown_wid(scratchpad: &ScratchpadState) -> Option<&str> {
    if let Some(shown) = &scratchpad.shown {
        Some(
            scratchpad
                .windows
                .iter()
                .find(|(name, _)| name == shown.as_str())
                .unwrap()
                .1
                .as_str(),
        )
    } else {
        None
    }
}
fn show_scratchpad(
    scratchpad: &ScratchpadState,
    workspace_size: (usize, usize),
    scratchpad_size: (usize, usize),
    border: impl Into<String>,
) {
    if let Some(shown) = &scratchpad.shown {
        let shown_wid = scratchpad
            .windows
            .iter()
            .find(|(name, _)| name == shown.as_str())
            .unwrap()
            .1
            .as_str();

        map_window(shown_wid);
        move_window(
            shown_wid,
            (workspace_size.0 - scratchpad_size.0) / 2,
            (workspace_size.1 - scratchpad_size.1) / 2,
            scratchpad_size.0,
            scratchpad_size.1,
        );
        foreground_window(shown_wid);
        border_window(shown_wid, border);
    }
}
