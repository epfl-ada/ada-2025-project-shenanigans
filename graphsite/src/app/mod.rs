use std::io::Write;
use std::{cell::RefCell, rc::Rc};

use deser::SerProps;
use edge::AdaEdgeShape;
use eframe::{App, CreationContext};
use egui::{CollapsingHeader, Color32, Context, Pos2, ScrollArea, Ui};
use egui_graphs::{
    FruchtermanReingoldWithCenterGravity,
    FruchtermanReingoldWithCenterGravityState, Graph, GraphView,
    LayoutForceDirected, SettingsInteraction, SettingsNavigation,
    events::Event,
};
use event_filters::EventFilters;
use flate2::write::GzDecoder;
use node::AdaNodeShape;
use petgraph::Undirected;
use petgraph::csr::DefaultIx;
use petgraph::stable_graph::StableUnGraph;
use web_sys::{console, window};

mod deser;
mod edge;
mod event_filters;
mod node;

type S = FruchtermanReingoldWithCenterGravityState;
type L = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
type GV<'a> = GraphView<
    'a,
    (String, String),
    usize,
    Undirected,
    u32,
    AdaNodeShape,
    AdaEdgeShape,
    S,
    L,
>;
type G = Graph<
    (String, String),
    usize,
    Undirected,
    DefaultIx,
    AdaNodeShape,
    AdaEdgeShape,
>;

#[derive(PartialEq)]
enum Palette {
    Connectivity,
    Leiden,
    Category,
    Subscribers,
    Videos,
    Activity,
}

#[derive(PartialEq)]
enum Radius {
    Constant(f32),
    Connectivity(f32),
}

#[derive(PartialEq)]
enum Width {
    Constant(f32),
    Connectivity(f32),
}

pub struct Adapp {
    g: G,
    events_buf: Rc<RefCell<Vec<Event>>>,
    event_filters: EventFilters,
    graph_props: Vec<SerProps>,
    selected_palette: Palette,
    selected_radius: Radius,
    selected_width: Width,
    show_sidebar: bool,
    pending_layout_state: Option<S>,
}

impl Adapp {
    pub fn new(_: &CreationContext<'_>) -> Self {
        let (g, graph_props) = generate_graph();

        let mut state = S::default();
        state.base.is_running = true;
        state.base.dt = 0.003;
        state.base.damping = 0.03;
        state.base.k_scale = 1.2;

        let mut init = Self {
            g: g,
            events_buf: Rc::new(RefCell::new(Vec::new())),
            event_filters: EventFilters::default(),
            graph_props: graph_props,
            selected_palette: Palette::Connectivity,
            selected_radius: Radius::Constant(2.0),
            selected_width: Width::Constant(0.5),
            show_sidebar: true,
            pending_layout_state: Some(state),
        };

        init.colour_graph();
        init.size_nodes();
        init.size_edges();

        init
    }

    fn consume_events(&mut self) {
        let push_event = |e: &Event| {
            if !self.event_filters.enabled_for(e) {
                return;
            }

            match e {
                Event::NodeDoubleClick(n) => {
                    if let Some(w) = window() {
                        let url = format!(
                            // "https://{}",
                            "https://www.youtube.com/channel/{}",
                            self.g
                                .node((n.id as u32).into())
                                .unwrap()
                                .props()
                                .payload
                                .0
                        );
                        w.open_with_url(&url).unwrap();
                    }
                }
                _ => {}
            }
        };

        let mut buf = self.events_buf.borrow_mut();
        for e in buf.drain(..) {
            push_event(&e);
        }
    }

    fn colour_graph(&mut self) {
        self.g
            .g_mut()
            .node_weights_mut()
            .zip(self.graph_props.iter())
            .for_each(|(n, p)| {
                assert_eq!(n.id().index(), p.id);

                let c = match self.selected_palette {
                    Palette::Connectivity => p.c,
                    Palette::Leiden => p.l,
                    Palette::Category => p.a,
                    Palette::Subscribers => p.b,
                    Palette::Videos => p.v,
                    Palette::Activity => p.t,
                };

                n.set_color(Color32::from_rgb(c[0], c[1], c[2]));
            });
    }

    fn size_nodes(&mut self) {
        self.g
            .g_mut()
            .node_weights_mut()
            .zip(self.graph_props.iter())
            .for_each(|(n, p)| {
                assert_eq!(n.id().index(), p.id);

                let r = match self.selected_radius {
                    Radius::Constant(e) => e,
                    Radius::Connectivity(e) => e * p.s,
                };

                n.display_mut().set_radius(r);
            });
    }

    fn size_edges(&mut self) {
        match self.selected_width {
            Width::Constant(a) => {
                self.g.g_mut().edge_weights_mut().for_each(|e| {
                    e.display_mut().set_width(a);
                });
            }
            Width::Connectivity(a) => {
                let max_con = *self
                    .g
                    .g()
                    .edge_weights()
                    .max_by_key(|e| e.payload())
                    .unwrap()
                    .payload() as f32;
                self.g.g_mut().edge_weights_mut().for_each(|e| {
                    let c = *e.payload() as f32;
                    let w = a * f32::powf(c / max_con, 0.5);
                    e.display_mut().set_width(w);
                });
            }
        }
    }

    fn toggle_sidebar_button(&mut self, ui: &mut Ui) {
        let g_rect = ui.max_rect();
        let btn_size = egui::vec2(28.0, 28.0);

        let right_margin = 10.0;
        let bottom_margin = 10.0;
        let toggle_pos = Pos2 {
            x: g_rect.right() - right_margin - btn_size.x,
            y: g_rect.bottom() - bottom_margin - btn_size.y,
        };

        let (arrow, tip) = if self.show_sidebar {
            ("▶", "hide sidebar")
        } else {
            ("◀", "show sidebar")
        };

        egui::Area::new(egui::Id::new("sidebar_toggle"))
            .order(egui::Order::Middle)
            .fixed_pos(toggle_pos)
            .movable(false)
            .show(ui.ctx(), |ui_area| {
                ui_area.set_clip_rect(g_rect);
                let arrow_text = egui::RichText::new(arrow).size(16.0);
                let response =
                    ui_area.add_sized(btn_size, egui::Button::new(arrow_text));
                if response.on_hover_text(tip).clicked() {
                    self.show_sidebar = !self.show_sidebar;
                }
            });
    }

    fn ui_sidebar(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            CollapsingHeader::new("Tutorial").default_open(true).show(ui,

                |ui| {
                    ui.label(
                        "In this visualisation, each dot is a YouTube channel, and each connection between channels is a shared sponsor. Try (double-)clicking on a dot!"
                    );
                    ui.label(
                        "You can also play around with the physics, or change the colour maps to see different channels properties. The different maps are as follows:"
                    );
                    [
                        "Connectivity shows how many neighbors each node has (logarithmic scale)",
                        "Category shows the channel category",
                        "Subscribers and Videos show the number of subscribers and videos (logarithmic scale)",
                        "Activity shows channels active in 2025 in red, others in grey",
                        "Leiden (the most interesting!) shows different community assignments by the Leiden algorithm. It is a community detection algorithm that optimises the modularity of the graph.",
                    ].iter().for_each(|e| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("•");
                            ui.label(e.to_owned());
                        });
                    });
                }
            );
            CollapsingHeader::new("Colours").default_open(true).show(
                ui,
                |ui| {
                    let r1 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Connectivity,
                        "Connectivity",
                    );
                    let r2 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Leiden,
                        "Leiden",
                    );
                    let r3 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Category,
                        "Category",
                    );
                    let r4 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Subscribers,
                        "Subscribers",
                    );
                    let r5 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Videos,
                        "Videos",
                    );
                    let r6 = ui.selectable_value(
                        &mut self.selected_palette,
                        Palette::Activity,
                        "Activity",
                    );

                    if r1.changed()
                        || r2.changed()
                        || r3.changed()
                        || r4.changed()
                        || r5.changed()
                        || r6.changed()
                    {
                        self.colour_graph();
                    }
                },
            );

            CollapsingHeader::new("Node Size").default_open(true).show(
                ui,
                |ui| {
                    let r1 = ui.selectable_value(
                        &mut self.selected_radius,
                        Radius::Constant(2.0),
                        "Constant",
                    );
                    let r2 = ui.selectable_value(
                        &mut self.selected_radius,
                        Radius::Connectivity(4.0),
                        "Connectivity",
                    );
                    let rs = match self.selected_radius {
                        Radius::Constant(ref mut e) => ui.add(
                            egui::Slider::new(e, 0.5..=4.0)
                                .text("constant radius"),
                        ),
                        Radius::Connectivity(ref mut e) => ui.add(
                            egui::Slider::new(e, 1.0..=8.0)
                                .text("connectivity radius"),
                        ),
                    };

                    if r1.changed() || r2.changed() || rs.changed() {
                        self.size_nodes();
                    }
                },
            );

            CollapsingHeader::new("Edge Size").default_open(true).show(
                ui,
                |ui| {
                    let r1 = ui.selectable_value(
                        &mut self.selected_width,
                        Width::Constant(0.5),
                        "Constant",
                    );
                    let r2 = ui.selectable_value(
                        &mut self.selected_width,
                        Width::Connectivity(2.0),
                        "Connectivity",
                    );
                    let rs = match self.selected_width {
                        Width::Constant(ref mut e) => ui.add(
                            egui::Slider::new(e, 0.1..=1.0)
                                .text("constant width"),
                        ),
                        Width::Connectivity(ref mut e) => ui.add(
                            egui::Slider::new(e, 1.0..=8.0)
                                .text("connectivity width"),
                        ),
                    };

                    if r1.changed() || r2.changed() || rs.changed() {
                        self.size_edges();
                    }
                },
            );

            CollapsingHeader::new("Physics").default_open(true).show(
                ui,
                |ui| {
                    let mut state = GV::get_layout_state(ui);

                    ui.add(
                        egui::Slider::new(&mut state.base.k_scale, 0.2..=3.0)
                            .text("k scale"),
                    );

                    GV::set_layout_state(ui, state);
                },
            );

            CollapsingHeader::new("Controls").default_open(true).show(
                ui,
                |ui| {
                    let entries = [
                        ("Ctrl+Scroll", "Zoom"),
                        ("Drag Background", "Pan"),
                        ("Click Node", "Select Node"),
                        ("Double-Click Node", "Open YouTube Channel"),
                    ];
                    egui::Grid::new("keybindings")
                        .num_columns(2)
                        .spacing(egui::vec2(8.0, 4.0))
                        .show(ui, |ui| {
                            for (key, desc) in entries {
                                ui.code(key);
                                ui.label(desc);
                                ui.end_row();
                            }
                        });
                },
            );
        });
    }
}

impl App for Adapp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        if self.show_sidebar {
            // console::log_1(
            //     &format!(
            //         "{:?}",
            //         self.g
            //             .g()
            //             .node_weights()
            //             .map(|e| e.location())
            //             .map(|e| [e.x, e.y])
            //             .collect::<Vec<[f32; 2]>>()
            //     )
            //     .into(),
            // );
            egui::SidePanel::right("right")
                .default_width(300.0)
                .min_width(300.0)
                .show(ctx, |ui| self.ui_sidebar(ui));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut vis = egui::Visuals::dark();
            vis.panel_fill = Color32::from_hex("#0f0f23").unwrap();
            ctx.set_visuals(vis);
            ctx.style_mut(|style| {
                style.visuals.widgets.inactive.fg_stroke.color = Color32::WHITE;
            });

            let mut view = GV::new(&mut self.g)
                .with_navigations(
                    &SettingsNavigation::default()
                        .with_fit_to_screen_enabled(false)
                        .with_zoom_and_pan_enabled(true),
                )
                .with_interactions(
                    &SettingsInteraction::default()
                        .with_node_selection_enabled(true),
                )
                .with_event_sink(&self.events_buf);

            if let Some(state) = self.pending_layout_state.take() {
                GV::set_layout_state(ui, state);
            }

            ui.add(&mut view);

            self.toggle_sidebar_button(ui);
        });

        self.consume_events();
    }
}

fn generate_graph() -> (G, Vec<SerProps>) {
    let gzipped = include_bytes!("../../changraph.json.gz");
    // let gzipped = include_bytes!("../../sitegraph.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let base_graph: StableUnGraph<(String, String), usize> =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    let gzipped = include_bytes!("../../changraphprops.json.gz");
    // let gzipped = include_bytes!("../../sitegraphprops.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let graph_props: Vec<SerProps> =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    console::log_1(
        &format!(
            "loaded {} nodes and {} edges",
            base_graph.node_count(),
            base_graph.edge_count()
        )
        .into(),
    );

    let mut g =
        Graph::<_, _, _, _, AdaNodeShape, AdaEdgeShape>::from(&base_graph);

    g.g_mut()
        .node_weights_mut()
        .zip(graph_props.iter())
        .for_each(|(n, p)| {
            assert_eq!(n.id().index(), p.id);

            n.set_label(n.payload().1.clone());
            n.set_location(Pos2 { x: p.x, y: p.y });
        });

    (g, graph_props)
}
