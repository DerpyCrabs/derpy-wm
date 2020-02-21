use ::derpywm::{map_window, unmap_window};
use std::ops::Index;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub workspace: Vec<String>,
    focus_history: Vec<String>,
    fullscreen: Option<String>,
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
        unmap_window(window_id.clone().into().as_str());
        self.workspaces[self.focused_workspace]
            .workspace
            .retain(|wid| wid.as_str() != window_id.clone().into().as_str());
        self.workspaces[to_workspace]
            .workspace
            .push(window_id.into());
    }
    pub fn focus(&mut self, workspace: usize) {
        if workspace != self.focused_workspace {
            self.focused().workspace.iter().for_each(unmap_window);
            self.focused_workspace = workspace;
            self.workspaces[workspace]
                .workspace
                .iter()
                .for_each(map_window);
            // TODO Focus last window in workspace history
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
            workspace
                .workspace
                .retain(|wid| wid.as_str() != window_id.clone().into());
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
