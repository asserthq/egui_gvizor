use egui::{emath::Numeric, Color32};
use crate::{DefaultEdgeShape, EdgeProps};

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