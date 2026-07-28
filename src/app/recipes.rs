use eframe::egui::{self, Layout, Ui};

// Library of generic "recipes" for use everywhere and in future projects for fixing headache (and sometimes causing new ones)

// Recipe for drawing a "triple";
// Left, right aligned content with centred content between.
pub fn _drawtriple<R>(
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

// Basic drawtriple was giving me headache with double/triple mutable borrows, so we have another version for sharing a mutable value instead 
// (since the compiler has to assume all three need to borrow at the same time, which just isn't true)
pub fn drawtriple_mutpass<R, T>(
	ui: &mut Ui,
	passval : &mut T,
	left: impl FnOnce(&mut Ui, &mut T) -> R,
	centre: impl FnOnce(&mut Ui, &mut T) -> R,
	right: impl FnOnce(&mut Ui, &mut T) -> R
) {
	ui.horizontal(|ui| {
		// Draw left, include mutable passval
		left(ui, passval);

		// Create rtl
		ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
			// Draw right with passval
			right(ui, passval);

			// Draw centred box with content
            ui.vertical_centered(|ui| {
				centre(ui, passval);
			});
		});
	}); 
}