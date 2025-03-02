// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

#![warn(missing_docs)]

//! This module contains the basic datastructures that are exposed to the C API

use crate::accessibility::AccessibleStringProperty;
use crate::item_tree::{
    ItemTreeNode, ItemVisitorVTable, ItemWeak, TraversalOrder, VisitChildrenResult,
};
use crate::items::{AccessibleRole, ItemVTable};
use crate::layout::{LayoutInfo, Orientation};
use crate::slice::Slice;
use crate::window::WindowRc;
use crate::SharedString;
use vtable::*;

#[repr(C)]
/// A range of indices
pub struct IndexRange {
    /// Start index
    pub start: usize,
    /// Index one past the last index
    pub end: usize,
}

/// A Component is representing an unit that is allocated together
#[vtable]
#[repr(C)]
pub struct ComponentVTable {
    /// Visit the children of the item at index `index`.
    /// Note that the root item is at index 0, so passing 0 would visit the item under root (the children of root).
    /// If you want to visit the root item, you need to pass -1 as an index.
    pub visit_children_item: extern "C" fn(
        core::pin::Pin<VRef<ComponentVTable>>,
        index: isize,
        order: TraversalOrder,
        visitor: VRefMut<ItemVisitorVTable>,
    ) -> VisitChildrenResult,

    /// Return a reference to an item using the given index
    pub get_item_ref: extern "C" fn(
        core::pin::Pin<VRef<ComponentVTable>>,
        index: usize,
    ) -> core::pin::Pin<VRef<ItemVTable>>,

    /// Return the range of indices below the dynamic `ItemTreeNode` at `index`
    pub get_subtree_range:
        extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>, index: usize) -> IndexRange,

    /// Return the `ComponentRc` at `subindex` below the dynamic `ItemTreeNode` at `index`
    pub get_subtree_component: extern "C" fn(
        core::pin::Pin<VRef<ComponentVTable>>,
        index: usize,
        subindex: usize,
        result: &mut vtable::VWeak<ComponentVTable, Dyn>,
    ),

    /// Return the item tree that is defined by this `Component`.
    /// The return value is an item weak because it can be null if there is no parent.
    /// And the return value is passed by &mut because ItemWeak has a destructor
    pub get_item_tree: extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>) -> Slice<ItemTreeNode>,

    /// Return the node this component is a part of in the parent component.
    ///
    /// The return value is an item weak because it can be null if there is no parent.
    /// And the return value is passed by &mut because ItemWeak has a destructor
    /// Note that the returned value will typically point to a repeater node, which is
    /// strictly speaking not an Item at all!
    pub parent_node: extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>, result: &mut ItemWeak),

    /// Return the index of the current subtree or usize::MAX if this is not a subtree
    pub subtree_index: extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>) -> usize,

    /// Returns the layout info for this component
    pub layout_info:
        extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>, Orientation) -> LayoutInfo,

    /// Returns the accessible role for a given item
    pub accessible_role:
        extern "C" fn(core::pin::Pin<VRef<ComponentVTable>>, item_index: usize) -> AccessibleRole,

    /// Returns the accessible property
    pub accessible_string_property: extern "C" fn(
        core::pin::Pin<VRef<ComponentVTable>>,
        item_index: usize,
        what: AccessibleStringProperty,
        result: &mut SharedString,
    ),

    /// in-place destructor (for VRc)
    pub drop_in_place: unsafe fn(VRefMut<ComponentVTable>) -> vtable::Layout,
    /// dealloc function (for VRc)
    pub dealloc: unsafe fn(&ComponentVTable, ptr: *mut u8, layout: vtable::Layout),
}

#[cfg(test)]
pub(crate) use ComponentVTable_static;

/// Alias for `vtable::VRef<ComponentVTable>` which represent a pointer to a `dyn Component` with
/// the associated vtable
pub type ComponentRef<'a> = vtable::VRef<'a, ComponentVTable>;

/// Type alias to the commonly used `Pin<VRef<ComponentVTable>>>`
pub type ComponentRefPin<'a> = core::pin::Pin<ComponentRef<'a>>;

/// Type alias to the commonly used VRc<ComponentVTable, Dyn>>
pub type ComponentRc = vtable::VRc<ComponentVTable, Dyn>;
/// Type alias to the commonly used VWeak<ComponentVTable, Dyn>>
pub type ComponentWeak = vtable::VWeak<ComponentVTable, Dyn>;

/// Call init() on the ItemVTable for each item of the component.
pub fn init_component_items<Base>(
    base: core::pin::Pin<&Base>,
    item_array: &[vtable::VOffset<Base, ItemVTable, vtable::AllowPin>],
    window: &WindowRc,
) {
    item_array.iter().for_each(|item| item.apply_pin(base).as_ref().init(window));
}

/// Free the backend graphics resources allocated by the component's items.
pub fn free_component_item_graphics_resources<Base>(
    base: core::pin::Pin<&Base>,
    component: ComponentRef,
    item_array: &[vtable::VOffset<Base, ItemVTable, vtable::AllowPin>],
    window: &WindowRc,
) {
    window.free_graphics_resources(
        component,
        &mut item_array.iter().map(|item| item.apply_pin(base)),
    );
}

#[cfg(feature = "ffi")]
pub(crate) mod ffi {
    #![allow(unsafe_code)]

    use super::*;

    /// Call init() on the ItemVTable of each item in the item array.
    #[no_mangle]
    pub unsafe extern "C" fn slint_component_init_items(
        component: ComponentRefPin,
        item_array: Slice<vtable::VOffset<u8, ItemVTable, vtable::AllowPin>>,
        window_handle: *const crate::window::ffi::WindowRcOpaque,
    ) {
        let window = &*(window_handle as *const WindowRc);
        super::init_component_items(
            core::pin::Pin::new_unchecked(&*(component.as_ptr() as *const u8)),
            item_array.as_slice(),
            window,
        )
    }

    /// Free the backend graphics resources allocated in the item array.
    #[no_mangle]
    pub unsafe extern "C" fn slint_component_free_item_array_graphics_resources(
        component: ComponentRefPin,
        item_array: Slice<vtable::VOffset<u8, ItemVTable, vtable::AllowPin>>,
        window_handle: *const crate::window::ffi::WindowRcOpaque,
    ) {
        let window = &*(window_handle as *const WindowRc);
        super::free_component_item_graphics_resources(
            core::pin::Pin::new_unchecked(&*(component.as_ptr() as *const u8)),
            core::pin::Pin::into_inner(component),
            item_array.as_slice(),
            window,
        )
    }
}
