use satisfactory_accounting::accounting::{Building, Group};
use yew::prelude::*;

use crate::node_display::Msg;

use super::{NodeDisplay, DRAG_INSERT_POINT};
use group_name::GroupName;

mod group_name;

impl NodeDisplay {
    /// Build the display for a Group.
    pub(super) fn view_group(&self, ctx: &Context<Self>, group: &Group) -> Html {
        let link = ctx.link();
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

        let ondragover = self.drag_over_handler(ctx);
        let ondragleave = link.callback(|_| Msg::DragLeave);
        let ondrop = self.drop_handler(ctx);
        html! {
            <div class="NodeDisplay group">
                <div class="header">
                    {self.drag_handle(ctx)}
                    <GroupName name={group.name.clone()} {rename} />
                    {self.delete_button(ctx)}
                </div>
                <div class="body">
                    <div class="children-display"
                        {ondragover} {ondragleave} {ondrop}
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
                                        move_node={move_node.clone()} />
                                </>
                            }
                        }) }
                        if self.insert_pos == Some(group.children.len()) {
                            <div class={DRAG_INSERT_POINT} />
                        }
                    </div>
                    {self.view_balance(ctx)}
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
}
