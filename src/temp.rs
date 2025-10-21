use sysinfo::Components;

#[derive(Clone)]
pub struct ComponentTemperature {
    pub label: String,
    pub temperature: f32,
}

pub fn get_cpu_temperature() -> Option<ComponentTemperature> {
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
