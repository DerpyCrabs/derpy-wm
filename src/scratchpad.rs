use ::derpywm::{border_window, focus_window, map_window, move_window, unmap_window};

const FOCUSED: &str = "0xff0000";

pub struct Scratchpad {
    windows: Vec<(String, String)>,
    shown: Option<(String, String)>,
}

impl Scratchpad {
    pub fn new() -> Scratchpad {
        Scratchpad {
            windows: Vec::new(),
            shown: None,
        }
    }
    pub fn add_window(&mut self, name: impl Into<String>, window_id: impl Into<String> + Clone) {
        // TODO don't retain many windows with same name
        self.windows.push((name.into(), window_id.clone().into()));
        unmap_window(window_id);
    }
    pub fn remove_window(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.windows.retain(|(wname, _)| wname != name.as_str());
    }
    pub fn toggle_window(&mut self, name: impl Into<String> + Clone) {
        if let Some((wname, wid)) = &self.shown {
            unmap_window(wid.clone());
            if wname == name.clone().into().as_str() {
                self.shown = None;
            } else {
                if let Some((wname, wid)) = self
                    .windows
                    .iter()
                    .find(|(wname, _)| wname == name.clone().into().as_str())
                {
                    map_window(wid.clone());
                    border_window(wid.clone(), FOCUSED);
                    // TODO get workspace size somehow
                    // TODO make scratchpad window size configurable
                    move_window(wid.clone(), 200, 100, 100, 100);
                    focus_window(wid);
                    self.shown = Some((name.clone().into(), wid.clone()));
                }
            }
        } else {
            if let Some((wname, wid)) = self
                .windows
                .iter()
                .find(|(wname, _)| wname == name.clone().into().as_str())
            {
                map_window(wid.clone());
                border_window(wid.clone(), FOCUSED);
                // TODO get workspace size somehow
                // TODO make scratchpad window size configurable
                move_window(wid.clone(), 200, 100, 100, 100);
                focus_window(wid);
                self.shown = Some((name.clone().into(), wid.clone()));
            }
        }
    }
}
