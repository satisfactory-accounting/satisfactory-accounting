// Copyright 2021, 2022 Zachary Stewart
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
use satisfactory_accounting::accounting::{Building, Group};
use yew::prelude::*;

use crate::node_display::copies::VirtualCopies;
use crate::node_display::{Msg, NodeDisplay, NodeMeta, DRAG_INSERT_POINT};
use crate::CtxHelper;

use group_name::GroupName;

mod group_name;

impl NodeDisplay {
    /// Build the display for a Group.
    pub(super) fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let meta = ctx.meta(group.id);
        if meta.collapsed {
            self.view_group_collapsed(ctx, group)
        } else {
            self.view_group_expanded(ctx, group)
        }
    }

    /// Get the expanded view of a group.
    fn view_group_expanded(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let link = ctx.link();
        let update_copies = link.callback(|copies| Msg::SetCopyCount { copies });
        let replace = link.callback(|(idx, replacement)| Msg::ReplaceChild { idx, replacement });
        let delete = link.callback(|idx| Msg::DeleteChild { idx });
        let copy = link.callback(|idx| Msg::CopyChild { idx });
        let move_node = link.callback(|(src_path, dest_path)| Msg::MoveNode {
            src_path,
            dest_path,
        });
        let add_group = link.callback(|_| Msg::AddChild {
            child: Group::empty_node(),
        });
        let add_building = link.callback(|_| Msg::AddChild {
            child: Building::empty_node(),
        });
        let rename = link.callback(|name| Msg::Rename { name });

        let ondragover = self.drag_over_handler(ctx, |insert_pos| Msg::DragOver { insert_pos });
        let ondragenter = self.drag_over_handler(ctx, |insert_pos| Msg::DragEnter { insert_pos });
        let ondragleave = link.callback(|_| Msg::DragLeave);
        let ondrop = self.drop_handler(ctx);

        let set_metadata = &ctx.props().set_metadata;
        let batch_set_metadata = &ctx.props().batch_set_metadata;
        html! {
            <div class="NodeDisplay group expanded" key={group.id.as_u128()}>
                <div class="header">
                    {self.drag_handle(ctx)}
                    <GroupName name={group.name.clone()} {rename} />
                    {self.child_warnings(ctx)}
                    {self.collapse_button(ctx, group)}
                    if !ctx.props().path.is_empty() {
                        <VirtualCopies copies={group.copies} {update_copies} />
                    }
                    {self.copy_button(ctx)}
                    {self.delete_button(ctx)}
                </div>
                <div class="body">
                    <div class="children-display"
                        {ondragover} {ondragenter} {ondragleave} {ondrop}
                        ref={self.children.clone()}>
                        { for group.children.iter().cloned().enumerate().map(|(i, node)| {
                            let mut path = ctx.props().path.clone();
                            path.push(i);
                            html! {
                                <>
                                    if self.insert_pos == Some(i) {
                                        <div class={DRAG_INSERT_POINT} />
                                    }
                                    <NodeDisplay {node} {path}
                                        replace={replace.clone()}
                                        delete={delete.clone()}
                                        copy={copy.clone()}
                                        move_node={move_node.clone()}
                                        set_metadata={set_metadata.clone()}
                                        batch_set_metadata={batch_set_metadata.clone()} />
                                </>
                            }
                        }) }
                        if self.insert_pos == Some(group.children.len()) {
                            <div class={DRAG_INSERT_POINT} />
                        }
                    </div>
                    {self.view_balance(ctx, true)}
                </div>
                <div class="footer">
                    <button class="create create-group" title="Add Group"
                        onclick={add_group}>
                        <span class="material-icons">{"create_new_folder"}</span>
                    </button>
                    <button class="create create-building" title="Add Building"
                        onclick={add_building}>
                        <span class="material-icons">{"add"}</span>
                    </button>
                </div>
            </div>
        }
    }

    fn view_group_collapsed(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let rename = ctx.link().callback(|name| Msg::Rename { name });
        let update_copies = ctx.link().callback(|copies| Msg::SetCopyCount { copies });
        html! {
            <div class="NodeDisplay group collapsed" key={group.id.as_u128()}>
                <div class="summary">
                    {self.drag_handle(ctx)}
                    <GroupName name={group.name.clone()} {rename} />
                    {self.view_balance(ctx, false)}
                    {self.child_warnings(ctx)}
                    {self.collapse_button(ctx, group)}
                    if !ctx.props().path.is_empty() {
                        <VirtualCopies copies={group.copies} {update_copies} />
                    }
                    {self.copy_button(ctx)}
                    {self.delete_button(ctx)}
                </div>
            </div>
        }
    }

    /// Get a collapse/expand button for this node.
    fn collapse_button(&self, ctx: &Context<Self>, group: &Group) -> Html {
        if ctx.props().path.is_empty() {
            // No collapse for root.
            html! {}
        } else {
            let meta = ctx.meta(group.id);
            let set_metadata = ctx.props().set_metadata.clone();
            let update = (
                group.id,
                NodeMeta {
                    collapsed: !meta.collapsed,
                    ..meta.clone()
                },
            );
            let onclick = Callback::from(move |_| set_metadata.emit(update.clone()));
            let title = if meta.collapsed { "Expand" } else { "Collapse" };
            html! {
                <button class="expand-collapse" {onclick} {title}>
                    <span class="material-icons">
                        if meta.collapsed {
                            {"expand_more"}
                        } else {
                            {"expand_less"}
                        }
                    </span>
                </button>
            }
        }
    }

    /// Show an icon to notify if any children have warnings.
    fn child_warnings(&self, ctx: &Context<Self>) -> Html {
        if ctx.props().node.children_had_warnings() {
            html! {
                <span class="BuildError material-icons warning"
                    title="One or more children had errors">
                    {"warning"}
                </span>
            }
        } else {
            html! {}
        }
    }
}
