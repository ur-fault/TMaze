use eframe::{
    self,
    egui::{
        self,
        plot::{CoordinatesFormatter, GridInput, GridMark, PlotPoint, PlotPoints, Polygon},
    },
    epaint::{ahash::HashMap, Color32},
    App, NativeOptions,
};

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "TMaze Editor",
        NativeOptions {
            ..Default::default()
        },
        Box::new(|_cc| Box::new(Editor::new())),
    )
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum Tool {
    Select,
    Draw,
    Delete,
}

struct Editor {
    cells: HashMap<(i32, i32), bool>,
    box_selection: Option<((i32, i32), (i32, i32))>,
    tool: Tool,
}

impl Editor {
    fn new() -> Self {
        Self {
            cells: HashMap::default(),
            box_selection: None,
            tool: Tool::Select,
        }
    }
}

impl App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let spacer = |i: GridInput| {
            (i.bounds.0.round() as i32..i.bounds.1.ceil() as i32)
                .map(|v| GridMark {
                    value: v as f64,
                    step_size: 1.0,
                })
                .collect()
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::plot::Plot::new("Maze Plot")
                .show_x(false)
                .show_y(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .data_aspect(1.0)
                .x_grid_spacer(spacer)
                .y_grid_spacer(spacer)
                .allow_drag(false)
                .allow_double_click_reset(false)
                .coordinates_formatter(
                    egui::plot::Corner::LeftBottom,
                    CoordinatesFormatter::new(|p, _| {
                        format!("({:.0}, {:.0})", p.x.ceil(), p.y.ceil())
                    }),
                )
                .show(ui, |ui| {
                    // Plot movement with middle button, because we want to use primary button
                    // for interaction with the Editor
                    if ctx.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
                        ui.translate_bounds(-ui.pointer_coordinate_drag_delta());
                    }

                    if let Some(mouse) = ui.pointer_coordinate() {
                        ui.polygon(draw_rectangle(
                            mouse.x.floor() as i32,
                            mouse.y.floor() as i32,
                            1,
                            1,
                        ))
                    }

                    
                    if ctx.input(|i| i.pointer.is_decidedly_dragging() && i.pointer.primary_down())
                    {
                        let start =
                            ui.plot_from_screen(ctx.input(|i| i.pointer.press_origin().unwrap()));
                        let end =
                            ui.plot_from_screen(ctx.input(|i| i.pointer.interact_pos().unwrap()));

                        let (sx, sy) = (
                            start.x.min(end.x).floor() as i32,
                            start.y.min(end.y).floor() as i32,
                        );
                        let (ex, ey) = (
                            start.x.max(end.x).ceil() as i32,
                            start.y.max(end.y).ceil() as i32,
                        );

                        ui.polygon(draw_rectangle(sx, sy, ex - sx, ey - sy).color(Color32::YELLOW));
                        self.box_selection = Some(((sx, sy), (ex, ey)));
                    } else if let Some(((sx, sy), (ex, ey))) = self.box_selection {
                        self.box_selection = None;
                        for x in sx..ex {
                            for y in sy..ey {
                                self.cells
                                    .entry((x, y))
                                    .and_modify(|s| *s = !*s)
                                    .or_insert(true);
                            }
                        }
                    }

                    if ui.plot_clicked() {
                        if let Some(mouse) = ui.pointer_coordinate() {
                            let x = mouse.x.floor() as i32;
                            let y = mouse.y.floor() as i32;
                            self.cells
                                .entry((x, y))
                                .and_modify(|s| *s = !*s)
                                .or_insert(true);
                        }
                    }

                    for ((x, y), cell) in self.cells.iter() {
                        if *cell {
                            ui.polygon(
                                draw_rectangle(*x, *y, 1, 1)
                                    .stroke((0.0, Color32::BLACK))
                                    .color(Color32::LIGHT_BLUE),
                            )
                        }
                    }
                });
        });

        egui::Window::new("Tools").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.radio_value(&mut self.tool, Tool::Select, "Select");
                ui.radio_value(&mut self.tool, Tool::Draw, "Draw");
                ui.radio_value(&mut self.tool, Tool::Delete, "Delete");
            });
        });
    }
}

fn draw_rectangle(x: i32, y: i32, width: i32, height: i32) -> Polygon {
    Polygon::new(PlotPoints::Owned(vec![
        PlotPoint::new(x as f64, y as f64),
        PlotPoint::new(x as f64 + width as f64, y as f64),
        PlotPoint::new(x as f64 + width as f64, y as f64 + height as f64),
        PlotPoint::new(x as f64, y as f64 + height as f64),
    ]))
}
