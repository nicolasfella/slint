// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

//! Pass that check that the public api is ok and mark the property as exposed

use std::rc::Rc;

use crate::diagnostics::{BuildDiagnostics, DiagnosticLevel};
use crate::langtype::Type;
use crate::object_tree::{Component, Document};

pub fn check_public_api(doc: &Document, diag: &mut BuildDiagnostics) {
    check_public_api_component(&doc.root_component, diag);
    for (export_name, ty) in doc.exports() {
        if let Type::Component(c) = ty {
            if c.is_global() {
                // This global will become part of the public API.
                c.exported_global_names.borrow_mut().push(export_name.clone());
                check_public_api_component(c, diag)
            }
        }
    }
}

fn check_public_api_component(root_component: &Rc<Component>, diag: &mut BuildDiagnostics) {
    let mut root_elem = root_component.root_element.borrow_mut();
    let root_elem = &mut *root_elem;
    let mut pa = root_elem.property_analysis.borrow_mut();
    root_elem.property_declarations.iter_mut().for_each(|(n, d)| {
        if d.property_type.ok_for_public_api() {
            d.expose_in_public_api = true;
            pa.entry(n.to_string()).or_default().is_set = true;
        } else {
            diag.push_diagnostic(
                 format!("Properties of type {} are not supported yet for public API. The property will not be exposed", d.property_type),
                 &d.type_node(),
                 DiagnosticLevel::Warning
            );
        }
    });
}
