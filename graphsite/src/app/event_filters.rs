use egui_graphs::events::Event;

#[derive(Clone)]
pub struct EventFilters {
    pub pan: bool,
    pub zoom: bool,
    pub node_move: bool,
    pub node_drag_start: bool,
    pub node_drag_end: bool,
    pub node_hover_enter: bool,
    pub node_hover_leave: bool,
    pub node_select: bool,
    pub node_deselect: bool,
    pub node_click: bool,
    pub node_double_click: bool,
    pub edge_click: bool,
    pub edge_select: bool,
    pub edge_deselect: bool,
}

impl Default for EventFilters {
    fn default() -> Self {
        Self {
            pan: false,
            zoom: false,
            node_move: false,
            node_drag_start: false,
            node_drag_end: false,
            node_hover_enter: false,
            node_hover_leave: false,
            node_select: false,
            node_deselect: false,
            node_click: false,
            node_double_click: true,
            edge_click: false,
            edge_select: false,
            edge_deselect: false,
        }
    }
}

impl EventFilters {
    pub fn enabled_for(&self, e: &Event) -> bool {
        use Event::*;
        match e {
            Pan(_) => self.pan,
            Zoom(_) => self.zoom,
            NodeMove(_) => self.node_move,
            NodeDragStart(_) => self.node_drag_start,
            NodeDragEnd(_) => self.node_drag_end,
            NodeHoverEnter(_) => self.node_hover_enter,
            NodeHoverLeave(_) => self.node_hover_leave,
            NodeSelect(_) => self.node_select,
            NodeDeselect(_) => self.node_deselect,
            NodeClick(_) => self.node_click,
            NodeDoubleClick(_) => self.node_double_click,
            EdgeClick(_) => self.edge_click,
            EdgeSelect(_) => self.edge_select,
            EdgeDeselect(_) => self.edge_deselect,
        }
    }
}
