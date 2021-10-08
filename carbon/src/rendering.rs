use std::cell::RefCell;
use std::rc::Rc;

use kurbo::{Affine, BezPath};
use piet::{Color, RenderContext, StrokeStyle};
use piet_web::WebRenderContext;

use crate::{Property, PropertyTreeContext, RenderTreeContext, StackFrame, Variable, Scope, PolymorphicType};
use std::collections::HashMap;

pub type RenderNodePtr = Rc<RefCell<dyn RenderNode >>;
pub type RenderNodePtrList = Rc<RefCell<Vec<RenderNodePtr>>>;

// Take a singleton node and wrap it into a Vec, e.g. to make a
// single node the entire `children` of another node
//TODO: handle this more elegantly, perhaps with some metaprogramming mojo?
pub fn wrap_render_node_ptr_into_list(rnp: RenderNodePtr) -> RenderNodePtrList {
    Rc::new(RefCell::new(vec![Rc::clone(&rnp)]))
}

pub struct RenderTree {
    pub root: RenderNodePtr //TODO:  maybe this should be more strictly a Rc<RefCell<Component>>, or a new type (alias) "ComponentPtr"
}

impl RenderTree {

}

/// `Runtime` is a container for data and logic needed by the `Engine`,
/// explicitly aside from rendering.  For example, logic for managing
/// scopes, stack frames, and properties should live here.
pub struct Runtime {
    stack: Vec<Rc<RefCell<StackFrame>>>,
    logger: fn(&str),
}
impl Runtime {
    pub fn new(logger: fn(&str)) -> Self {
        Runtime {
            stack: Vec::new(),
            logger,
        }
    }

    pub fn log(&self, message: &str) {
        (&self.logger)(message);
    }

    /// Return a pointer to the top StackFrame on the stack,
    /// without mutating the stack or consuming the value
    pub fn peek_stack_frame(&mut self) -> Option<Rc<RefCell<StackFrame>>> {
        if self.stack.len() > 0 {
            Some(Rc::clone(&self.stack[0]))
        }else{
            None
        }
    }

    /// Remove the top element from the stack.  Currently does
    /// nothing with the value of the popped StackFrame.
    pub fn pop_stack_frame(&mut self){
        self.stack.pop(); //TODO: handle value here if needed
    }

    /// Add a new frame to the stack, passing a list of adoptees
    /// that may be handled by `Placeholder` and a scope that includes
    pub fn push_stack_frame(&mut self, adoptees: RenderNodePtrList, scope: Scope) {


        //TODO:  for all children inside `adoptees`, check whether child `should_flatten`.
        //       If so, retrieve the `RenderNodePtrList` for its children and splice that list
        //       into a working full `RenderNodePtrList`.  This should be done recursively until
        //       there are no more descendents who are "contiguously flat".


        self.stack.push(
            Rc::new(RefCell::new(
                StackFrame::new(adoptees, Rc::new(RefCell::new(scope)))
            ))
        );
    }

}

//TODO:  do we need to refactor primitive properties (like Rectangle::width)
//       into the same `Property` structure as Components?
//          e.g. a `get_properties()` method
//       this would be imporant for addressing properties e.g. through
//       the property tree

/*
 Node {
    id: String
    properties: vec![
        (String.from("size"), PropertyLiteral {value: 500.0})
    ]
 }
 */
//
// TODO:
//  - Rename ScopeFrame to RepeatFrame
//  - make RepeatFrame<D(atum)> a component definition instead of a primitive
//     - this gives us a scope, stack frame, & data model for free
//       (just need to pass `i, datum` into the component declaration via Repeat's "template")
//  - What else do we need to do for <D> and <T> as they relate to Component?
//    (e.g. Component<D>?  What are the implications?)
//


pub trait RenderNode
{

    fn eval_properties_in_place(&mut self, ctx: &PropertyTreeContext);

    /// Lifecycle event: fires after evaluating a node's properties in place and its descendents properties
    /// in place.  Useful for cleaning up after a node (e.g. popping from the runtime stack) because
    /// this is the last time this node will be visited within the property tree for this frame.
    /// (Empty) default implementation because this is a rarely needed hook
    fn post_eval_properties_in_place(&mut self, ctx: &PropertyTreeContext) {}

    fn get_align(&self) -> (f64, f64) { (0.0,0.0) }
    fn get_children(&self, ) -> RenderNodePtrList;

    /// Returns the size of this node, or `None` if this node
    /// doesn't have a size (e.g. `Group`)
    fn get_size(&self) -> Option<(Size<f64>, Size<f64>)>;


    /// Rarely needed:  Used for exotic tree traversals, e.g. for `Spread` > `Repeat` > `Rectangle`
    /// where the repeated `Rectangle`s need to be be considered direct children of `Spread`.
    /// `Repeat` overrides `should_flatten` to return true, which `Engine` interprets to mean "ignore this
    /// node and consume its children" during traversal
    fn should_flatten(&self) -> bool {
        false
    }

    /// Returns the size of this node in pixels, requiring
    /// parent bounds for calculation of `Percent` values
    fn get_size_calc(&self, bounds: (f64, f64)) -> (f64, f64) {
        let size_raw = self.get_size();
        match size_raw {
            Some(size_raw) => {
                return (
                    match size_raw.0 {
                        Size::Pixel(width) => {
                            width
                        },
                        Size::Percent(width) => {
                            bounds.0 * (width / 100.0)
                        }
                    },
                    match size_raw.1 {
                        Size::Pixel(height) => {
                            height
                        },
                        Size::Percent(height) => {
                            bounds.1 * (height / 100.0)
                        }
                    }
                )
            },
            None => return bounds
        }
    }

    fn get_id(&self) -> &str;
    fn get_origin(&self) -> (Size<f64>, Size<f64>) { (Size::Pixel(0.0), Size::Pixel(0.0)) }
    fn get_transform(&self) -> &Affine;
    fn pre_render(&mut self, rtc: &mut RenderTreeContext, rc: &mut WebRenderContext);
    fn render(&self, rtc: &mut RenderTreeContext, rc: &mut WebRenderContext);
    fn post_render(&self, rtc: &mut RenderTreeContext, rc: &mut WebRenderContext);
}

pub struct Component<P: ?Sized> {
    pub template: Rc<RefCell<Vec<RenderNodePtr>>>,
    pub id: String,
    pub align: (f64, f64),
    pub origin: (Size<f64>, Size<f64>),
    pub transform: Affine,
    pub properties: P,
}

impl<T> RenderNode for Component<T> {
    fn eval_properties_in_place(&mut self, _: &PropertyTreeContext) {
        //TODO: handle each of Component's `Expressable` properties
        //  - this includes any custom properties (inputs) passed into this component
    }

    fn get_align(&self) -> (f64, f64) { self.align }
    fn get_children(&self) -> RenderNodePtrList {
        //Perhaps counter-intuitively, `Component`s return the root
        //of their template, rather than their `children`, for calls to get_children
        Rc::clone(&self.template)
    }
    fn get_size(&self) -> Option<(Size<f64>, Size<f64>)> { None }
    fn get_size_calc(&self, bounds: (f64, f64)) -> (f64, f64) { bounds }
    fn get_id(&self) -> &str {
        &self.id.as_str()
    }
    fn get_origin(&self) -> (Size<f64>, Size<f64>) { self.origin }
    fn get_transform(&self) -> &Affine {
        &self.transform
    }
    fn pre_render(&mut self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
    fn render(&self, _rtc: &mut RenderTreeContext, _rc: &mut WebRenderContext) {}
    fn post_render(&self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
}

pub struct Stroke {
    pub color: Color,
    pub width: f64,
    pub style: StrokeStyle,
}

#[derive(Copy, Clone)]
pub enum Size<T> {
    Pixel(T),
    Percent(T),
}

pub struct Rectangle {
    pub align: (f64, f64),
    pub size: (
        Box<dyn Property<Size<f64>>>,
        Box<dyn Property<Size<f64>>>,
    ),
    pub origin: (Size<f64>, Size<f64>),
    pub transform: Affine,
    pub stroke: Stroke,
    pub fill: Box<dyn Property<Color>>,
    pub id: String,
}


impl RenderNode for Rectangle {
    fn get_align(&self) -> (f64, f64) { self.align }
    fn get_children(&self) -> RenderNodePtrList {
        Rc::new(RefCell::new(vec![]))
    }
    fn eval_properties_in_place(&mut self, ctx: &PropertyTreeContext) {
        self.size.0.eval_in_place(ctx);
        self.size.1.eval_in_place(ctx);
        self.fill.eval_in_place(ctx);
    }
    fn get_origin(&self) -> (Size<f64>, Size<f64>) { self.origin }
    fn get_size(&self) -> Option<(Size<f64>, Size<f64>)> { Some((*self.size.0.read(), *self.size.1.read())) }
    fn get_size_calc(&self, bounds: (f64, f64)) -> (f64, f64) {
        let size_raw = self.get_size().unwrap();
        return (
            match size_raw.0 {
                Size::Pixel(width) => {
                    width
                },
                Size::Percent(width) => {
                    bounds.0 * (width / 100.0)
                }
            },
            match size_raw.1 {
                Size::Pixel(height) => {
                    height
                },
                Size::Percent(height) => {
                    bounds.1 * (height / 100.0)
                }
            }
        )
    }
    fn get_transform(&self) -> &Affine {
        &self.transform
    }
    fn get_id(&self) -> &str {
        &self.id.as_str()
    }
    fn pre_render(&mut self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
    fn render(&self, rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {

        let transform = rtc.transform;
        let bounding_dimens = rtc.bounding_dimens;
        let width: f64 =  bounding_dimens.0;
        let height: f64 =  bounding_dimens.1;

        let fill: &Color = &self.fill.read();

        let mut bez_path = BezPath::new();
        bez_path.move_to((0.0, 0.0));
        bez_path.line_to((width , 0.0));
        bez_path.line_to((width , height ));
        bez_path.line_to((0.0, height));
        bez_path.line_to((0.0,0.0));
        bez_path.close_path();

        let transformed_bez_path = *transform * bez_path;
        let duplicate_transformed_bez_path = transformed_bez_path.clone();
        // let mock_clipping_path = Affine::translate((width / 4.0, height / 4.0)) * transformed_bez_path.clone();

        // rc.clip(mock_clipping_path);
        // rc.save();
        rc.fill(transformed_bez_path, fill);
        rc.stroke(duplicate_transformed_bez_path, &self.stroke.color, self.stroke.width);
        // rc.restore();
    }
    fn post_render(&self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
}


pub struct If {

}

