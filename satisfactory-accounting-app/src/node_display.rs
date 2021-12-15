use std::ops::Deref;
use std::rc::Rc;

use yew::prelude::*;

use satisfactory_accounting::accounting::Node;
use satisfactory_accounting::accounting::refs::{NodeKindRef, GroupRef};

/// Path to a node in the tree.
#[derive(Clone, PartialEq, Debug)]
pub struct NodePath(Rc<Vec<usize>>);

impl NodePath {
    /// Root node in the tree.
    fn root() -> Self {
        Self(Rc::new(Vec::new()))
    }

    /// Get the path for a child with the given index.
    fn child(&self, index: usize) -> Self {
        let mut path = self.0.deref().clone();
        path.push(index);
        Self(Rc::new(path))
    }

    /// Returns true if this node is the root.
    fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Index of this element in the parent. For the root node, returns zero.
    fn index(&self) -> usize {
        self.0.last().copied().unwrap_or_default()
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct NodeDisplayProperties {
    /// The node to display.
    pub node: Node,
    /// Path to this node in the tree.
    pub index: Vec<usize>,
}

/// Display for a single AccountingGraph node.
pub struct NodeDisplay {}

impl Component for NodeDisplay {
    type Message = ();
    type Properties = NodeDisplayProperties;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().node.kind_ref() {
            NodeKindRef::Group(group) => {
                html! {
                    <GroupDisplay group={group} />
                }
            }
            NodeKindRef::Building(building) => {
                html! {}
            }
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct GroupDisplayProperties {
    group: GroupRef,
}

pub struct GroupDisplay {}

impl Component for GroupDisplay {
    type Message = ();
    type Properties = GroupDisplayProperties;

    fn create(_: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let group = &*ctx.props().group;
        html! {
            <div class="Group">
                <div class="header">
                    if let Some(name) = &group.name {
                        <span class="name">{name}</span>
                    } else {
                        <span class="name notset">{"unnamed"}</span>
                    }
                    <div class="space" />
                    <button class="edit">{"edit"}</button>
                </div>
                <div class="body">
                    <div class="children-display">
                        { for group.children.iter().cloned().map(|node| {
                            html! {<NodeDisplay node={node} />}
                        }) }
                    </div>
                    <div class="space" />
                    <div>{"TODO: Balance"}</div>
                </div>
                <div class="footer">
                </div>
            </div>
        }
    }
}
