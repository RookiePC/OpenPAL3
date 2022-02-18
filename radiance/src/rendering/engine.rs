use super::ComponentFactory;
use crate::{scene::Scene, rendering::ui::ImguiFrame};
use alloc::rc::Rc;

pub trait RenderingEngine {
    fn render(&mut self, scene: &mut dyn Scene, ui_frame: ImguiFrame);
    fn view_extent(&self) -> (u32, u32);
    fn component_factory(&self) -> Rc<dyn ComponentFactory>;
}
