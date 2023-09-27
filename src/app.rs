use eframe::emath;
use egui::{
    plot::{Legend, Line, Plot, PlotBounds, PlotPoints, PlotResponse},
    Color32, Pos2, Rect, Sense, Stroke, Vec2,
};

const MINV: f64 = 0.;
const MAXV: f64 = 10000.;
const MINP: f64 = 0.;
const MAXP: f64 = 100.;
const LINERES: i64 = 1000;
const GAMMA: f64 = 5. / 3.;

pub struct TemplateApp {
    pressure: f64,
    volume: f64,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            pressure: (MINP + MAXP) / 2.,
            volume: (MINV + MAXV) / 2.,
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { pressure, volume } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            let PlotResponse {
                response,
                inner: pointer_coordinate,
                ..
            } = Plot::new("my_plot")
                .legend(Legend::default())
                .height(500.)
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_double_click_reset(false)
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .show_x(false)
                .show_y(false)
                .show_axes([false, false])
                .show(ui, |plot_ui| {
                    plot_ui.set_plot_bounds(PlotBounds::from_min_max([MINV, MINP], [MAXV, MAXP]));
                    plot_ui.line(
                        Line::new(
                            (0..LINERES)
                                .map(|i| {
                                    let v = MINV + i as f64 * (MAXV - MINV) / LINERES as f64;
                                    [v, *pressure * *volume / v]
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
                                    [v, *pressure * (*volume).powf(GAMMA) / v.powf(GAMMA)]
                                })
                                .collect::<PlotPoints>(),
                        )
                        .name("adiabatic"),
                    );
                    plot_ui.line(
                        Line::new(PlotPoints::new(Vec::from([
                            [MINV, *pressure],
                            [MAXV, *pressure],
                        ])))
                        .name("isobaric"),
                    );
                    plot_ui.line(
                        Line::new(PlotPoints::new(Vec::from([
                            [*volume, MINP],
                            [*volume, MAXP],
                        ])))
                        .name("isochoric"),
                    );
                    plot_ui.pointer_coordinate()
                });
            if response.dragged() && response.hovered() {
                if let Some(pointer_coordinate) = pointer_coordinate {
                    *volume = pointer_coordinate.x;
                    *pressure = pointer_coordinate.y;
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
                        x: response.rect.width() / 2. - ((*volume).sqrt() / 2.) as f32,
                        y: response.rect.height() / 2. - ((*volume).sqrt() / 2.) as f32,
                    },
                    max: Pos2 {
                        x: response.rect.width() / 2. + ((*volume).sqrt() / 2.) as f32,
                        y: response.rect.height() / 2. + ((*volume).sqrt() / 2.) as f32,
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
        });
    }
}
