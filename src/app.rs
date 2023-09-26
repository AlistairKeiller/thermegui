use egui::plot::{Legend, Line, Plot, PlotBounds, PlotPoints, PlotResponse};

const MINV: f64 = 0.;
const MAXV: f64 = 1.;
const MINP: f64 = 0.;
const MAXP: f64 = 1.;
const LINERES: i64 = 1000;
const GAMMA: f64 = 5. / 3.;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    pressure: f64,
    volume: f64,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            pressure: 0.5,
            volume: 0.5,
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

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
        });
    }
}
