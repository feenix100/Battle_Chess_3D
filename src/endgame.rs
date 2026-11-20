//! Board-area check warning, victory plaque, and new-game confirmation.

use bevy_egui::egui::{self, Color32, Pos2, Rect, Shape, Stroke, Vec2};

use crate::{BattleGame, color_name, game::GameOutcome};

/// A passive warning: painting it never intercepts board clicks or camera drags.
pub(crate) fn check_banner(
    ctx: &egui::Context,
    game: &BattleGame,
    panel_width: f32,
) -> Option<Rect> {
    if !game.is_check() || game.outcome().is_some() {
        return None;
    }
    let viewport = ctx.viewport_rect();
    let board = Rect::from_min_max(
        viewport.min,
        egui::pos2(viewport.right() - panel_width, viewport.bottom()),
    );
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("check-warning"),
    ));
    let color = Color32::from_rgb(255, 215, 136);
    let font = egui::FontId::proportional((board.width() * 0.09).clamp(38.0, 64.0));
    let galley = painter.layout_no_wrap("CHECK!".to_owned(), font, color);
    let rect = Rect::from_center_size(board.center(), galley.size());
    painter.rect_filled(
        rect.expand2(egui::vec2(18.0, 10.0)),
        9,
        Color32::from_black_alpha(190),
    );
    painter.galley_with_override_text_color(
        rect.min + egui::vec2(2.0, 3.0),
        galley.clone(),
        Color32::BLACK,
    );
    // The bundled font has one weight; overlapping subpixel strokes make the
    // letters visibly bold without introducing a separately licensed font.
    for y in [-0.65, 0.0, 0.65] {
        for x in [-0.65, 0.0, 0.65] {
            painter.galley(rect.min + egui::vec2(x, y), galley.clone(), color);
        }
    }
    Some(rect)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewGameAction {
    Confirm,
    Back,
}

pub(crate) fn new_game_confirmation(
    ctx: &egui::Context,
    panel_width: f32,
) -> (
    Option<NewGameAction>,
    egui::ModalResponse<[egui::Response; 2]>,
) {
    let width = (ctx.viewport_rect().width() - panel_width - 48.0).clamp(280.0, 420.0);
    let gold = Color32::from_rgb(226, 194, 125);
    let area = egui::Modal::default_area(egui::Id::new("new-game-confirmation")).anchor(
        egui::Align2::CENTER_CENTER,
        egui::vec2(-panel_width * 0.5, 0.0),
    );
    let modal = egui::Modal::new(egui::Id::new("new-game-confirmation"))
        .area(area)
        .frame(
            egui::Frame::new()
                .fill(Color32::from_rgb(24, 30, 34))
                .stroke(Stroke::new(1.0, gold))
                .corner_radius(12)
                .inner_margin(20),
        )
        .show(ctx, |ui| {
            ui.set_width(width - 40.0);
            ui.label(
                egui::RichText::new("Start a new game?")
                    .size(25.0)
                    .strong()
                    .color(gold),
            );
            ui.add_space(7.0);
            ui.label(egui::RichText::new("Your current game will be cleared.").size(16.0));
            ui.add_space(14.0);
            let button_width = (ui.available_width() - ui.spacing().item_spacing.x) * 0.5;
            ui.horizontal(|ui| {
                [
                    ui.add_sized(
                        [button_width, 44.0],
                        egui::Button::new(
                            egui::RichText::new("Confirm")
                                .size(17.0)
                                .strong()
                                .color(Color32::from_rgb(38, 29, 14)),
                        )
                        .fill(gold),
                    ),
                    ui.add_sized(
                        [button_width, 44.0],
                        egui::Button::new(egui::RichText::new("Back to game").size(17.0)),
                    ),
                ]
            })
            .inner
        });
    let action = if modal.inner[0].clicked() {
        Some(NewGameAction::Confirm)
    } else if modal.inner[1].clicked() || modal.should_close() {
        Some(NewGameAction::Back)
    } else {
        None
    };
    (action, modal)
}

pub(crate) fn checkmate_menu(
    ctx: &egui::Context,
    game: &BattleGame,
    panel_width: f32,
) -> Option<egui::InnerResponse<egui::Response>> {
    let Some(GameOutcome::Checkmate { winner }) = game.outcome() else {
        return None;
    };
    let viewport = ctx.viewport_rect();
    let board_width = viewport.width() - panel_width;
    let width = (board_width * 0.44).clamp(300.0, 380.0);
    let compact = width < 340.0;
    let padding: i8 = if compact { 14 } else { 18 };
    let position = egui::pos2(
        viewport.left() + (board_width - width) * 0.5,
        viewport.top() + 16.0,
    );
    Some(
        egui::Area::new(egui::Id::new("checkmate-result"))
            .order(egui::Order::Foreground)
            .fixed_pos(position)
            .movable(false)
            .show(ctx, |ui| {
                let background = ui.painter().add(Shape::Noop);
                let frame = egui::Frame::new().inner_margin(padding).show(ui, |ui| {
                    ui.set_width(width - padding as f32 * 2.0);
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.vertical_centered(|ui| {
                        let (emblem, _) = ui.allocate_exact_size(
                            egui::vec2(40.0, if compact { 20.0 } else { 22.0 }),
                            egui::Sense::hover(),
                        );
                        ui.painter().add(crown(emblem.center()));
                        ui.label(
                            egui::RichText::new("Checkmate")
                                .size(if compact { 30.0 } else { 34.0 })
                                .strong()
                                .color(Color32::from_rgb(249, 222, 158)),
                        );
                        ui.label(
                            egui::RichText::new(format!("{} wins", color_name(winner)))
                                .size(if compact { 20.0 } else { 22.0 })
                                .strong()
                                .color(Color32::from_rgb(245, 239, 220)),
                        );
                        ui.add_space(7.0);
                        ui.add_sized(
                            [ui.available_width(), if compact { 40.0 } else { 44.0 }],
                            egui::Button::new(
                                egui::RichText::new("New game")
                                    .size(19.0)
                                    .strong()
                                    .color(Color32::from_rgb(38, 29, 14)),
                            )
                            .fill(Color32::from_rgb(226, 194, 125))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(255, 231, 170)))
                            .corner_radius(6),
                        )
                    })
                    .inner
                });
                ui.painter().set(background, plaque(frame.response.rect));
                frame.inner
            }),
    )
}

fn bevel(rect: Rect, cut: f32) -> Vec<Pos2> {
    vec![
        egui::pos2(rect.left() + cut, rect.top()),
        egui::pos2(rect.right() - cut, rect.top()),
        egui::pos2(rect.right(), rect.top() + cut),
        egui::pos2(rect.right(), rect.bottom() - cut),
        egui::pos2(rect.right() - cut, rect.bottom()),
        egui::pos2(rect.left() + cut, rect.bottom()),
        egui::pos2(rect.left(), rect.bottom() - cut),
        egui::pos2(rect.left(), rect.top() + cut),
    ]
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let blend = |from: u8, to: u8| (from as f32 + (to as f32 - from as f32) * t).round() as u8;
    Color32::from_rgb(
        blend(a.r(), b.r()),
        blend(a.g(), b.g()),
        blend(a.b(), b.b()),
    )
}

/// Vertex color bands give the brass a raised, polished rim instead of a flat
/// yellow outline. Geometry stays sharp at every display scale.
fn gradient(points: Vec<Pos2>, top: Color32, middle: Color32, bottom: Color32) -> Shape {
    let bounds = Rect::from_points(&points);
    let color = |point: Pos2| {
        let y = ((point.y - bounds.top()) / bounds.height().max(1.0)).clamp(0.0, 1.0);
        if y < 0.50 {
            mix(top, middle, y * 2.0)
        } else {
            mix(middle, bottom, (y - 0.5) * 2.0)
        }
    };
    let mut mesh = egui::epaint::Mesh::default();
    mesh.vertices.push(egui::epaint::Vertex::untextured(
        bounds.center(),
        color(bounds.center()),
    ));
    for &point in &points {
        mesh.vertices
            .push(egui::epaint::Vertex::untextured(point, color(point)));
    }
    for i in 0..points.len() as u32 {
        mesh.indices
            .extend([0, i + 1, (i + 1) % points.len() as u32 + 1]);
    }
    Shape::mesh(mesh)
}

fn plaque(rect: Rect) -> Shape {
    let gold = Color32::from_rgb(219, 177, 89);
    let light = Color32::from_rgb(255, 237, 177);
    let bronze = Color32::from_rgb(119, 77, 28);
    let mut shapes = vec![
        Shape::convex_polygon(
            bevel(rect.expand(3.0).translate(egui::vec2(0.0, 5.0)), 15.0),
            Color32::from_black_alpha(100),
            Stroke::NONE,
        ),
        gradient(bevel(rect, 12.0), light, gold, bronze),
        gradient(
            bevel(rect.shrink(1.2), 10.8),
            Color32::from_rgb(196, 148, 63),
            light,
            Color32::from_rgb(174, 126, 48),
        ),
        gradient(
            bevel(rect.shrink(3.3), 8.8),
            Color32::from_rgb(70, 44, 19),
            gold,
            Color32::from_rgb(247, 219, 147),
        ),
        gradient(
            bevel(rect.shrink(5.0), 7.5),
            Color32::from_rgb(44, 39, 30),
            Color32::from_rgb(26, 31, 32),
            Color32::from_rgb(13, 22, 28),
        ),
        Shape::closed_line(
            bevel(rect.shrink(7.5), 6.0),
            Stroke::new(0.7, Color32::from_rgba_unmultiplied(219, 177, 89, 95)),
        ),
    ];
    // Small laurel branches frame the title without covering the winner text.
    let y = rect.top() + rect.height() * 0.35;
    for side in [-1.0, 1.0] {
        let x = if side < 0.0 {
            rect.left() + 30.0
        } else {
            rect.right() - 30.0
        };
        shapes.push(Shape::line(
            vec![
                egui::pos2(x + side * 3.0, y - 22.0),
                egui::pos2(x, y),
                egui::pos2(x + side * 4.0, y + 22.0),
            ],
            Stroke::new(0.85, gold),
        ));
        for i in 0..4 {
            let root = egui::pos2(x + side * (i as f32 - 1.5).abs(), y - 13.0 + i as f32 * 9.0);
            for branch in [1.0, -0.65] {
                let direction = egui::vec2(side * branch * 12.0, -8.0);
                let across = egui::vec2(-direction.y, direction.x).normalized() * 2.6;
                let middle = root + direction * 0.52;
                shapes.push(gradient(
                    vec![root, middle + across, root + direction, middle - across],
                    light,
                    gold,
                    bronze,
                ));
            }
        }
    }
    Shape::Vec(shapes)
}

fn crown(center: Pos2) -> Shape {
    let light = Color32::from_rgb(255, 237, 177);
    let gold = Color32::from_rgb(225, 184, 98);
    let points = [
        (-16.0, -5.0),
        (-8.0, 0.0),
        (0.0, -10.0),
        (8.0, 0.0),
        (16.0, -5.0),
        (12.0, 9.0),
        (-12.0, 9.0),
    ]
    .map(|(x, y)| center + Vec2::new(x, y))
    .to_vec();
    // The crown is concave, so make its raised band separately from its peaks.
    let mut shapes = vec![gradient(
        vec![
            center + egui::vec2(-12.0, 1.0),
            center + egui::vec2(12.0, 1.0),
            center + egui::vec2(12.0, 9.0),
            center + egui::vec2(-12.0, 9.0),
        ],
        light,
        gold,
        Color32::from_rgb(138, 91, 33),
    )];
    for peak in [
        [points[0], points[1], center + egui::vec2(-12.0, 6.0)],
        [points[1], points[2], points[3]],
        [points[3], points[4], center + egui::vec2(12.0, 6.0)],
    ] {
        shapes.push(Shape::convex_polygon(
            peak.to_vec(),
            gold,
            Stroke::new(0.6, light),
        ));
    }
    for point in [points[0], points[2], points[4]] {
        shapes.push(Shape::circle_filled(point, 1.7, light));
    }
    shapes.push(Shape::line_segment(
        [
            center + egui::vec2(-11.0, 7.0),
            center + egui::vec2(11.0, 7.0),
        ],
        Stroke::new(1.0, light),
    ));
    shapes.push(Shape::convex_polygon(
        vec![
            center + egui::vec2(0.0, 1.0),
            center + egui::vec2(2.4, 4.0),
            center + egui::vec2(0.0, 7.0),
            center + egui::vec2(-2.4, 4.0),
        ],
        Color32::from_rgb(117, 58, 31),
        Stroke::new(0.6, light),
    ));
    Shape::Vec(shapes)
}
