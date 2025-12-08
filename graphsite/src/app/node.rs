use egui::{
    Color32, FontFamily, FontId, Pos2, Shape, StrokeKind, Vec2,
    epaint::{CircleShape, RectShape, Stroke, TextShape},
};
use egui_graphs::{DefaultNodeShape, DisplayNode, DrawContext, NodeProps};
use petgraph::{EdgeType, stable_graph::IndexType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaNodeShape {
    default_impl: DefaultNodeShape,
}

impl AdaNodeShape {
    pub fn set_radius(&mut self, radius: f32) {
        self.default_impl.radius = radius;
    }

    fn label_shape(
        galley: std::sync::Arc<egui::Galley>,
        center: Pos2,
        radius: f32,
        color: Color32,
        bagckground: Color32,
    ) -> Shape {
        let label_pos =
            Pos2::new(center.x - galley.size().x / 2., center.y - radius * 2.);
        let label = TextShape::new(label_pos, galley, color);
        let rect = RectShape::new(
            label.visual_bounding_rect().expand(25.0),
            50.0,
            bagckground,
            Stroke::NONE,
            StrokeKind::Outside,
        );
        Shape::Vec(vec![rect.into(), label.into()])
    }
}

impl<E: Clone> From<NodeProps<E>> for AdaNodeShape {
    fn from(props: NodeProps<E>) -> Self {
        Self {
            default_impl: DefaultNodeShape::from(props),
        }
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<N, E, Ty, Ix>
    for AdaNodeShape
{
    fn shapes(&mut self, ctx: &DrawContext) -> Vec<Shape> {
        let mut res = Vec::with_capacity(2);
        let circle_center =
            ctx.meta.canvas_to_screen_pos(self.default_impl.pos);
        let circle_radius =
            ctx.meta.canvas_to_screen_size(self.default_impl.radius);
        let ppp = ctx.ctx.pixels_per_point();

        res.push(
            CircleShape {
                center: circle_center,
                radius: circle_radius,
                fill: self.default_impl.color.unwrap_or(Color32::PLACEHOLDER),
                stroke: Stroke::NONE,
            }
            .into(),
        );

        if !(self.default_impl.selected
            || self.default_impl.dragged
            || self.default_impl.hovered)
        {
            return res;
        }

        let galley = ctx.ctx.fonts(|f| {
            f.layout_no_wrap(
                self.default_impl.label_text.clone(),
                FontId::new(ppp * 10.0, FontFamily::Monospace),
                Color32::PLACEHOLDER,
            )
        });

        res.push(Self::label_shape(
            galley,
            Pos2 {
                x: circle_center.x,
                y: circle_center.y - 20.0 * ppp * circle_radius.sqrt(),
            },
            circle_radius,
            Color32::BLACK,
            Color32::from_white_alpha(204),
        ));

        res
    }

    fn update(&mut self, state: &NodeProps<N>) {
        <DefaultNodeShape as DisplayNode<N, E, Ty, Ix>>::update(
            &mut self.default_impl,
            state,
        )
    }

    fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
        <DefaultNodeShape as DisplayNode<N, E, Ty, Ix>>::closest_boundary_point(
            &self.default_impl,
            dir,
        )
    }

    fn is_inside(&self, pos: Pos2) -> bool {
        <DefaultNodeShape as DisplayNode<N, E, Ty, Ix>>::is_inside(
            &self.default_impl,
            pos,
        )
    }
}
