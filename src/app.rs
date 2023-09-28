use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use eframe::emath;
use egui::{
    plot::{Legend, Line, Plot, PlotBounds, PlotPoints, PlotResponse},
    Color32, Frame, Pos2, Rect, Sense, Stroke, Vec2,
};
use rapier2d::prelude::*;

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

const METERS_TO_PIXELS: f64 = 100.;

// const WALL_THICKNESS: f64 = 0.1; // m
const BALL_RADIUS: f64 = 0.1; // m

pub struct TemplateApp {
    pressure: f64,
    volume: f64,
    work: f64,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    physics_hooks: (),
    event_handler: (),
    physics_pipeline: PhysicsPipeline,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let floor_body_handle = rigid_body_set.insert(
            RigidBodyBuilder::fixed()
                .translation(vector![0.0, 0.0])
                .build(),
        );
        collider_set.insert_with_parent(
            ColliderBuilder::cuboid(0.2, 0.2)
                .restitution(1.0)
                .friction(0.0)
                .build(),
            floor_body_handle,
            &mut rigid_body_set,
        );
        // let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
        // collider_set.insert(collider);

        let ball_body_handle = rigid_body_set.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![0.0, 1.0])
                .build(),
        );
        collider_set.insert_with_parent(
            ColliderBuilder::ball(BALL_RADIUS as f32)
                .restitution(1.0)
                .friction(0.0)
                .build(),
            ball_body_handle,
            &mut rigid_body_set,
        );

        Self {
            pressure: (MINP + MAXP) / 2.,
            volume: (MINV + MAXV) / 2.,
            work: 0.,
            gravity: vector![0.0, -1.0],
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set,
            collider_set,
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
            physics_pipeline: PhysicsPipeline::new(),
        }
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            let pressure = self.pressure;
            let volume = self.volume;
            let axes: Rc<RefCell<(f64,f64,f64)>> = Default::default();
            let axes_clone = axes.clone();

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
                    *axes_clone.borrow_mut() = (value.x,value.y,work);
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
                    plot_ui.pointer_coordinate()
                });

            if let Some(pointer_coordinate) = pointer_coordinate {
                if response.dragged_by(egui::PointerButton::Primary) && response.hovered() {
                    self.volume = pointer_coordinate.x;
                    self.pressure = pointer_coordinate.y;
                    self.work += (pointer_coordinate.y + pressure) / 2. * (pointer_coordinate.x - volume);
                }
                  if response.clicked_by(egui::PointerButton::Secondary) && response.hovered() {
                    (self.volume, self.pressure, self.work) = *axes.borrow();
                }
            }
            let (response, painter) = ui.allocate_painter(
                Vec2::new(ui.available_width(), ui.available_height()),
                Sense::hover(),
            );
            let to_screen = emath::RectTransform::from_to(
                Rect {
                    min: Pos2 {
                        x: -response.rect.width() / (2. * METERS_TO_PIXELS as f32),
                        y: response.rect.height() / (2. * METERS_TO_PIXELS as f32),
                    },
                    max: Pos2 {
                        x: response.rect.width() / (2. * METERS_TO_PIXELS as f32),
                        y: -response.rect.height() / (2. * METERS_TO_PIXELS as f32),
                    },
                },
                response.rect,
            );
            for (_, colider) in self.collider_set.iter() {
                if let Some(parent) = colider.parent() {
                    if let Some(body) = self.rigid_body_set.get(parent) {
                        if let Some(shape) = colider.shape().as_ball() {
                            painter.add(eframe::epaint::CircleShape {
                                center: to_screen.transform_pos(Pos2 {
                                    x: body.translation().x,
                                    y: body.translation().y,
                                }),
                                radius: shape.radius * METERS_TO_PIXELS as f32,
                                fill: Color32::GRAY,
                                stroke: Stroke::default(),
                            });
                        }
                        if let Some(shape) = colider.shape().as_cuboid() {
                            painter.add(eframe::epaint::RectShape {
                                rect: to_screen.transform_rect(Rect {
                                    min: Pos2 {
                                        x: body.translation().x - shape.half_extents.x,
                                        y: body.translation().y + shape.half_extents.y,
                                    },
                                    max: Pos2 {
                                        x: body.translation().x + shape.half_extents.x,
                                        y: body.translation().y - shape.half_extents.y,
                                    },
                                }),
                                rounding: egui::Rounding {
                                    nw: 0.,
                                    ne: 0.,
                                    sw: 0.,
                                    se: 0.,
                                },
                                fill: Color32::GRAY,
                                stroke: Stroke::default(),
                            });
                        }
                    }
                }
            }
            ui.allocate_ui_at_rect(
                Rect {
                    min: Pos2 { x: 10., y: 10. },
                    max: Pos2 { x: 250., y: 250. },
                },
                |ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        ui.collapsing("info", |ui| {
                            ui.label(format!(
                                "T= {:.1} K\nU = {:.1} J\nTotal W = {:.1} J\nTotal Q = {:.1}",
                                self.pressure * self.volume / (N * R),
                                CV / R * self.pressure * self.volume,
                                self.work,
                                CV / R * self.pressure * self.volume + self.work
                            ));
                            ui.collapsing("Keybindings", |ui| ui.label("Left Click/Drag: Move PV Diagram To Cursor Location\nRight Click: Move PV Diagram To Axes Location"));
                        })
                    })
                },
            );
        });
    }
}
