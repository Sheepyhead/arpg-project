use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_picking::{Hover, PickableMesh, Selection};

#[derive(Bundle)]
pub struct NoHighlightPickableBundle {
    pub pickable_mesh: PickableMesh,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    pub selection: Selection,
    pub hover: Hover,
}

impl Default for NoHighlightPickableBundle {
    fn default() -> Self {
        Self {
            pickable_mesh: default(),
            interaction: default(),
            focus_policy: FocusPolicy::Block,
            selection: default(),
            hover: default(),
        }
    }
}
