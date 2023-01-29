use std::{ops::Add, rc::Rc, cell::{Cell, RefCell}, f64::consts::E};

use eframe::{egui, CreationContext};
use chrono::{Datelike, NaiveDate, Weekday, IsoWeek, Duration, Days, Local};
use egui::{ColorImage, TextureOptions};
use crate::{timetable::{Timetable, Event, TimetableGetter, GetTimetableError}, config::{ConfigStore, Config}, events_table::EventsTable};

use crate::utils::load_image_from_memory;

const MAX_FUTURE_WEEKS: u64 = 4 * 12;

lazy_static! {
    pub static ref BREAK_IMAGE: ColorImage = load_image_from_memory(include_bytes!("../assets/break-area.png"))
        .expect("Failed to decode break area texture") as ColorImage;
}

struct AppAssets {
    break_texture: egui::TextureHandle
}

pub struct MainApp {
    shown_week: IsoWeek,
    shown_events: Vec<Event>,

    timetable_getter: Box<dyn TimetableGetter>,
    pub timetable: Option<Timetable>,

    config_store: Box<dyn ConfigStore>,
    config: Option<Config>,

    assets: Option<AppAssets>,
    vidko_textfield: String,

    screen: Option<Rc<RefCell<dyn Screen>>>
}

#[inline]
fn is_weekend(time: NaiveDate) -> bool {
    let day = time.weekday();
    return day == Weekday::Sat || day == Weekday::Sun;
}

fn get_current_week() -> IsoWeek {
    let now = Local::now();
    if is_weekend(now.date_naive()) {
        now.add(Duration::days(7)).iso_week()
    } else {
        now.iso_week()
    }
}

fn get_future_week(week_offset: u64) -> IsoWeek {
    let now_week = get_current_week();
    let year = now_week.year();
    let week = now_week.week();
    let week_date = NaiveDate::from_isoywd_opt(now_week.year(), now_week.week(), Weekday::Mon).expect("Invalid week or year given");
    week_date.checked_add_days(Days::new(7 * week_offset)).unwrap().iso_week()
}

trait Screen {
    fn show(&mut self, app: &mut MainApp, ctx: &egui::Context);
}

#[derive(Default)]
struct MainScreen {}
impl Screen for MainScreen {
    fn show(&mut self, app: &mut MainApp, ctx: &egui::Context) {
        use egui::*;
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                if ctx.input().key_pressed(Key::D) && app.shown_week < get_future_week(MAX_FUTURE_WEEKS) {
                    app.shift_shown_week(1);
                }
                if ctx.input().key_pressed(Key::A) && get_current_week() < app.shown_week {
                    app.shift_shown_week(-1);
                }
                if ctx.input().key_pressed(Key::S) {
                    app.set_shown_week(get_current_week());
                }
                if ctx.input().key_pressed(Key::F2) {
                    if ctx.style().visuals.dark_mode {
                        ctx.set_visuals(egui::Visuals::light());
                    } else {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                }

                let mut events_table = EventsTable::new(&app.shown_events);
                events_table.week = Some(app.shown_week);
                events_table.now = Some(Local::now().naive_local());
                events_table.break_texture = Some(app.assets.as_ref().unwrap().break_texture.clone());
                ui.add(events_table);
        });
    }
}

#[derive(Default)]
struct VidkoScreen {
    vidko_textfield: String,
    get_error: Option<GetTimetableError>
}
impl Screen for VidkoScreen {
    fn show(&mut self, app: &mut MainApp, ctx: &egui::Context) {
        use egui::*;

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Įveskite savo vidko kodą");
                    ui.horizontal(|ui| {
                        ui.label("Vidko: ");
                        ui.text_edit_singleline(&mut self.vidko_textfield);
                    });
                    if ui.button("Įvesti").clicked() {
                        match app.timetable_getter.get(&self.vidko_textfield) {
                            Ok(timetable) => {
                                if app.config.is_none() {
                                    app.config = Some(Config::default());
                                }
                                app.config.as_mut().unwrap().vidko = Some(self.vidko_textfield.clone());
                                app.set_timetable(timetable);
                                app.switch_to_main();
                            },
                            Err(e) => {
                                self.get_error = Some(e);
                            },
                        }
                    }
                    if self.get_error.is_some() {
                        ui.colored_label(Color32::RED, "Netinkamas kodas");
                    }
                });
        });
    }
}

impl MainApp {
    pub fn new(config_store: Box<dyn ConfigStore>, timetable_getter: Box<dyn TimetableGetter>) -> Self {
        Self {
            timetable: None,
            shown_week: get_current_week(),
            shown_events: vec![],
            assets: None,
            config_store,
            config: None,
            vidko_textfield: String::new(),
            timetable_getter,
            screen: None
        }
    }

    pub fn init(&mut self, cc: &CreationContext) {

        let texture_handle = cc.egui_ctx.load_texture("break-area", BREAK_IMAGE.clone(), TextureOptions::LINEAR);
        self.assets = Some(AppAssets {
            break_texture: texture_handle
        });

        self.config = match self.config_store.load() {
            Ok(config) => Some(config),
            Err(_) => None,
        };

        if self.vidko().is_none() {
            self.switch_to_vidko();
        } else {
            if self.refresh_timetable().is_err() {
                self.switch_to_vidko();
            } else {
                self.switch_to_main();
            }
        }
    }

    fn switch_to_main(&mut self) {
        self.screen = Some(Rc::new(RefCell::new(MainScreen::default())))
    }

    fn switch_to_vidko(&mut self) {
        self.screen = Some(Rc::new(RefCell::new(VidkoScreen::default())));
    }

    #[inline]
    pub fn vidko(&self) -> Option<&str> {
        if let Some(config) = &self.config {
            return config.vidko.as_deref();
        }
        None
    }

    pub fn refresh_timetable(&mut self) -> Result<(), GetTimetableError> {
        let vidko;
        {
            if self.vidko().is_none() {
                self.shown_events = vec![];
                self.timetable = None;
                return Err(GetTimetableError::NotFound);
            }
            vidko = self.vidko().unwrap();
        }

        let timetable = match self.timetable_getter.get(vidko) {
            Ok(timetable) => timetable,
            Err(e) => return Err(e),
        };

        self.shown_events = timetable.by_week(self.shown_week);
        self.timetable = Some(timetable);

        Ok(())
    }

    pub fn set_timetable(&mut self, timetable: Timetable) {
        self.shown_events = timetable.by_week(self.shown_week);
        self.timetable = Some(timetable);
    }

    fn shift_shown_week(&mut self, shift: i32) {
        let year = self.shown_week.year();
        let week = self.shown_week.week();
        let week_date = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).expect("Invalid week or year given");
        let shifted_week;
        if shift > 0 {
            shifted_week = week_date.checked_add_days(Days::new(7 * (shift as u64)));
        } else {
            shifted_week = week_date.checked_sub_days(Days::new(7 * ((-shift) as u64)));
        }

        if let Some(shifted_week) = shifted_week {
            self.set_shown_week(shifted_week.iso_week());
        }
    }

    fn set_shown_week(&mut self, week: IsoWeek) {
        self.shown_week = week;

        if let Some(timetable) = &self.timetable {
            self.shown_events = timetable.by_week(self.shown_week);
        }
    }
}

impl eframe::App for MainApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(config) = &self.config {
            self.config_store.save(config).unwrap();
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if let Some(screen) = self.screen.clone() {
            screen.borrow_mut().show(self, ctx)
        } else {
            // TODO: show error screen
            todo!()
        }
    }
}
