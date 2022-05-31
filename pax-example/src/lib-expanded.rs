// #[macro_use]
// extern crate pax_compiler_api;
//
// use pax_compiler_api::lazy_static;
//
//
// use pax::*;
// use pax::api::{ArgsCoproduct, ArgsRender, Property, ArgsClick, EasingCurve};
//
// use pax_std::primitives::{Group, Rectangle};
//
// //Re-exports of all used types, for use by cartridge
// pub mod pax_types {
//     pub mod pax_std {
//         pub mod primitives {
//             pub use pax_std::primitives::Rectangle;
//             pub use pax_std::primitives::Group;
//             pub use pax_std::primitives::Text;
//         }
//         pub mod components {
//             pub use pax_std::components::Stacker;
//         }
//         pub mod types {
//             pub use pax_std::types::StackerCellProperties;
//             pub use pax_std::types::Color;
//             pub use pax_std::types::Font;
//             pub use pax_std::types::Stroke;
//             pub use pax_std::types::Size;
//             pub use pax_std::types::StackerDirection;
//         }
//     }
//     pub use pax::api::Transform2D;
//     pub use pax::api::SizePixels;
//
//     pub use crate::Root;
// }
//
// #[derive(Default)]
// pub struct Root {
//     pub num_clicks: Property<isize>,
//     pub current_rotation: Property<f64>,
// }
//
// impl Root {
//     pub fn handle_pre_render(&mut self, args: ArgsRender) {
//         if args.frames_elapsed % 180 == 0 {
//             //every 3s
//             pax::log(&format!("pax::log from frame {}", args.frames_elapsed));
//         }
//     }
//
//     pub fn handle_click(&mut self, args: ArgsClick) {
//         let new_rotation = self.current_rotation.get() + (2.0 * std::f64::consts::PI);
//         self.current_rotation.ease_to(new_rotation, 120, EasingCurve::InOutBack );
//         self.current_rotation.ease_to_later(0.0, 40, EasingCurve::OutBack );
//     }
// }
//
// #[cfg(feature = "parser")]
// use pax::internal::message::ComponentDefinition;
// #[cfg(feature = "parser")]
// use pax_compiler_api::ManifestContext;
// #[cfg(feature = "parser")]
// use std::collections::HashMap;
// #[cfg(feature = "parser")]
// use std::collections::HashSet;
// #[cfg(feature = "parser")]
// use std::path::{Path, PathBuf};
// #[cfg(feature = "parser")]
// use std::{env, fs};
// #[cfg(feature = "parser")]
// use pax::internal::message::{SettingsValueDefinition, PaxManifest ,SettingsLiteralBlockDefinition};
// #[cfg(feature = "parser")]
// lazy_static! {
//     static ref source_id: String = pax_compiler_api::create_uuid();
// }
// #[cfg(feature = "parser")]
// pub fn main() {
//     let mut ctx = ManifestContext {
//         root_component_id: "".into(),
//         visited_source_ids: HashSet::new(),
//         component_definitions: vec![],
//         template_map: HashMap::new(),
//     };
//
//     let (ctx, _) = Root::parse_to_manifest(ctx);
//
//     //TODO: should be able to de-dupe PaxManifest and ManifestContext data structures
//     let manifest = PaxManifest {
//         components: ctx.component_definitions,
//         root_component_id: ctx.root_component_id,
//     };
//
//     println!("serialized bytes: {:?}", manifest.serialize());
//
// }
//
//
// #[cfg(feature = "parser")]
// //GENERATE pascal_identifier
// impl Root {
//     pub fn parse_to_manifest(mut ctx: ManifestContext) -> (ManifestContext, String) {
//         //a given source may be EITHER a.) inline pax, or b.) code-behind pax.
//         //this is decided based on which macro is used: [#pax(contents)] for inline and [#pax_file("path")] for file
//         //those two macros should be otherwise equivalent, generating simply a different line that
//         //evaluates `raw_pax`.
//
//         const raw_pax: &str = r#"<Stacker cell_count=10 >
//     <Stacker cell_count=5 direction=Vertical >
//         for i in 0..5 {
//             <Rectangle fill={Color::Rgba(20%, 0, 100%, 100%)} />
//         }
//     </Stacker>
//
//     for i in 0..8 {
//         <Group>
//             <Text id=index_text>"Index: {i}"</Text>
//             <Rectangle fill={Color::Rgba(100%, 100% - (i * 12.5%), (i * 12.5)%, 100%)} />
//         </Group>
//     }
//
//     <Group @click=self.handle_click transform={Rotate(self.current_rotation)}>
//         <Text>{JABBERWOCKY}</Text>
//         <Rectangle fill=Color::Rgba(100%, 100%, 0, 100%) />
//     </Group>
// </Stacker>
//
// @settings {
//     #index_text {
//         transform: { Align(0%, i * 12.5%) }
//         font: {
//             family: "Real Text Pro",
//             variant: "Demibold",
//             size: {(20 + (i * 5))px},
//         }
//         fill: Color::Rgb(100%, 100%, 100%)
//     }
// }
// "#;
//
// //
// // const raw_pax: &str = r#"<Group><Rectangle /></Group>
// // "#;
//
//         match ctx.visited_source_ids.get(&source_id as &str) {
//
//             None => {
//                 //First time visiting this file/source — parse the relevant contents
//                 //then recurse through child nodes, unrolled here in the macro as
//                 //parsed from the template
//                 ctx.visited_source_ids.insert(source_id.clone());
//
//
//                 //GENERATE: do for each unique component type found in template
//                 let (mut ctx, component_id) = Rectangle::parse_to_manifest(ctx);
//                 ctx.template_map.insert("Rectangle".into(), component_id);
//                 let (mut ctx, component_id) = Group::parse_to_manifest(ctx);
//                 ctx.template_map.insert("Group".into(), component_id);
//
//                 //GENERATE: inject pascal_identifier instead of CONSTANT
//                 let PASCAL_IDENTIFIER = "Root";
//
//                 let template_map= ctx.template_map.clone();
//
//                 let (mut ctx, comp_def) =
//                     pax_compiler_api::parse_full_component_definition_string(
//                         ctx,
//                         &raw_pax,
//                         PASCAL_IDENTIFIER,
//                         true,
//                         template_map,
//                         &source_id as &str,
//                         module_path!(),
//                     );
//
//                 println!("{:?}", &comp_def);
//
//                 ctx.component_definitions
//                     .push(comp_def);
//
//                 (ctx, source_id.to_string())
//             },
//             _ => (ctx, source_id.to_string()), //early return; this file has already been parsed
//         }
//     }
// }
// //
// // impl RootProperties {
// //
// //     //ideally, this would accept &mut self
// //     pub fn handle_tick(&mut self, evt: pax::api::ArgsTick) {
// //
// //         &self.num_clicks.set(*self.num_clicks.get() + 1);
// //         // let mut num_clicks = (*props).num_clicks;
// //         // num_clicks.set(num_clicks.get() + 1);
// //
// //     }
// //     // pub fn handle_tick(&mut props: RootProperties , evt: pax::api::EventTick) {
// //     //     let mut num_clicks = (*props).num_clicks;
// //     //     num_clicks.set(num_clicks.get() + 1)
// //     // }
// // }
// //
