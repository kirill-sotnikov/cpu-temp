use std::{thread, time::Duration};

use image::{ImageBuffer, Rgba};
use imageproc::drawing::{draw_filled_circle_mut, draw_text_mut};
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};

use ab_glyph::FontRef;
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem},
};

mod temp;

enum UserEvent {
    Tick,
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    let proxy = event_loop.create_proxy();
    thread::spawn(move || {
        loop {
            let _ = proxy.send_event(UserEvent::Tick);
            thread::sleep(Duration::from_secs(10));
        }
    });

    let tray_menu = Menu::new();

    let quit_i = MenuItem::new("Quit", true, None);

    tray_menu.append_items(&[&quit_i]).unwrap();

    let mut tray_icon = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_icon(create_icon_with_text(0))
                        .with_menu(Box::new(tray_menu.clone()))
                        .build()
                        .unwrap(),
                );

                #[cfg(target_os = "macos")]
                unsafe {
                    use objc2_core_foundation::{CFRunLoopGetMain, CFRunLoopWakeUp};

                    let rl = CFRunLoopGetMain().unwrap();
                    CFRunLoopWakeUp(&rl);
                }
            }

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                println!("{event:?}");

                if event.id == quit_i.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }

            Event::UserEvent(UserEvent::Tick) => {
                let temp = temp::get_cpu_temperature();
                if let Some(component_temp) = temp {
                    let temp = component_temp.temperature as u8;
                    tray_icon = Some(
                        TrayIconBuilder::new()
                            .with_icon(create_icon_with_text(temp))
                            .with_menu(Box::new(tray_menu.clone()))
                            .build()
                            .unwrap(),
                    );
                }
            }

            _ => {}
        }
    })
}

fn create_icon_with_text(temp: u8) -> tray_icon::Icon {
    let text = format!("{}°", temp);
    let count_of_numbers = temp.to_string().len() as u32;
    let width = 20 + 80 + 20 + count_of_numbers * 60 + 40 + 20;
    let height = 128u32;
    let mut img = ImageBuffer::new(width, height);

    // Заполняем белым фоном
    for y in 0..height {
        for x in 0..width {
            let mut draw = true;

            // Радиус скругления
            let radius = 40.0;

            // Проверяем углы
            // Верхний левый угол
            if x < radius as u32 && y < radius as u32 {
                let dx = radius - x as f32;
                let dy = radius - y as f32;
                if dx * dx + dy * dy > radius * radius {
                    draw = false;
                }
            }
            // Верхний правый угол
            else if x >= width - radius as u32 && y < radius as u32 {
                let dx = x as f32 - (width as f32 - radius);
                let dy = radius - y as f32;
                if dx * dx + dy * dy > radius * radius {
                    draw = false;
                }
            }
            // Нижний левый угол
            else if x < radius as u32 && y >= height - radius as u32 {
                let dx = radius - x as f32;
                let dy = y as f32 - (height as f32 - radius);
                if dx * dx + dy * dy > radius * radius {
                    draw = false;
                }
            }
            // Нижний правый угол
            else if x >= width - radius as u32 && y >= height - radius as u32 {
                let dx = x as f32 - (width as f32 - radius);
                let dy = y as f32 - (height as f32 - radius);
                if dx * dx + dy * dy > radius * radius {
                    draw = false;
                }
            }

            if draw {
                img.put_pixel(x, y, Rgba([255, 255, 255, 230]));
            }
        }
    }

    // Загружаем встроенный шрифт
    let font_data = include_bytes!("../Roboto_Condensed-Medium.ttf");
    let font = FontRef::try_from_slice(font_data as &[u8]).expect("Ошибка загрузки шрифта");

    let circle_color = if temp < 75 {
        Rgba([0, 205, 0, 255])
    } else if temp < 90 {
        Rgba([255, 165, 0, 255])
    } else {
        Rgba([235, 0, 0, 255])
    };

    draw_filled_circle_mut(&mut img, (60, height as i32 / 2), 40, circle_color);

    // Рисуем текст (чёрный цвет)
    draw_text_mut(
        &mut img,
        Rgba([0, 0, 0, 215]),
        130,
        4,     // 16
        120.0, // 100.0
        &font,
        &text,
    );

    let rgba = img.into_raw();
    tray_icon::Icon::from_rgba(rgba, width, height).unwrap()
}
