use objc_id::ShareId;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};

use crate::foundation::{id, nil, YES, NO, NSArray, NSString};
use crate::color::Color;
use crate::layout::{Layout, LayoutAnchorX, LayoutAnchorY, LayoutAnchorDimension};
use crate::utils::properties::ObjcProperty;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
use macos::register_image_view_class;

#[cfg(target_os = "ios")]
mod ios;

#[cfg(target_os = "ios")]
use ios::register_image_view_class;

mod image;
pub use image::{Image, DrawConfig, ResizeBehavior};

mod icons;
pub use icons::*;

/// A helper method for instantiating view classes and applying default settings to them.
fn allocate_view(registration_fn: fn() -> *const Class) -> id { 
    unsafe {
        let view: id = msg_send![registration_fn(), new];
        let _: () = msg_send![view, setTranslatesAutoresizingMaskIntoConstraints:NO];

        #[cfg(target_os = "macos")]
        let _: () = msg_send![view, setWantsLayer:YES];

        view 
    }
}

/// A clone-able handler to a `ViewController` reference in the Objective C runtime. We use this
/// instead of a stock `View` for easier recordkeeping, since it'll need to hold the `View` on that
/// side anyway.
#[derive(Clone, Debug)]
pub struct ImageView {
    /// A pointer to the Objective-C runtime view controller.
    pub objc: ObjcProperty,
    
    /// A pointer to the Objective-C runtime top layout constraint.
    pub top: LayoutAnchorY,

    /// A pointer to the Objective-C runtime leading layout constraint.
    pub leading: LayoutAnchorX,

    /// A pointer to the Objective-C runtime left layout constraint.
    pub left: LayoutAnchorX,

    /// A pointer to the Objective-C runtime trailing layout constraint.
    pub trailing: LayoutAnchorX,

    /// A pointer to the Objective-C runtime right layout constraint.
    pub right: LayoutAnchorX,

    /// A pointer to the Objective-C runtime bottom layout constraint.
    pub bottom: LayoutAnchorY,

    /// A pointer to the Objective-C runtime width layout constraint.
    pub width: LayoutAnchorDimension,

    /// A pointer to the Objective-C runtime height layout constraint.
    pub height: LayoutAnchorDimension,

    /// A pointer to the Objective-C runtime center X layout constraint.
    pub center_x: LayoutAnchorX,

    /// A pointer to the Objective-C runtime center Y layout constraint.
    pub center_y: LayoutAnchorY
}

impl Default for ImageView {
    fn default() -> Self {
        ImageView::new()
    }
}

impl ImageView {
    /// Returns a default `View`, suitable for 
    pub fn new() -> Self {
        let view = allocate_view(register_image_view_class);

        ImageView {
            top: LayoutAnchorY::top(view),
            left: LayoutAnchorX::left(view),
            leading: LayoutAnchorX::leading(view),
            right: LayoutAnchorX::right(view),
            trailing: LayoutAnchorX::trailing(view),
            bottom: LayoutAnchorY::bottom(view),
            width: LayoutAnchorDimension::width(view),
            height: LayoutAnchorDimension::height(view),
            center_x: LayoutAnchorX::center(view),
            center_y: LayoutAnchorY::center(view),
            objc: ObjcProperty::retain(view),
        }
    }

    /// Call this to set the background color for the backing layer.
    pub fn set_background_color<C: AsRef<Color>>(&self, color: C) {
        self.objc.with_mut(|obj| unsafe {
            let cg = color.as_ref().cg_color();
            let layer: id = msg_send![obj, layer];
            let _: () = msg_send![layer, setBackgroundColor:cg];
        });
    }

    pub fn set_image(&self, image: &Image) {
        self.objc.with_mut(|obj| unsafe {
            let _: () = msg_send![obj, setImage:&*image.0];
        });
    }
}

impl Layout for ImageView {
    fn with_backing_node<F: Fn(id)>(&self, handler: F) {
        self.objc.with_mut(handler);
    }

    fn get_from_backing_node<F: Fn(&Object) -> R, R>(&self, handler: F) -> R {
        self.objc.get(handler)
    }
}

impl Drop for ImageView {
    /// A bit of extra cleanup for delegate callback pointers. If the originating `View` is being
    /// dropped, we do some logic to clean it all up (e.g, we go ahead and check to see if
    /// this has a superview (i.e, it's in the heirarchy) on the AppKit side. If it does, we go
    /// ahead and remove it - this is intended to match the semantics of how Rust handles things).
    ///
    /// There are, thankfully, no delegates we need to break here.
    fn drop(&mut self) {
        /*if self.delegate.is_some() {
            unsafe {
                let superview: id = msg_send![&*self.objc, superview];
                if superview != nil {
                    let _: () = msg_send![&*self.objc, removeFromSuperview];
                }
            }
        }*/
    }
}
