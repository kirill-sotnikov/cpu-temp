use std::time::Duration;

use eframe;
use eframe::egui;
use egui::{Id, Margin, PointerButton, Sense, ViewportCommand};
use sysinfo::{Components, Disks, Networks, System};

#[derive(Clone)]
struct ComponentTemperature {
    label: String,
    temperature: f32,
}

fn get_cpu_temperature() -> Option<ComponentTemperature> {
    let components = Components::new_with_refreshed_list();
    println!("=> components:");

    let mut component_temps = Vec::new();

    for component in &components {
        if let Some(temperature) = component.temperature() {
            component_temps.push(ComponentTemperature {
                label: component.label().to_string(),
                temperature,
            });
        }
    }

    // component_temps.sort_by(|a, b| a.temperature.partial_cmp(&b.temperature).unwrap());

    let mut max: Option<ComponentTemperature> = None;

    for component in &component_temps {
        if component.label.starts_with("PMU") && component.label.contains("tdie") {
            println!("{:<30} {:.1}°C", component.label, component.temperature);

            if let Some(max_temp) = &mut max {
                if component.temperature > max_temp.temperature {
                    *max_temp = component.clone();
                }
            } else {
                max = Some(component.clone());
            }
        }
    }

    max
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([40.0, 15.0])
            .with_decorations(false)
            .with_position([100.0, 7.0])
            .with_always_on_top()
            .with_transparent(true)
            .with_resizable(false).with_window_level(egui::WindowLevel::AlwaysOnTop),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            Ok(Box::<MyApp>::default())
        }),
    )
    .unwrap();
}

struct MyApp {
    cpu_temperature: Option<ComponentTemperature>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            cpu_temperature: get_cpu_temperature(),
        }
    }
}

impl eframe::App for MyApp {
    // fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    //     // egui::Rgba::TRANSPARENT.to_array()
    //     egui::Rgba::GREEN.to_array()
    // }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.cpu_temperature = get_cpu_temperature();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(Margin::from(0)))
            .show(ctx, |ui| {
                ui.style_mut().interaction.selectable_labels = false;
                println!("render");
                if let Some(cpu_temperature) = &self.cpu_temperature {
                    let app_rect = ui.max_rect();
                    let title_rect = {
                        let mut rect = app_rect;
                        rect.max.y = rect.min.y + 20.0;
                        rect
                    }.shrink(4.0);

                    

                    let title_bar_response =ui.interact(title_rect, Id::new("title_bar"), Sense::click_and_drag());

                    if title_bar_response.drag_started_by(PointerButton::Primary) {
                        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                    }
                    ui.label(format!("{:.1}°C", cpu_temperature.temperature));
                } else {
                    println!("No CPU temperature data available");
                }
            });

        ctx.request_repaint_after(Duration::from_secs(10));
    }
}

