use super::DebugLayer;
use crate::{
    audio::AudioEngine,
    input::{InputEngine, InputEngineInternal},
};
use crate::{
    rendering::{self, ui::ImguiContext, RenderingEngine},
    scene::SceneManager,
};
use alloc::{boxed::Box, rc::Rc};
use core::cell::RefCell;

pub struct CoreRadianceEngine {
    rendering_engine: Rc<RefCell<dyn RenderingEngine>>,
    audio_engine: Rc<dyn AudioEngine>,
    input_engine: Rc<RefCell<dyn InputEngineInternal>>,
    imgui_context: Rc<RefCell<ImguiContext>>,
    scene_manager: Option<Box<dyn SceneManager>>,
    debug_layer: Option<Box<dyn DebugLayer>>,
}

impl CoreRadianceEngine {
    pub(crate) fn new(
        rendering_engine: Rc<RefCell<dyn RenderingEngine>>,
        audio_engine: Rc<dyn AudioEngine>,
        input_engine: Rc<RefCell<dyn InputEngineInternal>>,
        imgui_context: Rc<RefCell<ImguiContext>>,
        scene_manager: Box<dyn SceneManager>,
    ) -> Self {
        Self {
            rendering_engine,
            audio_engine,
            input_engine,
            imgui_context,
            scene_manager: Some(scene_manager),
            debug_layer: None,
        }
    }

    pub fn rendering_component_factory(&self) -> Rc<dyn rendering::ComponentFactory> {
        self.rendering_engine.borrow().component_factory()
    }

    pub fn audio_engine(&mut self) -> Rc<dyn AudioEngine> {
        self.audio_engine.clone()
    }

    pub fn input_engine(&self) -> Rc<RefCell<dyn InputEngine>> {
        self.input_engine.borrow().as_input_engine()
    }

    pub fn set_debug_layer(&mut self, debug_layer: Box<dyn DebugLayer>) {
        self.debug_layer = Some(debug_layer);
    }

    pub fn scene_manager(&mut self) -> &mut dyn SceneManager {
        self.scene_manager.as_mut().unwrap().as_mut()
    }

    #[cfg(any(target_os = "windows", target_os = "psp"))]
    pub fn update(&mut self, delta_sec: f32) {
        self.input_engine.borrow_mut().update(delta_sec);

        let scene_manager = self.scene_manager.as_mut().unwrap();
        let debug_layer = self.debug_layer.as_mut();
        let ui_frame = self.imgui_context.borrow_mut().draw_ui(delta_sec, |ui| {
            scene_manager.update(ui, delta_sec);
            if let Some(dl) = debug_layer {
                dl.update(scene_manager.as_mut(), ui, delta_sec);
            }
        });

        let scene = self.scene_manager.as_mut().unwrap().scene_mut();
        if let Some(s) = scene {
            let mut rendering_engine = self.rendering_engine.as_ref().borrow_mut();
            let extent = rendering_engine.view_extent();
            s.camera_mut().set_aspect(extent.0 as f32 / extent.1 as f32);
            rendering_engine.render(s, ui_frame);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "android",))]
    pub fn update(&mut self, window: &winit::window::Window, delta_sec: f32) {
        self.input_engine.borrow_mut().update(delta_sec);

        let scene_manager = self.scene_manager.as_mut().unwrap();
        let debug_layer = self.debug_layer.as_mut();
        let ui_frame = self
            .imgui_context
            .borrow_mut()
            .draw_ui(window, delta_sec, |ui| {
                scene_manager.update(ui, delta_sec);
                if let Some(dl) = debug_layer {
                    dl.update(scene_manager.as_mut(), ui, delta_sec);
                }
            });

        let scene = self.scene_manager.as_mut().unwrap().scene_mut();
        if let Some(s) = scene {
            let extent = self.rendering_engine.as_ref().borrow().view_extent();
            s.camera_mut().set_aspect(extent.0 as f32 / extent.1 as f32);
            self.rendering_engine
                .as_ref()
                .borrow_mut()
                .render(s, ui_frame);
        }
    }
}

impl Drop for CoreRadianceEngine {
    fn drop(&mut self) {
        self.scene_manager = None;
    }
}
