//! Reference-counted references to items that are children of [`Node`].

use std::fmt;
use std::ops::Deref;

use crate::database::BuildingKindId;

use super::{
    Balance, Building, BuildingSettings, GeneratorSettings, GeothermalSettings, Group,
    ManufacturerSettings, MinerSettings, Node, NodeKind, PumpSettings,
};

/// A reference derived from Node which also owns the Node the reference is derived from,
/// to ensure the correct lifetime.
struct OwningRef<T> {
    /// Node the reference comes from.
    node: Node,
    /// The reference.
    child: *const T,
}

impl<T> OwningRef<T> {
    /// Construct an OwningRef from a Node and child reference. Unsafe because the caller
    /// is responsible for ensuring that the child is actually owned by the Node.
    unsafe fn new(node: Node, child: &T) -> Self {
        Self { node, child }
    }
}

impl<T> Deref for OwningRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // As long as the constructor followed the requirements, this is safe since the
        // node must still exist.
        unsafe { &*self.child }
    }
}

impl<T: PartialEq> PartialEq for OwningRef<T> {
    fn eq(&self, other: &Self) -> bool {
        // Opt: we know all the types we're using this on have structural equality, so we
        // can do pointer equality checks first.
        self.child == other.child || *self.deref() == *other.deref()
    }
}

impl<T: fmt::Debug> fmt::Debug for OwningRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<T: fmt::Display> fmt::Display for OwningRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}

impl<T> Clone for OwningRef<T> {
    fn clone(&self) -> Self {
        // This is safe because we know nodes are Rc types, so cloning the node ensures
        // the child lives as long as the newly returned value.
        Self {
            node: self.node.clone(),
            child: self.child,
        }
    }
}

macro_rules! ref_to {
    ($(
        $(#[$m:meta])*
        $Wrap:ident ($Inner:ident);
    )+) => {
        $(
            $(#[$m])*
            #[derive(Clone, Debug, PartialEq)]
            pub struct $Wrap(OwningRef<$Inner>);

            impl $Wrap {
                /// Construct this handle from a Node and child reference. Unsafe because
                /// the caller is responsible for ensuring that the child is actually
                /// owned by the Node.
                pub(crate) unsafe fn new(node: Node, child: &$Inner) -> Self {
                    Self(OwningRef::new(node, child))
                }

                /// Get a referece to the node this item is a child of.
                pub fn node(&self) -> &Node {
                    &self.0.node
                }

                /// Clone the wrapped referenced value.
                pub fn clone_inner(&self) -> $Inner {
                    self.deref().clone()
                }
            }

            impl Deref for $Wrap {
                type Target = $Inner;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }
        )+
    };
}

ref_to! {
    /// Reference counted reference to a [`Group`] that is part of a [`Node`].
    GroupRef(Group);
    /// Reference counted reference to a [`Balance`] that is part of a [`Node`].
    BalanceRef(Balance);
    /// Reference counted reference to a [`Building`] that is part of a [`Node`].
    BuildingRef(Building);
    /// Reference counted reference to a [`ManufacturerSettings`] that is part of a
    /// [`Node`].
    ManufacturerSettingsRef(ManufacturerSettings);
    /// Reference counted reference to a [`MinerSettings`] that is part of a [`Node`].
    MinerSettingsRef(MinerSettings);
    /// Reference counted reference to a [`GeneratorSettings`] that is part of a [`Node`].
    GeneratorSettingsRef(GeneratorSettings);
    /// Reference counted reference to a [`PumpSettings`] that is part of a [`Node`].
    PumpSettingsRef(PumpSettings);
    /// Reference counted reference to a [`GeothermalSettings`] that is part of a
    /// [`Node`].
    GeothermalSettingsRef(GeothermalSettings);
}

/// Reference counted version of [`NodeKind`].
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKindRef {
    Group(GroupRef),
    Building(BuildingRef),
}

impl NodeKindRef {
    pub(crate) fn new(node: &Node) -> Self {
        match node.kind() {
            NodeKind::Group(group) => Self::Group(unsafe { GroupRef::new(node.clone(), group) }),
            NodeKind::Building(building) => {
                Self::Building(unsafe { BuildingRef::new(node.clone(), building) })
            }
        }
    }

    /// Clones the contents of this ref
    pub fn clone_inner(self) -> NodeKind {
        match self {
            Self::Group(group) => group.clone_inner().into(),
            Self::Building(building) => building.clone_inner().into(),
        }
    }

    /// Gets the reference to the Group if this is a group.
    pub fn group(self) -> Option<GroupRef> {
        match self {
            Self::Group(group) => Some(group),
            _ => None,
        }
    }

    /// Gets the reference to the building if this is a building.
    pub fn building(self) -> Option<BuildingRef> {
        match self {
            Self::Building(building) => Some(building),
            _ => None,
        }
    }
}

impl BuildingRef {
    /// Get a reference-counted reference to the building settings for this building.
    pub fn settings_ref(&self) -> BuildingSettingsRef {
        match &self.deref().settings {
            BuildingSettings::Manufacturer(m) => BuildingSettingsRef::Manufacturer(unsafe {
                ManufacturerSettingsRef::new(self.0.node.clone(), m)
            }),
            BuildingSettings::Miner(m) => BuildingSettingsRef::Miner(unsafe {
                MinerSettingsRef::new(self.0.node.clone(), m)
            }),
            BuildingSettings::Generator(m) => BuildingSettingsRef::Generator(unsafe {
                GeneratorSettingsRef::new(self.0.node.clone(), m)
            }),
            BuildingSettings::Pump(m) => BuildingSettingsRef::Pump(unsafe {
                PumpSettingsRef::new(self.0.node.clone(), m)
            }),
            BuildingSettings::Geothermal(m) => BuildingSettingsRef::Geothermal(unsafe {
                GeothermalSettingsRef::new(self.0.node.clone(), m)
            }),
            BuildingSettings::PowerConsumer => BuildingSettingsRef::PowerConsumer,
        }
    }
}

/// Reference-counted version of [`BuildingSettings`].
#[derive(Debug, Clone, PartialEq)]
pub enum BuildingSettingsRef {
    Manufacturer(ManufacturerSettingsRef),
    Miner(MinerSettingsRef),
    Generator(GeneratorSettingsRef),
    Pump(PumpSettingsRef),
    Geothermal(GeothermalSettingsRef),
    PowerConsumer,
}

impl BuildingSettingsRef {
    /// Get the ID of this buiilding kind.
    pub fn kind_id(&self) -> BuildingKindId {
        match self {
            Self::Manufacturer(_) => BuildingKindId::Manufacturer,
            Self::Miner(_) => BuildingKindId::Miner,
            Self::Generator(_) => BuildingKindId::Generator,
            Self::Pump(_) => BuildingKindId::Pump,
            Self::Geothermal(_) => BuildingKindId::Geothermal,
            Self::PowerConsumer => BuildingKindId::PowerConsumer,
        }
    }
}
