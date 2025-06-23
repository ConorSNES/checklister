use eframe::egui::{Button, Context, Response, Ui, Widget, WidgetText};
use rfd::FileDialog;

use crate::app::{currentact, App, CurrentAct};

pub fn draw_menubar(app: &mut App, ctx: &Context, ui: &mut Ui) {
    // Add navigation buttons.
    ui.style_mut().visuals.button_frame = false;
    {
        // File menu
        ui.menu_button("File", |ui| {
            // File menu popup contents
            ui.set_min_width(120.0);

			if ui.button("New list").clicked() {
				app.action = CurrentAct::Confirm(Box::new(CurrentAct::New));
			}

			ui.separator();

            // Export function (pending implementation)
            if ui.button("Export json...").clicked() {
                do_export(app);
            }

            // Import function (pending implementation)
            if ui.button("Import json...").clicked() {
                do_import(app);
            }

            ui.separator();

            draw_themeconfig(ctx, ui);

            // Exit program
            if shortcutbutton(ui, "Exit", "F4").clicked() {
                app.action = CurrentAct::Confirm(Box::new(CurrentAct::Exit));
            }
        });
    }
    {
        // Edit menu
        ui.menu_button("Edit", |ui| {
            // Edit menu popup contents
            ui.set_min_width(120.0);

            // Find entry utility (pending implementation)
            if shortcutbutton(ui, "Find", ctx.format_shortcut(&App::KEYCOMBO_FIND)).clicked() {
                app.action = CurrentAct::Find(String::new());
            }

            ui.separator();

            // Autosort pref

            // Sort button
            if ui.button("Sort now").clicked() {
                app.action = CurrentAct::Sort;
            }

            // Clean up button
            if ui.button("Cleanup").clicked() {
                app.action = CurrentAct::Confirm(Box::new(CurrentAct::Cleanup));
            }
        });
    }
    {
        // Help menu
        ui.menu_button("Help", |ui| {
            ui.set_min_width(120.0);

            if ui.button("Usage hints").clicked() {
                app.action = currentact::hints();
            }

            if ui.button("About").clicked() {
                app.action = currentact::appinfo();
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

// Showing a button with a shortcut
fn shortcutbutton(ui: &mut Ui, text: impl Into<WidgetText>, shortcut: impl Into<WidgetText>) -> Response {
	Button::new(text).shortcut_text(shortcut).ui(ui)
}

fn imp_exp_dialog() -> FileDialog {
	FileDialog::new()
	.add_filter("json", &["json"])
	.add_filter("All files", &[""])
	.set_directory("/")
}

fn do_import(app: &mut App) {
	// Query user for target file
	let targpath = imp_exp_dialog().set_title("Import from...").pick_file();

	if let Some(v) = targpath {
		app.action = CurrentAct::Confirm(Box::new(CurrentAct::Import(v)));
	}
}

fn do_export(app: &mut App) {
	// Query user for target file
	let targpath = imp_exp_dialog().set_title("Export to...").save_file();

	if let Some(v) = targpath {
		app.action = CurrentAct::Export(v);
	}
}
