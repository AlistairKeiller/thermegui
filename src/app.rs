use eframe::emath;
use egui::{
    plot::{Legend, Line, Plot, PlotBounds, PlotPoint, PlotPoints, PlotResponse},
    CollapsingHeader, Color32, Frame, Pos2, Rect, Sense, Stroke, Vec2,
};

const MINV: f64 = 0.; // m^3
const MAXV: f64 = 10.; // m^3
const MINP: f64 = 0.; // Pa
const MAXP: f64 = 10.; // Pa
const LINERES: i64 = 1000;
const R: f64 = 8.314; //J mol^-1 K^-1
const N: f64 = 1.; // mol
const CV: f64 = 3. / 2. * R; // J mol^-1 K^-1
const CP: f64 = 5. / 2. * R; // J mol^-1 K^-1
const GAMMA: f64 = CP / CV;

const PIXELS_TO_METERS: f64 = 100.;

pub struct TemplateApp {
    pressure: f64,
    volume: f64,
    work: f64,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            pressure: (MINP + MAXP) / 2.,
            volume: (MINV + MAXV) / 2.,
            work: 0.,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let pressure = self.pressure;
            let volume = self.volume;

            let PlotResponse {
                response,
                inner: pointer_coordinate,
                ..
            } = Plot::new("my_plot")
                .label_formatter(move |name, value| {
                    let delta_u = CV / R * (value.x * value.y - pressure * volume);
                    let work = match name {
                        "isothermic" => pressure * volume * (value.x / volume).ln(),
                        "adiabatic" => -delta_u,
                        _ => (value.y + pressure) / 2. * (value.x - volume),
                    };
                    format!(
                        "{}\nV = {:.1} m^3\nP = {:.1} Pa\nΔU = {:.1} J\nW = {:.1} J\nQ = {:.1} J",
                        name,
                        value.x,
                        value.y,
                        delta_u,
                        work,
                        delta_u + work
                    )
                })
                .legend(Legend::default())
                .height(500.)
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_double_click_reset(false)
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .show_axes([false, false])
                .show(ui, |plot_ui| {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max([MINV, MINP], [MAXV, MAXP]));
                    plot_ui.line(
                        Line::new(
                            (0..LINERES)
                                .map(|i| {
                                    let v = MINV + i as f64 * (MAXV - MINV) / LINERES as f64;
                                    [v, self.pressure * self.volume / v]
                                })
                                .collect::<PlotPoints>(),
                        )
                        .name("isothermic"),
                    );
                    plot_ui.line(
                        Line::new(
                            (0..LINERES)
                                .map(|i| {
                                    let v = MINV + i as f64 * (MAXV - MINV) / LINERES as f64;
                                    [v, self.pressure * self.volume.powf(GAMMA) / v.powf(GAMMA)]
                                })
                                .collect::<PlotPoints>(),
                        )
                        .name("adiabatic"),
                    );
                    plot_ui.line(
                        Line::new(
                            (0..LINERES)
                                .map(|i| [i as f64 * (MAXV - MINV) / LINERES as f64, self.pressure])
                                .collect::<PlotPoints>(),
                        )
                        .name("isobaric"),
                    );
                    plot_ui.line(
                        Line::new(
                            (0..LINERES)
                                .map(|i| [self.volume, i as f64 * (MAXP - MINP) / LINERES as f64])
                                .collect::<PlotPoints>(),
                        )
                        .name("isochoric"),
                    );
                    // efficent drawing but does not work with axis selector
                    // plot_ui.line(
                    //     Line::new(PlotPoints::new(Vec::from([
                    //         [MINV, *pressure],
                    //         [MAXV, *pressure],
                    //     ])))
                    //     .name("isobaric"),
                    // );
                    // plot_ui.line(
                    //     Line::new(PlotPoints::new(Vec::from([
                    //         [*volume, MINP],
                    //         [*volume, MAXP],
                    //     ])))
                    //     .name("isochoric"),
                    // );
                    plot_ui.pointer_coordinate()
                });
            if response.dragged_by(egui::PointerButton::Primary) && response.hovered() {
                if let Some(pointer_coordinate) = pointer_coordinate {
                    self.volume = pointer_coordinate.x;
                    self.pressure = pointer_coordinate.y;
                }
            }
            let (response, painter) = ui.allocate_painter(
                Vec2::new(ui.available_width(), ui.available_height()),
                Sense::hover(),
            );
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                response.rect,
            );
            painter.add(eframe::epaint::RectShape {
                rect: to_screen.transform_rect(Rect {
                    min: Pos2 {
                        x: response.rect.width() / 2.
                            - (PIXELS_TO_METERS * self.volume.sqrt() / 2.) as f32,
                        y: response.rect.height() / 2.
                            - (PIXELS_TO_METERS * self.volume.sqrt() / 2.) as f32,
                    },
                    max: Pos2 {
                        x: response.rect.width() / 2.
                            + (PIXELS_TO_METERS * self.volume.sqrt() / 2.) as f32,
                        y: response.rect.height() / 2.
                            + (PIXELS_TO_METERS * self.volume.sqrt() / 2.) as f32,
                    },
                }),
                rounding: egui::Rounding {
                    nw: 0.,
                    ne: 0.,
                    sw: 0.,
                    se: 0.,
                },
                fill: Color32::TRANSPARENT,
                stroke: Stroke {
                    width: 5.,
                    color: Color32::GRAY,
                },
            });
            // Frame::popup(ui.style())
            //     .stroke(Stroke::NONE)
            //     .show(ui, |ui| {
            //         ui.set_max_width(270.0);
            //         CollapsingHeader::new("Settings").show(ui, |ui| ui.label("test"));
            //     });
        });
    }
}
