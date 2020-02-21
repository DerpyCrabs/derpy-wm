use ::derpywm::{
    border_window, focus_window, fullscreen_window, map_window, move_window, unmap_window,
};
use std::ops::Index;

//TODO move all configs in one place
const GAP: usize = 10;
const FOCUSED: &str = "0xff0000";
const UNFOCUSED: &str = "0x888888";

#[derive(Debug, Clone)]
pub struct Workspace {
    pub workspace: Vec<String>,
    focus_history: Vec<String>,
    pub fullscreen: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Workspaces {
    pub workspaces: Vec<Workspace>,
    pub focused_workspace: usize,
}

impl Workspace {
    fn new() -> Workspace {
        Workspace {
            workspace: Vec::new(),
            focus_history: Vec::new(),
            fullscreen: None,
        }
    }
    pub fn tile(&self, gap: usize) {
        let (wsw, wsh) = self.size();

        match self.workspace.len() {
            0 => return,
            1 => {
                let full_w = wsw - 2 * gap;
                let full_h = wsh - 2 * gap;
                move_window(&self.workspace[0], gap, gap, full_w, full_h);
            }
            n => {
                let half_w = (wsw - 3 * gap) / 2;
                let left_n = n / 2;
                let right_n = n - left_n;
                let left_h = (wsh - (left_n + 1) * gap) / left_n;
                let right_h = (wsh - (right_n + 1) * gap) / right_n;
                let mut left_strip = self.workspace.clone();
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
    pub fn focus_window(&mut self, window_id: impl Into<String> + Clone) {
        self.focus_history
            .retain(|wid| wid.as_str() != window_id.clone().into());
        self.focus_history.push(window_id.into());
    }
    pub fn focus_last_window(&self) {
        self.workspace
            .iter()
            .for_each(|wid| border_window(wid, UNFOCUSED));
        if let Some(wid) = self.focus_history.iter().last() {
            border_window(wid, FOCUSED);
            focus_window(wid);
        }
    }
    pub fn fullscreen_window(&mut self, window_id: impl Into<String> + Clone) {
        self.fullscreen = Some(window_id.clone().into());
        fullscreen_window(window_id, self.size());
    }
    pub fn unfullscreen_window(&mut self) {
        if let Some(wid) = &self.fullscreen {
            border_window(wid.clone(), FOCUSED);
            self.tile(GAP);
        }
        self.fullscreen = None;
    }
    fn size(&self) -> (usize, usize) {
        (500, 300)
    }
    pub fn delete_window(&mut self, window_id: impl Into<String> + Clone) {
        self.workspace
            .retain(|wid| wid.as_str() != window_id.clone().into());
        self.focus_history
            .retain(|wid| wid.as_str() != window_id.clone().into());
    }
}

impl Index<usize> for Workspaces {
    type Output = Workspace;
    fn index(&self, workspace: usize) -> &Self::Output {
        &self.workspaces[workspace]
    }
}

impl Workspaces {
    pub fn new(workspace_count: usize) -> Workspaces {
        Workspaces {
            workspaces: vec![Workspace::new(); workspace_count],
            focused_workspace: 0,
        }
    }
    pub fn move_window(&mut self, window_id: impl Into<String> + Clone, to_workspace: usize) {
        unmap_window(window_id.clone().into());
        if let Some(wid) = self.workspaces[to_workspace].focus_history.iter().last() {
            border_window(wid, UNFOCUSED);
        }
        self.focused_mut().unfullscreen_window();
        self.focused_mut()
            .workspace
            .retain(|wid| wid.as_str() != window_id.clone().into().as_str());
        self.workspaces[to_workspace]
            .workspace
            .push(window_id.clone().into());
        self.workspaces[to_workspace].focus_window(window_id.into());
    }
    pub fn focus(&mut self, workspace: usize) {
        if workspace != self.focused_workspace {
            self.focused().workspace.iter().for_each(unmap_window);
            self.focused_workspace = workspace;
            self.workspaces[workspace]
                .workspace
                .iter()
                .for_each(map_window);
            self.workspaces[workspace].focus_last_window();
            self.workspaces[workspace].tile(GAP);
            if let Some(wid) = &self.focused().fullscreen {
                let wid = wid.clone();
                self.focused_mut().fullscreen_window(wid);
            }
        }
    }
    pub fn add_window(&mut self, window_id: impl Into<String>) {
        self.focused_mut().workspace.push(window_id.into());
    }
    pub fn delete_window(&mut self, window_id: impl Into<String> + Clone) -> usize {
        let mut changed_workspace = 0;
        for (i, workspace) in self.workspaces.iter_mut().enumerate() {
            if let Some(_) = workspace
                .workspace
                .iter()
                .find(|wid| wid == &window_id.clone().into().as_str())
            {
                if i == self.focused_workspace {
                    changed_workspace = i;
                }
            }
            workspace.delete_window(window_id.clone());
        }
        changed_workspace
    }
    pub fn focused(&self) -> &Workspace {
        &self.workspaces[self.focused_workspace]
    }
    pub fn focused_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.focused_workspace]
    }
}
