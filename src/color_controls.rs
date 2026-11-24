//! Exact sRGB color entry in the color square's popup.
use bevy_egui::egui;

#[derive(Clone)]
struct ColorInputs {
    color: [u8; 3],
    hex: String,
    rgb: String,
}

impl ColorInputs {
    fn new(color: [u8; 3]) -> Self {
        Self {
            color,
            hex: format_hex(color),
            rgb: format_rgb(color),
        }
    }

    fn sync(&mut self, color: [u8; 3]) {
        if self.color != color {
            *self = Self::new(color);
        }
    }

    fn apply_hex(&mut self) -> Option<[u8; 3]> {
        let color = parse_hex(&self.hex)?;
        self.color = color;
        self.rgb = format_rgb(color);
        Some(color)
    }

    fn apply_rgb(&mut self) -> Option<[u8; 3]> {
        let color = parse_rgb(&self.rgb)?;
        self.color = color;
        self.hex = format_hex(color);
        Some(color)
    }
}

pub(super) fn edit_color(ui: &mut egui::Ui, label: &str, color: &mut [f32; 3]) -> egui::Response {
    ui.push_id(label, |ui| {
        let id = ui.make_persistent_id("color-inputs");
        let previous = *color;
        let srgb = to_srgb(*color);
        let viewport = ui.ctx().viewport_rect();
        let menu_width = crate::sidebar_dimensions(viewport.size()).0;
        let mut response = ui
            .horizontal(|ui| {
                let swatch = ui.add_sized(
                    [36.0, 36.0],
                    egui::Button::new("").fill(egui::Color32::from_rgb(srgb[0], srgb[1], srgb[2])),
                );
                let label = ui.label(label);
                swatch.labelled_by(label.id)
            })
            .inner
            .on_hover_text("Click to choose a color or enter hex / RGB values");

        egui::Popup::menu(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .at_position(egui::pos2(viewport.right() - 6.0, viewport.top() + 12.0))
            .align(egui::RectAlign {
                parent: egui::Align2::RIGHT_TOP,
                child: egui::Align2::RIGHT_TOP,
            })
            .align_alternatives(&[])
            .width((menu_width - 12.0).min(380.0))
            .show(|ui| {
                style_color_popup(ui);
                ui.heading(label);
                let mut inputs = ui
                    .data_mut(|data| data.get_temp::<ColorInputs>(id))
                    .unwrap_or_else(|| ColorInputs::new(srgb));
                inputs.sync(srgb);
                color_fields(ui, color, &mut inputs);
                ui.separator();
                ui.scope(|ui| {
                    // Keep the visual controls compact so the entire editor fits
                    // over the sidebar, including at the minimum window size.
                    ui.spacing_mut().interact_size.y = 28.0;
                    ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);
                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                    ui.spacing_mut().slider_width = 250.0;
                    ui.style_mut()
                        .text_styles
                        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
                    ui.style_mut()
                        .text_styles
                        .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
                    // Appearance stores sRGB; the float picker expects linear RGB.
                    let mut selected =
                        egui::Color32::from_rgb(inputs.color[0], inputs.color[1], inputs.color[2]);
                    if egui::color_picker::color_picker_color32(
                        ui,
                        &mut selected,
                        egui::color_picker::Alpha::Opaque,
                    ) {
                        let value = [selected.r(), selected.g(), selected.b()];
                        *color = value.map(|channel| f32::from(channel) / 255.0);
                        inputs = ColorInputs::new(value);
                    }
                });
                ui.data_mut(|data| data.insert_temp(id, inputs));
            });
        if *color != previous {
            response.mark_changed();
        }
        response
    })
    .inner
}

fn to_srgb(color: [f32; 3]) -> [u8; 3] {
    color.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn style_color_popup(ui: &mut egui::Ui) {
    let style = ui.style_mut();
    style.spacing.interact_size.y = 40.0;
    style.spacing.button_padding = egui::vec2(10.0, 8.0);
    style.spacing.item_spacing = egui::vec2(8.0, 10.0);
    style.spacing.slider_width = 320.0;
    style.visuals.numeric_color_space = egui::style::NumericColorSpace::GammaByte;
    for text_style in [egui::TextStyle::Body, egui::TextStyle::Button] {
        style
            .text_styles
            .insert(text_style, egui::FontId::proportional(18.0));
    }
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(18.0));
    style
        .text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(15.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(22.0));
}

fn color_fields(ui: &mut egui::Ui, color: &mut [f32; 3], inputs: &mut ColorInputs) {
    let hex = ui
        .horizontal(|ui| {
            let label = ui.add_sized([44.0, 40.0], egui::Label::new("Hex"));
            ui.add(
                egui::TextEdit::singleline(&mut inputs.hex)
                    .id_salt("hex")
                    .font(egui::TextStyle::Monospace)
                    .min_size(egui::vec2(0.0, 40.0))
                    .vertical_align(egui::Align::Center)
                    .desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .on_hover_text("Type 3 or 6 hex digits, with or without # (for example #FF8040).")
        })
        .inner;
    if hex.changed()
        && let Some(value) = inputs.apply_hex()
    {
        *color = value.map(|channel| f32::from(channel) / 255.0);
    }

    let rgb = ui
        .horizontal(|ui| {
            let label = ui.add_sized([44.0, 40.0], egui::Label::new("RGB"));
            ui.add(
                egui::TextEdit::singleline(&mut inputs.rgb)
                    .id_salt("rgb")
                    .font(egui::TextStyle::Monospace)
                    .min_size(egui::vec2(0.0, 40.0))
                    .vertical_align(egui::Align::Center)
                    .desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .on_hover_text(
                "Type R, G, B from 0 to 255, for example 255, 128, 64 or rgb(255, 128, 64).",
            )
        })
        .inner;
    if rgb.changed()
        && let Some(value) = inputs.apply_rgb()
    {
        *color = value.map(|channel| f32::from(channel) / 255.0);
    }
    let (hint, valid) = match (
        parse_hex(&inputs.hex).is_some(),
        parse_rgb(&inputs.rgb).is_some(),
    ) {
        (true, true) => ("Hex: #RRGGBB or #RGB · RGB: 0–255", true),
        (false, true) => ("Hex: use 3 or 6 digits (0–9, A–F).", false),
        (true, false) => ("RGB: three whole numbers from 0 to 255.", false),
        (false, false) => ("Hex: 3 or 6 digits. RGB: three values, 0–255.", false),
    };
    let color = if valid {
        ui.visuals().weak_text_color()
    } else {
        egui::Color32::LIGHT_RED
    };
    ui.add_sized(
        [ui.available_width(), 36.0],
        egui::Label::new(egui::RichText::new(hint).small().color(color)).wrap(),
    );

    for response in [hex, rgb] {
        if (response.has_focus() || response.lost_focus())
            && ui.input(|input| input.key_pressed(egui::Key::Escape))
        {
            *inputs = ColorInputs::new(inputs.color);
            response.surrender_focus();
        }
    }
}

fn format_hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

fn format_rgb([r, g, b]: [u8; 3]) -> String {
    format!("{r}, {g}, {b}")
}

fn parse_hex(text: &str) -> Option<[u8; 3]> {
    let text = text.trim();
    let digits = text
        .strip_prefix('#')
        .or_else(|| text.strip_prefix("0x"))
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    if !digits.bytes().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        3 => Some([
            ((value >> 8) & 0xf) as u8 * 17,
            ((value >> 4) & 0xf) as u8 * 17,
            (value & 0xf) as u8 * 17,
        ]),
        6 => Some([(value >> 16) as u8, (value >> 8) as u8, value as u8]),
        _ => None,
    }
}

fn parse_rgb(text: &str) -> Option<[u8; 3]> {
    let text = text.trim();
    let text = if text
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("rgb("))
    {
        text.get(4..)?.strip_suffix(')')?.trim()
    } else {
        text
    };
    let mut values = text.split(',').map(str::trim);
    let mut color = [0; 3];
    for channel in &mut color {
        let value = values.next()?;
        if value.is_empty() || !value.bytes().all(|digit| digit.is_ascii_digit()) {
            return None;
        }
        *channel = value.parse().ok()?;
    }
    values.next().is_none().then_some(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_common_hex_and_rgb_notation() {
        for text in ["#FF8040", "ff8040", " 0xFf8040 "] {
            assert_eq!(parse_hex(text), Some([255, 128, 64]), "{text}");
        }
        assert_eq!(parse_hex("#a3F"), Some([170, 51, 255]));
        for text in ["255, 128, 64", "rgb(255,128,64)", " RGB(255, 128, 64) "] {
            assert_eq!(parse_rgb(text), Some([255, 128, 64]), "{text}");
        }
        assert_eq!(parse_rgb("0, 0, 0"), Some([0, 0, 0]));
    }

    #[test]
    fn rejects_incomplete_out_of_range_and_non_ascii_values() {
        for text in [
            "",
            "#",
            "12",
            "1234",
            "#11223344",
            "GG0000",
            "ＦＦ0000",
            "+12345",
        ] {
            assert_eq!(parse_hex(text), None, "{text}");
        }
        for text in [
            "",
            "1,2",
            "1,2,3,4",
            "256,0,0",
            "-1,0,0",
            "1.5,0,0",
            "rgb(1,2,3",
            "1,,3",
            "NaN,0,0",
            "１,2,3",
        ] {
            assert_eq!(parse_rgb(text), None, "{text}");
        }
    }

    #[test]
    fn entry_sync_preserves_drafts_and_tracks_picker_or_preset_changes() {
        let mut inputs = ColorInputs::new([255, 128, 64]);
        inputs.hex = "#12".into();
        assert_eq!(inputs.apply_hex(), None);
        inputs.sync([255, 128, 64]);
        assert_eq!(inputs.hex, "#12");
        assert_eq!(inputs.color, [255, 128, 64]);

        inputs.hex = "#a3f".into();
        assert_eq!(inputs.apply_hex(), Some([170, 51, 255]));
        assert_eq!(inputs.rgb, "170, 51, 255");
        inputs.rgb = "rgb(0, 128, 255)".into();
        assert_eq!(inputs.apply_rgb(), Some([0, 128, 255]));
        assert_eq!(inputs.hex, "#0080FF");

        inputs.sync([215, 189, 150]);
        assert_eq!(inputs.hex, "#D7BD96");
        assert_eq!(inputs.rgb, "215, 189, 150");
    }

    #[test]
    fn color_squares_fit_the_sidebar_and_keep_values_hidden() {
        for screen in [
            egui::vec2(900.0, 620.0),
            egui::vec2(1320.0, 860.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let ctx = egui::Context::default();
            let (width, height, font) = crate::sidebar_dimensions(screen);
            let original = [215.0 / 255.0, 189.0 / 255.0, 150.0 / 255.0];
            let mut color = original;
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen)),
                    ..Default::default()
                },
                |root| {
                    egui::Panel::right("test-sidebar")
                        .exact_size(width)
                        .resizable(false)
                        .show(root, |ui| {
                            crate::style_sidebar(ui, height, font);
                            ui.indent("appearance", |ui| {
                                let right = ui.max_rect().right();
                                let square = edit_color(ui, "Light square", &mut color);
                                assert!(square.rect.width() >= 36.0);
                                assert!(square.rect.height() >= 36.0);
                                assert!(
                                    ui.min_rect().right() <= right + 1.0,
                                    "overflow at {screen:?}"
                                );
                            });
                        });
                },
            );
            assert!(!output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if ["Hex", "RGB", "#D7BD96"].contains(&text.galley.job.text.as_str()))
            }));
            output.drop_without_applying_deltas();
            assert_eq!(color, original);
        }
    }

    #[test]
    fn clicking_a_color_square_opens_readable_hex_and_rgb_fields_and_outside_click_closes() {
        for screen in [
            egui::vec2(900.0, 620.0),
            egui::vec2(1320.0, 860.0),
            egui::vec2(1920.0, 1080.0),
        ] {
            let ctx = egui::Context::default();
            let original = [215.0 / 255.0, 189.0 / 255.0, 150.0 / 255.0];
            let mut color = original;
            let mut draw = |events| {
                let mut square = None;
                let output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, screen)),
                        events,
                        ..Default::default()
                    },
                    |root| {
                        egui::Panel::right("test-sidebar")
                            .exact_size(crate::sidebar_dimensions(screen).0)
                            .show(root, |ui| {
                                ui.add_space(screen.y - 90.0);
                                square = Some(edit_color(ui, "Light square", &mut color));
                            });
                    },
                );
                (output, square.unwrap())
            };
            let (output, square) = draw(vec![]);
            output.drop_without_applying_deltas();
            let popup_id = egui::Popup::default_response_id(&square);
            assert!(!egui::Popup::is_id_open(&ctx, popup_id));

            for pressed in [true, false] {
                let (output, _) = draw(vec![
                    egui::Event::PointerMoved(square.rect.center()),
                    egui::Event::PointerButton {
                        pos: square.rect.center(),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ]);
                output.drop_without_applying_deltas();
            }
            let (output, _) = draw(vec![]);
            output.drop_without_applying_deltas();
            let (output, _) = draw(vec![]);
            assert!(egui::Popup::is_id_open(&ctx, popup_id));
            for value in ["Hex", "RGB", "#D7BD96", "215, 189, 150"] {
                let text = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Text(text) if text.galley.job.text == value => Some(text),
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("Missing popup field {value}"));
                assert!(
                    text.galley
                        .job
                        .sections
                        .iter()
                        .all(|section| section.format.font_id.size >= 18.0)
                );
            }
            let popup = ctx.read_response(popup_id).unwrap().rect;
            let game_right = screen.x - crate::sidebar_dimensions(screen).0;
            assert!(
                popup.min.x >= game_right,
                "Popup covers the game at {screen:?}: {popup:?}"
            );
            assert!(popup.max.x <= screen.x + 1.0);
            assert!(popup.min.y >= 0.0 && popup.max.y <= screen.y + 1.0);
            // The saturation/value square and hue bar must both be fully visible,
            // including when the menu opens near the bottom of a small window.
            for vertex_count in [37 * 37, 37 * 2] {
                let (clip, mesh) = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::Shape::Mesh(mesh) if mesh.vertices.len() == vertex_count => {
                            Some((shape.clip_rect, mesh))
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("Missing color slider with {vertex_count} vertices"));
                assert!(
                    clip.contains_rect(mesh.calc_bounds()),
                    "Color slider clipped at {screen:?}"
                );
                assert!(
                    popup.contains_rect(mesh.calc_bounds()),
                    "Color slider outside popup at {screen:?}"
                );
            }
            output.drop_without_applying_deltas();

            for pressed in [true, false] {
                let (output, _) = draw(vec![
                    egui::Event::PointerMoved(egui::pos2(10.0, 10.0)),
                    egui::Event::PointerButton {
                        pos: egui::pos2(10.0, 10.0),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: Default::default(),
                    },
                ]);
                output.drop_without_applying_deltas();
            }
            assert!(!egui::Popup::is_id_open(&ctx, popup_id));
            assert_eq!(color, original);
        }
    }
}
