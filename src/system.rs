use std::{
    collections::HashMap,
    time::Duration,
    fmt,
};

use amethyst_assets::Handle;
use amethyst_core::{
    ecs::{
        ParallelRunnable,
        System,
        SystemBuilder,
    },
    shrev::ReaderId,
    EventChannel,
    Time,
};
use amethyst_input::{
    ScrollDirection,
    VirtualKeyCode,
};
use amethyst_rendy::Texture;
use amethyst_window::{
    ScreenDimensions,
    Window,
};
use derivative::Derivative;
use egui::{
    CtxRef,
    CursorIcon,
    Event,
    Key,
    Modifiers,
    Output,
    PointerButton,
    Pos2,
    RawInput,
    Rect,
    Vec2,
};
use winit::{
    dpi::PhysicalPosition,
    event::{
        ElementState,
        Event as WEvent,
        KeyboardInput,
        ModifiersState,
        MouseButton,
        MouseScrollDelta,
        WindowEvent,
    },
    window::CursorIcon as WCursorIcon,
};

#[derive(Debug, Default)]
pub struct EguiInputGrab {
    pub keyboard: bool,
    pub mouse: bool,
}

pub(crate) enum EguiStage {
    Begin,
    Render,
    End(Output),
}

impl fmt::Debug for EguiStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EguiStage::Begin => write!(f, "EguiStage::Begin"),
            EguiStage::Render => write!(f, "EguiStage::Render"),
            EguiStage::End(_) => write!(f, "EguiStage::End(_)"),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct EguiContext {
    #[derivative(Debug = "ignore")]
    pub(crate) ctx: CtxRef,
    pub(crate) stage: EguiStage,
    #[derivative(Debug = "ignore")]
    pub(crate) user_textures: HashMap<u64, Handle<Texture>>,
}


impl Default for EguiContext {
    fn default() -> Self {
        Self {
            ctx: CtxRef::default(),
            stage: EguiStage::Begin,
            user_textures: HashMap::new(),
        }
    }
}

impl EguiContext {
    pub fn ctx(&self) -> Option<&CtxRef> {
        if matches!(self.stage, EguiStage::Render) {
            Some(&self.ctx)
        }
        else {
            None
        }
    }
}

#[derive(Clone, Derivative)]
#[derivative(Default)]
pub struct EguiConfig {
    #[derivative(Default(value = "1.0"))]
    pub scroll_sensitivity: f32,

    pub mirror_mouse_buttons: bool,

    #[cfg(feature = "webbrowser")]
    pub allow_webbrowser: bool,

    #[cfg(feature = "clipboard")]
    pub allow_clipboard: bool,

    #[cfg(feature = "tts")]
    pub enable_tts: bool,
}

#[derive(Derivative)]
pub struct EguiSystem {
    winit_event_reader: ReaderId<WEvent<'static, ()>>,
    current_mouse_pos: Pos2,
    current_modifiers: Modifiers,
}

impl EguiSystem {
    pub fn new(winit_event_reader: ReaderId<WEvent<'_, ()>>) -> Self {
        Self {
            winit_event_reader,
            current_mouse_pos: Pos2::default(),
            current_modifiers: Modifiers::default(),
        }
    }

    fn handle_window_events(
        &mut self,
        egui_input: &mut RawInput,
        window_events: &EventChannel<WEvent<'static, ()>>,
        config: &EguiConfig,
    ) {
        for event in window_events.read(&mut self.winit_event_reader) {
            match event {
                WEvent::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            egui_input.key(input);
                        }
                        WindowEvent::ModifiersChanged(modifiers) => {
                            egui_input.modifiers(modifiers, &mut self.current_modifiers);
                            self.current_modifiers = egui_input.modifiers;
                        }
                        WindowEvent::ReceivedCharacter(chr) => {
                            egui_input.key_char(*chr);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            egui_input.mouse_moved(position, &mut self.current_mouse_pos);
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            egui_input.mouse_button(
                                *state,
                                *button,
                                &self.current_mouse_pos,
                                config.mirror_mouse_buttons,
                            );
                        }
                        WindowEvent::CursorLeft { .. } => {
                            egui_input.mouse_left();
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            egui_input.mouse_wheel(delta);
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            //todo!()
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(unused_variables)]
    fn handle_output(&mut self, egui_output: Output, window: &Window, config: &EguiConfig) {
        set_cursor(window, egui_output.cursor_icon);

        #[cfg(feature = "webbrowser")]
        if config.allow_webbrowser {
            use egui::output::OpenUrl;
            if let Some(OpenUrl { url, .. }) = egui_output.open_url {
                if let Err(e) = webbrowser::open(&url) {
                    log::error!("{}", e);
                }
            }
        }

        #[cfg(feature = "clipboard")]
        if config.allow_clipboard {
            if !egui_output.copied_text.is_empty() {
                todo!()
            }
        }

        // TODO: handle `needs_repaint`?

        #[cfg(feature = "tts")]
        if config.enable_tts {
            use egui::output::OutputEvent;
            for event in egui_output.events {
                match event {
                    OutputEvent::Clicked(i)
                    | OutputEvent::DoubleClicked(i)
                    | OutputEvent::FocusGained(i)
                    | OutputEvent::TextSelectionChanged(i)
                    | OutputEvent::ValueChanged(i) => {
                        let _text = i.description();
                        todo!()
                    }
                }
            }
        }
    }
}

impl System for EguiSystem {
    fn build(mut self) -> Box<dyn ParallelRunnable + 'static> {
        Box::new(
            SystemBuilder::new("EguiSystem")
                .read_resource::<EguiConfig>()
                .write_resource::<EguiContext>()
                .read_resource::<ScreenDimensions>()
                .read_resource::<EventChannel<WEvent<'_, ()>>>()
                .read_resource::<Time>()
                .read_resource::<Window>()
                .write_resource::<EguiInputGrab>()
                //.with_query(<(Read<Transform>, Read<ChunkLoadTag>, TryRead<Camera>)>::query())
                .build(move |_commands, _world, resources, _queries| {
                    // # TODO
                    //
                    // - Fill field `pixels_per_point`. See [1]
                    // - Should we use `FpsCounter` for `predicted_dt`?
                    // - Allow events to be sent to this system? E.g. Copy/Cut?
                    //
                    // [1] https://docs.rs/winit/0.25.0/winit/dpi/index.html

                    let (
                        config,
                        egui_ctx,
                        screen_dimensions,
                        winit_events,
                        time,
                        window,
                        input_grab,
                    ) = resources;

                    if matches!(&egui_ctx.stage, EguiStage::Render) {
                        log::error!("EguiSystem running with invalid EguiState");
                        return;
                    }

                    // If the render pass left us some output, we first handle it.
                    if let EguiStage::End(egui_output) =
                        std::mem::replace(&mut egui_ctx.stage, EguiStage::Render)
                    {
                        self.handle_output(egui_output, window, config);
                    }

                    // Set `EguiInputGrab` resource for other systems to know, whether Egui is using
                    // the input.
                    input_grab.keyboard = egui_ctx.ctx.wants_keyboard_input();
                    input_grab.mouse = egui_ctx.ctx.wants_pointer_input();
                    //log::debug!("{:?}", input_grab);

                    // Gather inputs

                    let screen_rect = Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(screen_dimensions.width(), screen_dimensions.height()),
                    );

                    let mut egui_input = RawInput {
                        screen_rect: Some(screen_rect),
                        modifiers: self.current_modifiers,
                        time: Some(duration_as_secs_with_nanos_f64(time.absolute_real_time())),
                        predicted_dt: duration_as_secs_with_nanos_f32(time.delta_real_time()),
                        ..RawInput::default()
                    };

                    self.handle_window_events(&mut egui_input, &winit_events, &config);

                    // Render UI
                    egui_ctx.ctx.begin_frame(egui_input);
                    egui_ctx.stage = EguiStage::Render;
                }),
        )
    }
}

fn duration_as_secs_with_nanos_f64(duration: Duration) -> f64 {
    duration.as_secs_f64() + duration.subsec_nanos() as f64 / 1_000_000_000.0
}

fn duration_as_secs_with_nanos_f32(duration: Duration) -> f32 {
    duration.as_secs_f32() + duration.subsec_nanos() as f32 / 1_000_000_000.0
}

trait EguiInput {
    fn key(&mut self, keyboard_input: &KeyboardInput);
    fn modifiers(&mut self, modifiers: &ModifiersState, current_modifiers: &mut Modifiers);
    fn key_char(&mut self, c: char);
    fn mouse_moved(&mut self, position: &PhysicalPosition<f64>, current_mouse_pos: &mut Pos2);
    fn mouse_button(
        &mut self,
        state: ElementState,
        button: MouseButton,
        current_mouse_pos: &Pos2,
        mirror_buttons: bool,
    );
    fn mouse_wheel(&mut self, delta: &MouseScrollDelta);
    fn mouse_left(&mut self);

    //fn set_modifier(&mut self, key: VirtualKeyCode, value: bool);
    fn add_scroll_delta(&mut self, direction: ScrollDirection, sensitivty: f32);
}

impl EguiInput for RawInput {
    fn key(&mut self, keyboard_input: &KeyboardInput) {
        if let Some(key) = keyboard_input
            .virtual_keycode
            .as_ref()
            .and_then(convert_key)
        {
            self.events.push(Event::Key {
                key,
                pressed: matches!(keyboard_input.state, ElementState::Pressed),
                modifiers: self.modifiers,
            });
        }
    }

    fn modifiers(&mut self, modifiers: &ModifiersState, current_modifiers: &mut Modifiers) {
        let modifiers = convert_modifiers(modifiers);
        self.modifiers = modifiers;
        *current_modifiers = self.modifiers;
    }

    fn key_char(&mut self, c: char) {
        if c != '\n' && c != '\r' {
            self.events.push(Event::Text(c.to_string()))
        }
    }

    fn mouse_moved(&mut self, position: &PhysicalPosition<f64>, current_mouse_pos: &mut Pos2) {
        let pos = Pos2::new(position.x as f32, position.y as f32);
        self.events.push(Event::PointerMoved(pos));
        *current_mouse_pos = pos;
    }

    fn mouse_button(
        &mut self,
        state: ElementState,
        button: MouseButton,
        current_mouse_pos: &Pos2,
        mirror_buttons: bool,
    ) {
        if let Some(button) = convert_mouse_button(button, mirror_buttons) {
            self.events.push(Event::PointerButton {
                pos: *current_mouse_pos,
                button,
                pressed: matches!(state, ElementState::Pressed),
                modifiers: self.modifiers,
            });
        }
    }

    fn mouse_wheel(&mut self, delta: &MouseScrollDelta) {
        if let MouseScrollDelta::PixelDelta(px) = delta {
            self.scroll_delta += Vec2::new(px.x as f32, px.y as f32);
        }
    }

    fn mouse_left(&mut self) {
        self.events.push(Event::PointerGone);
    }

    fn add_scroll_delta(&mut self, direction: ScrollDirection, sensitivty: f32) {
        match direction {
            ScrollDirection::ScrollUp => self.scroll_delta.y -= sensitivty,
            ScrollDirection::ScrollDown => self.scroll_delta.y += sensitivty,
            ScrollDirection::ScrollLeft => self.scroll_delta.x -= sensitivty,
            ScrollDirection::ScrollRight => self.scroll_delta.x += sensitivty,
        }
    }
}

fn convert_key(key: &VirtualKeyCode) -> Option<Key> {
    match key {
        VirtualKeyCode::Down => Some(Key::ArrowDown),
        VirtualKeyCode::Left => Some(Key::ArrowLeft),
        VirtualKeyCode::Right => Some(Key::ArrowRight),
        VirtualKeyCode::Up => Some(Key::ArrowUp),
        VirtualKeyCode::Escape => Some(Key::Escape),
        VirtualKeyCode::Tab => Some(Key::Tab),
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Return => Some(Key::Enter),
        VirtualKeyCode::Space => Some(Key::Space),
        VirtualKeyCode::Insert => Some(Key::Insert),
        VirtualKeyCode::Delete => Some(Key::Delete),
        VirtualKeyCode::Home => Some(Key::Home),
        VirtualKeyCode::End => Some(Key::End),
        VirtualKeyCode::PageUp => Some(Key::PageUp),
        VirtualKeyCode::PageDown => Some(Key::PageDown),
        VirtualKeyCode::Key0 => Some(Key::Num0),
        VirtualKeyCode::Key1 => Some(Key::Num1),
        VirtualKeyCode::Key2 => Some(Key::Num2),
        VirtualKeyCode::Key3 => Some(Key::Num3),
        VirtualKeyCode::Key4 => Some(Key::Num4),
        VirtualKeyCode::Key5 => Some(Key::Num5),
        VirtualKeyCode::Key6 => Some(Key::Num6),
        VirtualKeyCode::Key7 => Some(Key::Num7),
        VirtualKeyCode::Key8 => Some(Key::Num8),
        VirtualKeyCode::Key9 => Some(Key::Num9),
        VirtualKeyCode::A => Some(Key::A),
        VirtualKeyCode::B => Some(Key::B),
        VirtualKeyCode::C => Some(Key::C),
        VirtualKeyCode::D => Some(Key::D),
        VirtualKeyCode::E => Some(Key::E),
        VirtualKeyCode::F => Some(Key::F),
        VirtualKeyCode::G => Some(Key::G),
        VirtualKeyCode::H => Some(Key::H),
        VirtualKeyCode::I => Some(Key::I),
        VirtualKeyCode::J => Some(Key::J),
        VirtualKeyCode::K => Some(Key::K),
        VirtualKeyCode::L => Some(Key::L),
        VirtualKeyCode::M => Some(Key::M),
        VirtualKeyCode::N => Some(Key::N),
        VirtualKeyCode::O => Some(Key::O),
        VirtualKeyCode::P => Some(Key::P),
        VirtualKeyCode::Q => Some(Key::Q),
        VirtualKeyCode::R => Some(Key::R),
        VirtualKeyCode::S => Some(Key::S),
        VirtualKeyCode::T => Some(Key::T),
        VirtualKeyCode::U => Some(Key::U),
        VirtualKeyCode::V => Some(Key::V),
        VirtualKeyCode::W => Some(Key::W),
        VirtualKeyCode::X => Some(Key::X),
        VirtualKeyCode::Y => Some(Key::Y),
        VirtualKeyCode::Z => Some(Key::Z),
        _ => None,
    }
}

fn convert_modifiers(modifiers: &ModifiersState) -> Modifiers {
    Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        mac_cmd: modifiers.logo(),
        command: modifiers.logo(),
    }
}

fn convert_mouse_button(mouse_button: MouseButton, mirror_buttons: bool) -> Option<PointerButton> {
    match mouse_button {
        MouseButton::Left if mirror_buttons => Some(PointerButton::Secondary),
        MouseButton::Right if mirror_buttons => Some(PointerButton::Primary),
        MouseButton::Left if !mirror_buttons => Some(PointerButton::Primary),
        MouseButton::Right if !mirror_buttons => Some(PointerButton::Secondary),
        MouseButton::Middle => Some(PointerButton::Middle),
        _ => None,
    }
}

fn set_cursor(window: &Window, cursor_icon: CursorIcon) {
    let icon = match cursor_icon {
        CursorIcon::Default => WCursorIcon::Default,
        CursorIcon::None => {
            window.set_cursor_visible(false);
            return;
        }
        CursorIcon::ContextMenu => WCursorIcon::ContextMenu,
        CursorIcon::Help => WCursorIcon::Help,
        CursorIcon::PointingHand => WCursorIcon::Hand,
        CursorIcon::Progress => WCursorIcon::Progress,
        CursorIcon::Wait => WCursorIcon::Wait,
        CursorIcon::Cell => WCursorIcon::Cell,
        CursorIcon::Crosshair => WCursorIcon::Crosshair,
        CursorIcon::Text => WCursorIcon::Text,
        CursorIcon::VerticalText => WCursorIcon::VerticalText,
        CursorIcon::Alias => WCursorIcon::Alias,
        CursorIcon::Copy => WCursorIcon::Copy,
        CursorIcon::Move => WCursorIcon::Move,
        CursorIcon::NoDrop => WCursorIcon::NoDrop,
        CursorIcon::NotAllowed => WCursorIcon::NotAllowed,
        CursorIcon::Grab => WCursorIcon::Grab,
        CursorIcon::Grabbing => WCursorIcon::Grabbing,
        CursorIcon::AllScroll => WCursorIcon::AllScroll,
        CursorIcon::ResizeHorizontal => WCursorIcon::EwResize,
        CursorIcon::ResizeNeSw => WCursorIcon::NeswResize,
        CursorIcon::ResizeNwSe => WCursorIcon::NwseResize,
        CursorIcon::ResizeVertical => WCursorIcon::NsResize,
        CursorIcon::ZoomIn => WCursorIcon::ZoomIn,
        CursorIcon::ZoomOut => WCursorIcon::ZoomOut,
    };

    window.set_cursor_icon(icon);
    window.set_cursor_visible(true);
}
