use gpui::prelude::*;
use gpui_el::{Icon, IconPack, Space, el_icon};

use crate::demo::row;

pub(super) fn explicit_icons() -> Space {
    row()
        .child(el_icon("Plus").size(24.))
        .child(Icon::from_pack(IconPack::ElementPlus, "Plus").size(24.))
        .child(Icon::from_pack(IconPack::Tabler, "plus").size(24.))
}
