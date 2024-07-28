use egui::{emath::Numeric, epaint::TextShape, Color32, FontFamily, FontId, Pos2, Shape, Stroke, Vec2};
use petgraph::{csr::IndexType, EdgeType};
use crate::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};

use super::edge_shape_builder::{EdgeShapeBuilder, TipProps};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct GvizorEdgeShape {
    pub default: DefaultEdgeShape,
    pub color: Color32,
}

impl<E: Clone + Numeric> From<EdgeProps<E>> for GvizorEdgeShape {
    fn from(edge: EdgeProps<E>) -> Self {
        Self {
            default: DefaultEdgeShape::from(edge.clone()),
            color: if edge.payload > Numeric::from_f64(0.) {
                Color32::from_hex("f00").unwrap()
            } else {
                Color32::from_hex("00f").unwrap()
            }
        }
    }
}

impl<N: Clone, E: Clone + Numeric, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>
    DisplayEdge<N, E, Ty, Ix, D> for GvizorEdgeShape
{
    fn is_inside(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: egui::Pos2,
    ) -> bool {
        self.default.is_inside(start, end, pos)
    }

    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<egui::Shape> {
        let mut res = vec![];

        let label_visible = ctx.style.labels_always || self.default.selected;

        // let style = match self.default.selected {
        //     true => ctx.ctx.style().visuals.widgets.active,
        //     false => ctx.ctx.style().visuals.widgets.inactive,
        // };

        let color = self.color;
        let stroke = Stroke::new(self.default.width, color);

        if start.id() == end.id() {
            // draw loop
            let size = node_size(start);
            let mut line_looped_shapes = EdgeShapeBuilder::new(stroke)
                .looped(start.location(), size, self.default.loop_size, self.default.order)
                .with_scaler(ctx.meta)
                .build();
            let line_looped_shape = line_looped_shapes.clone().pop().unwrap();
            res.push(line_looped_shape);

            let line_looped = match line_looped_shapes.pop().unwrap() {
                Shape::CubicBezier(cubic) => cubic,
                _ => panic!("Invalid shape type"),
            };

            // TODO: export to func
            if label_visible {
                let galley = ctx.ctx.fonts(|f| {
                    f.layout_no_wrap(
                        self.default.label_text.clone(),
                        FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                        color,
                    )
                });

                let flattened_curve = line_looped.flatten(None);
                let median = *flattened_curve.get(flattened_curve.len() / 2).unwrap();

                let label_width = galley.rect.width();
                let label_height = galley.rect.height();
                let pos = Pos2::new(median.x - label_width / 2., median.y - label_height);

                let label_shape = TextShape::new(pos, galley, color);
                res.push(label_shape.into());
            }
            return res;
        }

        let dir = (end.location() - start.location()).normalized();
        let start_connector_point = start.display().closest_boundary_point(dir);
        let end_connector_point = end.display().closest_boundary_point(-dir);

        if self.default.order == 0 {
            // draw straight edge

            let mut builder = EdgeShapeBuilder::new(stroke)
                .straight((start_connector_point, end_connector_point))
                .with_scaler(ctx.meta);

            let tip_props = TipProps {
                size: self.default.tip_size,
                angle: self.default.tip_angle,
            };
            if ctx.is_directed {
                builder = builder.with_tip(&tip_props);
            };
            let straight_shapes = builder.build();
            res.extend(straight_shapes);

            // TODO: export to func
            if label_visible {
                let size = (node_size(start) + node_size(end)) / 2.;
                let galley = ctx.ctx.fonts(|f| {
                    f.layout_no_wrap(
                        self.default.label_text.clone(),
                        FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                        color,
                    )
                });

                let dist = end_connector_point - start_connector_point;
                let center = ctx
                    .meta
                    .canvas_to_screen_pos(start_connector_point + dist / 2.);
                let label_width = galley.rect.width();
                let label_height = galley.rect.height();
                let pos = Pos2::new(center.x - label_width / 2., center.y - label_height);

                let label_shape = TextShape::new(pos, galley, color);
                res.push(label_shape.into());
            }

            return res;
        }

        let mut builder = EdgeShapeBuilder::new(stroke)
            .curved(
                (start_connector_point, end_connector_point),
                self.default.curve_size,
                self.default.order,
            )
            .with_scaler(ctx.meta);

        let tip_props = TipProps {
            size: self.default.tip_size,
            angle: self.default.tip_angle,
        };
        if ctx.is_directed {
            builder = builder.with_tip(&tip_props);
        };
        let curved_shapes = builder.build();
        let line_curved = match curved_shapes.clone().first() {
            Some(Shape::CubicBezier(curve)) => *curve,
            _ => panic!("Invalid shape type"),
        };
        res.extend(curved_shapes);

        // TODO: export to func
        if label_visible {
            let size = (node_size(start) + node_size(end)) / 2.;
            let galley = ctx.ctx.fonts(|f| {
                f.layout_no_wrap(
                    self.default.label_text.clone(),
                    FontId::new(ctx.meta.canvas_to_screen_size(size), FontFamily::Monospace),
                    color,
                )
            });

            let flattened_curve = line_curved.flatten(None);
            let median = *flattened_curve.get(flattened_curve.len() / 2).unwrap();

            let label_width = galley.rect.width();
            let label_height = galley.rect.height();
            let pos = Pos2::new(median.x - label_width / 2., median.y - label_height);

            let label_shape = TextShape::new(pos, galley, color);
            res.push(label_shape.into());
        }

        res
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        <DefaultEdgeShape as DisplayEdge<N, E, Ty, Ix, D>>::update(&mut self.default, state)
    }
}

fn node_size<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, E, Ty, Ix>>(
    node: &Node<N, E, Ty, Ix, D>,
) -> f32 {
    let left_dir = Vec2::new(-1., 0.);
    let connector_left = node.display().closest_boundary_point(left_dir);
    let connector_right = node.display().closest_boundary_point(-left_dir);

    (connector_right.x - connector_left.x) / 2.
}