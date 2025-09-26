use deser::SerGraph;
use edge::AdaEdgeShape;
use eframe::{App, CreationContext};
use egui::{Color32, Context, Pos2};
use egui_graphs::{
    Graph, GraphView, SettingsInteraction, SettingsNavigation, events::Event,
};
use event_filters::EventFilters;
use node::AdaNodeShape;
use petgraph::Undirected;
use petgraph::csr::DefaultIx;
use petgraph::stable_graph::StableGraph;
use std::{cell::RefCell, rc::Rc};
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
            ctx.style_mut(|style| {
                style.visuals.widgets.inactive.fg_stroke.color =
                    Color32::DARK_GRAY;
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
    let mut g = Graph::<
        (),
        (),
        Undirected,
        DefaultIx,
        AdaNodeShape,
        AdaEdgeShape,
    >::from(&StableGraph::<_, _, Undirected>::default());

    let desered: SerGraph =
        serde_json::from_str(include_str!("../../graph.json"))
            .unwrap();

    console::log_1(
        &format!("loading {} nodes", desered.nodes.len())
            .as_str()
            .into(),
    );
    desered.nodes.iter().for_each(|n| {
        let id = g.add_node_with_label_and_location(
            (),
            n.name.clone(),
            Pos2 { x: n.x, y: n.y },
        );
        assert_eq!(id, n.id.into());
        let node = g.node_mut(id).unwrap();
        node.display_mut().set_radius(n.size / 5.0);
        node.set_color(Color32::WHITE);
    });

    console::log_1(
        &format!("loading {} edges", desered.edges.len())
            .as_str()
            .into(),
    );
    desered.edges.iter().for_each(|e| {
        g.add_edge(e.source.into(), e.target.into(), ());
    });

    console::log_1(&"finished loading!".into());

    g
}
