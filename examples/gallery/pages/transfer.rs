use gpui::{Context, Div, SharedString, div, prelude::*, px};
use gpui_el::{TransferItem, TransferTargetOrder, el_button, el_icon, el_transfer};

use crate::{Gallery, copy::Copy, demo::row};

pub(crate) struct State {
    basic: Vec<SharedString>,
    filterable: Vec<SharedString>,
    ordered: Vec<SharedString>,
    custom: Vec<SharedString>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            basic: vec!["alpha".into()],
            filterable: vec!["beta".into()],
            ordered: vec!["alpha".into()],
            custom: vec!["gamma".into()],
        }
    }
}

fn data() -> Vec<TransferItem> {
    vec![
        TransferItem::new("alpha", "Alpha"),
        TransferItem::new("beta", "Beta"),
        TransferItem::new("gamma", "Gamma"),
        TransferItem::new("delta", "Delta").disabled(true),
        TransferItem::new("epsilon", "Epsilon"),
    ]
}

impl Gallery {
    pub(crate) fn transfers(&self, s: &'static Copy, cx: &Context<Self>) -> Div {
        let view = cx.entity();
        let field =
            |slot: usize, id: gpui::ElementId, value: Vec<SharedString>| {
                let view = view.clone();
                el_transfer().id(id).data(data()).value(value).onchange(
                    move |value, _, _, _, cx| {
                        view.update(cx, |this, cx| {
                            match slot {
                                0 => this.transfer.basic = value,
                                1 => this.transfer.filterable = value,
                                2 => this.transfer.ordered = value,
                                3 => this.transfer.custom = value,
                                _ => unreachable!("unknown transfer demo slot"),
                            }
                            cx.notify();
                        })
                    },
                )
            };
        let basic = row().child(field(
            0,
            "transfer-basic".into(),
            self.transfer.basic.clone(),
        ));
        let filterable = row().child(
            field(
                1,
                "transfer-filterable".into(),
                self.transfer.filterable.clone(),
            )
            .filterable(true),
        );
        let ordered = row().child(
            field(2, "transfer-ordered".into(), self.transfer.ordered.clone())
                .target_order(TransferTargetOrder::Unshift),
        );
        let custom = row().child(
            field(3, "transfer-custom".into(), self.transfer.custom.clone())
                .titles(["Available".into(), "Chosen".into()])
                .button_texts(["Add".into(), "Remove".into()])
                .filterable(true)
                .filter_placeholder("Find item")
                .left_footer(div().child("Custom left footer"))
                .right_footer(div().child("Custom right footer"))
                .left_empty(div().child("Nothing available"))
                .right_empty(div().child("Nothing chosen")),
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(self.section(
                cx,
                "transfer-basic",
                &s.transfer_basic,
                || "el_transfer().data(items).value(vec![\"alpha\".into()]);".into(),
                basic.into_any_element(),
            ))
            .child(self.section(
                cx,
                "transfer-filterable",
                &s.transfer_filterable,
                || "el_transfer().filterable(true);".into(),
                filterable.into_any_element(),
            ))
            .child(self.section(
                cx,
                "transfer-target-order",
                &s.transfer_target_order,
                || "el_transfer().target_order(TransferTargetOrder::Unshift);".into(),
                ordered.into_any_element(),
            ))
            .child(self.section(
                cx,
                "transfer-custom",
                &s.transfer_custom,
                || "el_transfer().titles([\"Available\".into(), \"Chosen\".into()]).button_texts([\"Add\".into(), \"Remove\".into()]);".into(),
                custom.into_any_element(),
            ))
            .child(
                div().flex().items_center().gap(px(12.)).child(
                    el_button()
                        .id("transfer-reset")
                        .small()
                        .label(s.tag_reset.as_str())
                        .icon(el_icon("refresh-left"))
                        .onclick(cx.listener(|this, _, _, cx| {
                            this.transfer = Default::default();
                            cx.notify();
                        })),
                ),
            )
    }
}
