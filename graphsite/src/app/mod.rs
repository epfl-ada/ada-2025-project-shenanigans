use std::io::Write;
use std::{cell::RefCell, rc::Rc};

use deser::SerProps;
use edge::AdaEdgeShape;
use eframe::{App, CreationContext};
use egui::{Color32, Context, Pos2};
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
    String,
    usize,
    Undirected,
    u32,
    AdaNodeShape,
    AdaEdgeShape,
    S,
    L,
>;

pub struct Adapp {
    g: Graph<String, usize, Undirected, DefaultIx, AdaNodeShape, AdaEdgeShape>,
    events_buf: Rc<RefCell<Vec<Event>>>,
    event_filters: EventFilters,
}

impl Adapp {
    pub fn new(_: &CreationContext<'_>) -> Self {
        Self {
            g: generate_graph(),
            events_buf: Rc::new(RefCell::new(Vec::new())),
            event_filters: EventFilters::default(),
        }
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
                            "https://{}",
                            self.g
                                .node((n.id as u32).into())
                                .unwrap()
                                .props()
                                .label
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
}

impl App for Adapp {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(egui::Visuals::dark());
            ctx.style_mut(|style| {
                style.visuals.widgets.inactive.fg_stroke.color = Color32::GRAY;
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

            let mut state = GV::get_layout_state(ui);
            state.base.is_running = true;
            state.base.dt = 0.0001;
            state.base.damping = 0.5;
            state.base.k_scale = 1.2;
            GV::set_layout_state(ui, state);

            ui.add(&mut view);
        });

        self.consume_events();
    }
}

fn generate_graph()
-> Graph<String, usize, Undirected, DefaultIx, AdaNodeShape, AdaEdgeShape> {
    let gzipped = include_bytes!("../../sitegraph.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let desered: StableUnGraph<String, usize> =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    let gzipped = include_bytes!("../../sitegraphprops.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let colours: Vec<SerProps> =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    console::log_1(
        &format!("loading {} edges", desered.edge_count())
            .to_string()
            .into(),
    );
    let mut g = Graph::<_, _, _, _, AdaNodeShape, AdaEdgeShape>::from(&desered);

    console::log_1(
        &format!("loading {} nodes", desered.node_count())
            .to_string()
            .into(),
    );

    g.g_mut()
        .node_weights_mut()
        .zip(colours)
        .for_each(|(n, c)| {
            assert_eq!(n.id().index(), c.id);

            n.set_label(n.payload().clone());
            n.set_location(Pos2 { x: c.x, y: c.y });
            n.display_mut().set_radius(2.0);
            n.set_color(Color32::from_rgb(c.l[0], c.l[1], c.l[2]));

            console::log_1(&format!("{:?}", n.location()).into());
        });

    console::log_1(&"finished loading!".into());

    g
}
