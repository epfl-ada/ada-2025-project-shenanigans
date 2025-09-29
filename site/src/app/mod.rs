use std::io::Write;
use std::{cell::RefCell, rc::Rc};

use deser::{SerCol, SerGraph};
use edge::AdaEdgeShape;
use eframe::{App, CreationContext};
use egui::{Color32, Context, Pos2};
use egui_graphs::{
    Graph, GraphView, SettingsInteraction, SettingsNavigation, events::Event,
};
use event_filters::EventFilters;
use flate2::write::GzDecoder;
use node::AdaNodeShape;
use petgraph::Undirected;
use petgraph::csr::DefaultIx;
use petgraph::stable_graph::StableGraph;
use web_sys::console;

mod deser;
mod edge;
mod event_filters;
mod node;

pub struct Adapp {
    g: Graph<(), (), Undirected, DefaultIx, AdaNodeShape, AdaEdgeShape>,
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
                    console::log_1(
                        &format!(
                            "https://www.youtube.com/channel/{}",
                            self.g
                                .node((n.id as u32).into())
                                .unwrap()
                                .props()
                                .label
                        )
                        .into(),
                    );
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
            ctx.set_visuals(egui::Visuals::light());
            ctx.style_mut(|style| {
                style.visuals.widgets.inactive.fg_stroke.color =
                    Color32::GRAY;
            });

            let mut view = GraphView::<_, _, _, _, _, _>::new(&mut self.g)
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

            ui.add(&mut view);
        });

        self.consume_events();
    }
}

fn generate_graph()
-> Graph<(), (), Undirected, DefaultIx, AdaNodeShape, AdaEdgeShape> {
    let gzipped = include_bytes!("../../smallergraph.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let desered: SerGraph =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    let gzipped = include_bytes!("../../colours.json.gz");
    let mut deco = GzDecoder::new(Vec::new());
    deco.write_all(gzipped).unwrap();
    let colours: Vec<SerCol> =
        serde_json::from_slice(&deco.finish().unwrap()).unwrap();

    console::log_1(
        &format!("loading {} edges", desered.edges.len())
            .as_str()
            .into(),
    );
    let mut g = Graph::<
        (),
        (),
        Undirected,
        DefaultIx,
        AdaNodeShape,
        AdaEdgeShape,
    >::from(&StableGraph::<_, _, Undirected>::from_edges(
        &desered.edges,
    ));

    console::log_1(
        &format!("loading {} nodes", desered.nodes.len())
            .as_str()
            .into(),
    );
    desered.nodes.iter().zip(colours).for_each(|(n, c)| {
        assert_eq!(n.id, c.id);
        let node = g.node_mut(n.id.into()).unwrap();
        node.set_label(n.uid.clone());
        node.set_location(Pos2 { x: n.x, y: n.y });
        node.display_mut().set_radius(0.5);
        node.set_color(Color32::from_rgb(c.c[0], c.c[1], c.c[2]));
    });

    console::log_1(&"finished loading!".into());

    g
}
