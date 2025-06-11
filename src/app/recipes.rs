use eframe::egui::{self, Id, Layout, Ui};

// Library of generic "recipes" for use everywhere and in future projects for fixing headache (and sometimes causing new ones)

// Recipe for drawing a "triple";
// Left, right aligned content with centred content between.
pub fn drawtriple<R>(
    ui: &mut Ui,
    left: impl FnOnce(&mut Ui) -> R,
    centre: impl FnOnce(&mut Ui) -> R,
    right: impl FnOnce(&mut Ui) -> R,
) {
    ui.horizontal(|ui| {
        // Draw left
        left(ui);

        // Create rtl box
        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
            // Draw right
            right(ui);

            // Draw centred box with content
            ui.vertical_centered(centre);
        });
    });
}

// Recipe for popup draw toggle
pub fn togglepopup(ui: &mut Ui, id: Id) {
	ui.memory_mut(|mem| mem.toggle_popup(id));
}