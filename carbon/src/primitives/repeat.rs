use std::cell::RefCell;
use std::rc::Rc;

use piet_web::WebRenderContext;

use crate::{Affine, PropertyTreeContext, RenderNode, RenderNodePtr, RenderNodePtrList, RenderTreeContext, Size, Scope, PolymorphicType, StackFrame, Component, wrap_render_node_ptr_into_list};
use std::collections::HashMap;

pub struct Repeat<D> {
    pub children: RenderNodePtrList,
    pub list: Vec<Rc<D>>,
    pub id: String,
    pub transform: Affine,
    virtual_children: RenderNodePtrList,
}

/// Data structure for the virtually duplicated container that surrounds repeated nodes.
/// This is attached to a Component<RepeatFrame> that `Repeat` adds to its children dynamically
/// during property-tree traversal
struct RepeatFrame<D> {
    pub i: usize,
    pub datum: Rc<D>,
    pub id: String,
}

impl<D> Repeat<D> {
    fn new(list: Vec<Rc<D>>, children: RenderNodePtrList, id: String, transform: Affine) -> Self {
        Repeat {
            list,
            children,
            id,
            transform,
            virtual_children:  Rc::new(RefCell::new(vec![])),

        }
    }
}

impl<D: 'static> RenderNode for Repeat<D> {
    fn eval_properties_in_place(&mut self, ptc: &PropertyTreeContext) {
        //TODO: handle each of Repeat's `Expressable` properties
        //

        // TODO:
        //  - add internal component store for virtual nodes, making this node their owner
        //  - add generics throughout Component to represent its data model
        //  - add a Component<RepeatFrame<D>> to the above store for each datum in array
        //  - return
        //



        //reset children
        self.children = Rc::new(RefCell::new(Vec::new()));


        //for each element in self.list, create a new child (Component) and push it to self.children
        for (i, datum) in self.list.iter().enumerate() {
            let properties = RepeatFrame { datum: Rc::clone(&datum), i, id: format!("repeat_frame_{}", i) };
            let adoptees = &ptc.runtime.borrow_mut().peek_stack_frame().unwrap().borrow().get_adoptees();

            self.children.borrow_mut().push(Rc::new(RefCell::new(Component {
                template: Rc::clone(adoptees),
                id: "".to_string(),
                align: (0.0, 0.0),
                origin: (Size::Pixel(0.0), Size::Pixel(0.0)),
                transform: Affine::default(),
                properties,
            })));
            // self.child_node_values.push(Rc::new(RefCell::new(Component {
            //     template: Rc::clone(adoptees),
            //     id: "".to_string(),
            //     align: (0.0, 0.0),
            //     origin: (Size::Pixel(0.0), Size::Pixel(0.0)),
            //     transform: Affine::default(),
            //     properties,
            // })));
            // self.children.borrow_mut().push(Rc::clone(&self.child_node_values[i]))
        }
        //
        // let y : Vec<_> = self.list.iter().enumerate().map(|(i, datum)|{
        //     //Construct the RepeatFrame that wraps each repeated datum
        //
        //     let adoptees = &ptc.runtime.borrow_mut().peek_stack_frame().unwrap().borrow().get_adoptees();
        //     Rc::new(RefCell::new(Component {
        //         template: Rc::clone(adoptees),
        //         id: "".to_string(),
        //         align: (0.0, 0.0),
        //         origin: (Size::Pixel(0.0), Size::Pixel(0.0)),
        //         transform: Affine::default(),
        //         properties: frame
        //     }))
        // }).collect();
        //
        // for i in y {
        //     self.children.borrow_mut().push(Rc::clone(&i));
        // }


         //TODO:  how do we set self.children = ^ the above?



        //
        // self.children = Rc::new(RefCell::new(
        //     self.list.iter().enumerate().map(|(i, datum)|{
        //
        //         // 1. construct a `puppeteer` node,
        //         //     - pass it the scope data (i, datum)
        //         // 2. Attach a copy of each child of this `repeat` node
        //         //     as a child of `puppeteer`
        //         // 3. write logic in `puppeteer` that delegates rendering to its contained nodes
        //         // 4. evaluate if we need to support any flattening fanciness around here
        //
        //
        //         let children_borrowed = self.children.borrow();
        //
        //
        //     }).collect()
        // ))
    }

    fn get_align(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
    fn should_flatten(&self) -> bool {
        true
    }
    fn get_children(&self) -> RenderNodePtrList {
        Rc::clone(&self.children)
    }
    fn get_size(&self) -> Option<(Size<f64>, Size<f64>)> { None }
    fn get_size_calc(&self, bounds: (f64, f64)) -> (f64, f64) { bounds }
    fn get_id(&self) -> &str {
        &self.id.as_str()
    }
    fn get_origin(&self) -> (Size<f64>, Size<f64>) {
        (Size::Pixel(0.0), Size::Pixel(0.0))
    }
    fn get_transform(&self) -> &Affine {
        &self.transform
    }
    fn pre_render(&mut self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
    fn render(&self, _rtc: &mut RenderTreeContext, _rc: &mut WebRenderContext) {}
    fn post_render(&self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
}

//
// impl RenderNode for RepeatFrame {
//     fn eval_properties_in_place(&mut self, ptc: &PropertyTreeContext) {
//         //TODO: handle each of ScopeFrame's `Expressable` properties
//
//
//         ptc.runtime.borrow_mut().push_stack_frame(
//             Rc::clone(&self.children),
//             //TODO:  cloning this data is a potentially heavy operation.  May be worth
//             //       revisiting design here, e.g. to pass a smart/pointer of `Scope` to the stack
//             self.scope.clone(),
//         );
//
//     }
//
//     fn get_align(&self) -> (f64, f64) {
//         (0.0, 0.0)
//     }
//     fn should_flatten(&self) -> bool {
//         true
//     }
//     fn get_children(&self) -> RenderNodePtrList {
//         Rc::clone(&self.children)
//     }
//     fn get_size(&self) -> Option<(Size<f64>, Size<f64>)> { None }
//     fn get_size_calc(&self, bounds: (f64, f64)) -> (f64, f64) { bounds }
//     fn get_id(&self) -> &str {
//         &self.id.as_str()
//     }
//     fn get_origin(&self) -> (Size<f64>, Size<f64>) {
//         (Size::Pixel(0.0), Size::Pixel(0.0))
//     }
//     fn get_transform(&self) -> &Affine {
//         &self.transform
//     }
//     fn pre_render(&mut self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
//     fn render(&self, _rtc: &mut RenderTreeContext, _rc: &mut WebRenderContext) {}
//     fn post_render(&self, _rtc: &mut RenderTreeContext, rc: &mut WebRenderContext) {}
// }


/*
lab journal, zb
---------------

To support polymorphic data <T> inside stack frames,
we need a `dyn SomeTrait` contract that stack frame data
can adhere to (instead of arbitrary `T`)

ex. `repeat` element stackframe data:
{
    index: usize,
    datum: T
}

We could have any stack frame abide by this contract:

StackFrameData<T> {
    get_index() -> usize;
    get_datum() -> Box<dyn T>;
}
...but how does the consumer know it's dealing with `T`?  Where does `T` come from?

Ultimately, it's userland.  E.g. consider the user-provided data:
cats = [{fur_color: Colors.WHITE, eye_color: Colors.BLUE}, {fur_color: Colors.BROWN, eye_color: Colors.GREEN}]
describes a schema and thus `T` of {fur_color: Color, eye_color: Color}

Perhaps this gets easier if we introduce our `scope` object here, and deal with a string:value (dynamic) lookup?

This turns our StackFrameData approach into:

StackFrame {
    get_scope() -> Scope;
}

along with

Scope {
    get_type(key: &str) -> PolymorphicType // used for unsafe unboxing of value
    get_value(key: &str) -> PolymorphicValue
}

When working with a Scope inside a `repeat`, the user looks up values & types by (string) key.

Seems like a suitable solution.

 */