use amethyst::{
    assets::LoaderBundle,
    core::{
        dispatcher::DispatcherBuilder,
        transform::TransformBundle,
    },
    ecs::{
        System,
        SystemBuilder,
    },
    input::{
        is_close_requested,
        is_key_down,
        VirtualKeyCode,
    },
    renderer::{
        rendy::hal::command::ClearColor,
        types::DefaultBackend,
        RenderToWindow,
        RenderingBundle,
    },
    utils::application_root_dir,
    Application,
    GameData,
    SimpleState,
    SimpleTrans,
    StateData,
    StateEvent,
};
use amethyst_egui::{
    bundle::EguiBundle,
    egui,
    plugin::RenderEgui,
    system::{
        EguiConfig,
        EguiContext,
    },
};
use amethyst_input::InputBundle;

struct HelloWorldState;

impl SimpleState for HelloWorldState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        log::debug!("Staring ExampleStart");

        data.resources.insert(UiState::default());

        let mut _egui_config = data.resources.get_mut::<EguiConfig>().unwrap();
        //egui_config.allow_webbrowser = true;
        //egui_config.allow_clipboard = true;
    }

    fn handle_event(&mut self, _data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return SimpleTrans::Quit;
            }
        }

        SimpleTrans::None
    }
}

#[derive(Debug, Default)]
struct UiState {
    label: String,
    value: f32,
    painting: Painting,
    inverted: bool,
}

struct HelloWorldSystem;

impl System for HelloWorldSystem {
    fn build(self) -> Box<dyn amethyst::ecs::ParallelRunnable + 'static> {
        Box::new(
            SystemBuilder::new("HelloWorldSystem")
                .read_resource::<EguiContext>()
                .write_resource::<UiState>()
                .build(|_commands, _world, (egui_ctx, ui_state), _queries| {
                    let ctx = egui_ctx.ctx().unwrap();

                    /*let mut load = false;
                    let mut remove = false;
                    let mut invert = false;*/

                    egui::SidePanel::left("side_panel")
                        .default_width(200.0)
                        .show(ctx, |ui| {
                            ui.heading("Side Panel");

                            ui.horizontal(|ui| {
                                ui.label("Write something: ");
                                ui.text_edit_singleline(&mut ui_state.label);
                            });

                            ui.add(egui::Slider::new(&mut ui_state.value, 0.0..=10.0).text("value"));
                            if ui.button("Increment").clicked() {
                                ui_state.value += 1.0;
                            }

                            ui.allocate_space(egui::Vec2::new(1.0, 100.0));
                            // TODO: User textures
                            /*ui.horizontal(|ui| {
                                load = ui.button("Load").clicked();
                                invert = ui.button("Invert").clicked();
                                remove = ui.button("Remove").clicked();
                            });*/

                            /*ui.add(egui::widgets::Image::new(
                                egui::TextureId::User(BEVY_TEXTURE_ID),
                                [256.0, 256.0],
                            ));*/

                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                                ui.add(
                                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                                );
                            });
                        });

                    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                        // The top panel is often a good place for a menu bar:
                        egui::menu::bar(ui, |ui| {
                            egui::menu::menu(ui, "File", |ui| {
                                if ui.button("Quit").clicked() {
                                    std::process::exit(0);
                                }
                            });
                        });
                    });

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Egui Template");
                        ui.hyperlink("https://github.com/emilk/egui_template");
                        ui.add(egui::github_link_file_line!(
                            "https://github.com/emilk/egui_template/blob/master/",
                            "Direct link to source code."
                        ));
                        egui::warn_if_debug_build(ui);

                        ui.separator();

                        ui.heading("Central Panel");
                        ui.label("The central panel is the region left after adding TopPanel's and SidePanel's");
                        ui.label("It is often a great place for big things, like drawings:");

                        ui.heading("Draw with your mouse to paint:");
                        ui_state.painting.ui_control(ui);
                        egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                            ui_state.painting.ui_content(ui);
                        });
                    });

                    egui::Window::new("Window")
                        .scroll(true)
                        .show(ctx, |ui| {
                            ui.label("Windows can be moved by dragging them.");
                            ui.label("They are automatically sized based on contents.");
                            ui.label("You can turn on resizing and scrolling if you like.");
                            ui.label("You would normally chose either panels OR windows.");
                        });

                    /*
                    if invert {
                        ui_state.inverted = !ui_state.inverted;
                    }
                    if load || invert {
                        todo!()
                    }
                    if remove {
                        todo!()
                    }*/
                }),
        )
    }
}

#[derive(Debug)]
struct Painting {
    lines: Vec<Vec<egui::Vec2>>,
    stroke: egui::Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), egui::Sense::drag());
        let rect = response.rect;

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = pointer_pos - rect.min;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
            }
        }
        else if !current_line.is_empty() {
            self.lines.push(vec![]);
        }

        for line in &self.lines {
            if line.len() >= 2 {
                let points: Vec<egui::Pos2> = line.iter().map(|p| rect.min + *p).collect();
                painter.add(egui::Shape::line(points, self.stroke));
            }
        }
    }
}

fn main() -> Result<(), amethyst_error::Error> {
    amethyst_core::Logger::from_config(Default::default())
        .level_for("amethyst_egui", log::LevelFilter::Debug)
        .level_for("hello_world", log::LevelFilter::Debug)
        .start();

    let app_root = application_root_dir()?;
    let assets_directory = app_root.join("examples/assets");
    let display_config_path = app_root.join("examples/config/display.ron");

    let mut dispatcher = DispatcherBuilder::default();
    dispatcher.add_bundle(LoaderBundle);
    dispatcher.add_bundle(TransformBundle);
    dispatcher.add_bundle(InputBundle::new());
    dispatcher.add_bundle(EguiBundle::default());
    dispatcher.add_system(HelloWorldSystem);
    dispatcher.add_bundle(
        RenderingBundle::<DefaultBackend>::new()
            .with_plugin(
                RenderToWindow::from_config_path(display_config_path)?.with_clear(ClearColor {
                    float32: [0.1, 0.03, 0.35, 1.0],
                }),
            )
            .with_plugin(RenderEgui::default()),
    );

    let game = Application::build(assets_directory, HelloWorldState)?.build(dispatcher)?;
    game.run();
    Ok(())
}
