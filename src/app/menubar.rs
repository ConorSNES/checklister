use eframe::egui::{Context, Ui};

use crate::app::{App, CurrentAct};

pub fn draw_menubar(app: &mut App, ctx: &Context, ui: &mut Ui) {
    // Add navigation buttons.
    ui.style_mut().visuals.button_frame = false;
    {
        // File menu
        ui.menu_button("File", |ui| {
            // File menu popup contents
            ui.set_min_width(120.0);

            // Export function (pending implementation)
            if ui.button("Export tasks...").clicked() {
                println!("Export unimplemented!");
            }

            // Import function (pending implementation)
            if ui.button("Import tasks...").clicked() {
                println!("Import unimplemented!");
            }

            ui.separator();

			draw_themeconfig(ctx, ui);

            // todo: show hotkeys in buttons
            // todo: confirm exit

            // Exit program
            if ui.button("Exit").clicked() {
                std::process::exit(0);
            }
        });
    }
    {
        // Edit menu
        ui.menu_button("Edit", |ui| {
            // Edit menu popup contents
            ui.set_min_width(120.0);

            // Find entry utility (pending implementation)
            if ui.button("Find").clicked() {
                app.action = CurrentAct::Find(String::new());
            }

            // Clean up button
            if ui.button("Cleanup").clicked() {
                app.action = CurrentAct::Confirm(Box::new(CurrentAct::Cleanup));
            }
        });
    }
}

fn draw_themeconfig(ctx: &Context, ui: &mut Ui) {
    ui.menu_button("Theme", |ui| {
        let mut theme = ctx.theme();
        ui.radio_value(
            &mut theme,
            ctx.system_theme().unwrap_or(eframe::egui::Theme::Dark),
            "Match System",
        );
        ui.radio_value(&mut theme, eframe::egui::Theme::Light, "Light");
        ui.radio_value(&mut theme, eframe::egui::Theme::Dark, "Dark");
        ctx.set_theme(theme);
    });
}
