use egui::{Pos2, Shape};
use egui_graphs::{
    DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps,
};
use petgraph::{EdgeType, stable_graph::IndexType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AdaEdgeShape {
    default_impl: DefaultEdgeShape,
}

impl<E: Clone> From<EdgeProps<E>> for AdaEdgeShape {
    fn from(props: EdgeProps<E>) -> Self {
        let mut default_impl = DefaultEdgeShape::from(props);
        default_impl.width = 0.3;
        Self { default_impl }
    }
}

impl<
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
> DisplayEdge<N, E, Ty, Ix, D> for AdaEdgeShape
{
    fn shapes(
        &mut self,
        start: &egui_graphs::Node<N, E, Ty, Ix, D>,
        end: &egui_graphs::Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<Shape> {
        if start.selected()
            || start.dragged()
            || start.hovered()
            || end.selected()
            || end.dragged()
            || end.hovered()
        {
            self.default_impl.shapes(start, end, ctx)
        } else {
            vec![]
        }
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        <DefaultEdgeShape as DisplayEdge<N, E, Ty, Ix, D>>::update(
            &mut self.default_impl,
            state,
        )
    }

    fn extra_bounds(
        &self,
        start: &egui_graphs::Node<N, E, Ty, Ix, D>,
        end: &egui_graphs::Node<N, E, Ty, Ix, D>,
    ) -> Option<(Pos2, Pos2)> {
        <DefaultEdgeShape as DisplayEdge<N, E, Ty, Ix, D>>::extra_bounds(
            &self.default_impl,
            start,
            end,
        )
    }

    fn is_inside(
        &self,
        start: &egui_graphs::Node<N, E, Ty, Ix, D>,
        end: &egui_graphs::Node<N, E, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        <DefaultEdgeShape as DisplayEdge<N, E, Ty, Ix, D>>::is_inside(
            &self.default_impl,
            start,
            end,
            pos,
        )
    }
}
