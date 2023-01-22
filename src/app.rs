use std::ops::Add;

use eframe::{egui, CreationContext, epaint::text::TextWrapping};
use chrono::{Datelike, Timelike, Utc, NaiveDate, Weekday, IsoWeek, Duration, DateTime, NaiveDateTime, Days, Local};
use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Rect, text::LayoutJob, Visuals, Stroke};
use crate::{timetable::{Timetable, get_timetable, Event, EventCategory}, config::ConfigStorage};

struct EventsTableStyle {
    highlight_color: Color32,
    bg_fill: Color32,
    fg_stroke: egui::Stroke
}

/*
let highlight_color = style.visuals.selection.bg_fill;
let bg_fill = style.visuals.widgets.noninteractive.bg_fill; //Color32::from_rgb(18, 18, 18);
let fg_stroke = style.visuals.widgets.active.fg_stroke;

let light_bg_col = highlight_color; // shift_color(bg_col, 1.5); // Color32::from_rgb(30, 30, 30);
let dark_bg_col = shift_color(bg_fill, 0.5);  // style.visuals.widgets.noninteractive.bg_stroke.color; Color32::from_rgb(11, 8, 8);
//let fg_col = Color32::from_rgb(252, 232, 195);
let now_line_col = shift_color(highlight_color, 1.5); // fg_stroke.color; // shift_color(highlight_color, 1.5); // Color32::from_rgb(44, 120, 191);
*/
impl EventsTableStyle {
    fn from_visuals(visuals: &Visuals) -> Self {
        let bg_fill = visuals.widgets.noninteractive.bg_fill;
        Self {
            highlight_color: visuals.selection.bg_fill,
            bg_fill,
            fg_stroke: visuals.widgets.active.fg_stroke,
        }
    }

    #[inline]
    fn dark_bg_fill(&self) -> Color32 {
        shift_color(self.bg_fill, 0.5)
    }

    #[inline]
    fn now_line_fill(&self) -> Color32 {
        shift_color(self.highlight_color, 1.5)
    }
}

struct AppAssets {
    break_texture: egui::TextureHandle
}

pub struct MainApp {
    pub vidko: Option<String>,
    pub timetable: Option<Timetable>,
    shown_week: IsoWeek,
    shown_events: Vec<Event>,

    assets: Option<AppAssets>,
    storage: ConfigStorage
}

fn count_minutes(time: &str) -> u32 {
    let (time_h, time_m) = time.split_once(":").unwrap();
    return 60*time_h.parse::<u32>().unwrap() + time_m.parse::<u32>().unwrap();
}

fn get_category_bg(category: EventCategory) -> Color32 {
    match category {
        EventCategory::Default => Color32::GRAY,
        EventCategory::Yellow => Color32::from_rgb(251, 184, 41),
    }
}

fn is_bright_color(color: Color32) -> bool {
    return color.r() + color.g() + color.b() > 128 * 3
}

fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

fn draw_repeating_texture(ui: &mut egui::Ui, texture: &TextureHandle, rect: Rect, tint: Color32) {
    use egui::*;
    let painter = ui.painter();

    let texture_size = texture.size_vec2();
    let horizontal_count = rect.width()/texture_size[0];
    let vertical_count = rect.height()/texture_size[1];

    let draw_tile = |ix: u32, iy: u32, scale: Vec2| {
        painter.image(
            texture.id(),
            Rect::from_min_size(
                rect.left_top() + texture_size * vec2(ix as f32, iy as f32),
                texture_size * scale
            ),
            Rect::from_min_max(pos2(0.0, 0.0), scale.to_pos2()),
            tint
        );
    };

    {
        let scale = vec2(1.0, 1.0);
        for ix in 0..horizontal_count.floor() as u32 {
            for iy in 0..vertical_count.floor() as u32 {
                draw_tile(ix, iy, scale);
            }
        }
    }

    // right edge
    {
        let scale = vec2(horizontal_count % 1.0, 1.0);
        let ix = horizontal_count.floor() as u32;
        for iy in 0..vertical_count.floor() as u32 {
            draw_tile(ix, iy, scale);
        }
    }

    // bottom edge
    {
        let scale = vec2(1.0, vertical_count % 1.0);
        let iy = vertical_count.floor() as u32;
        for ix in 0..horizontal_count.floor() as u32 {
            draw_tile(ix, iy, scale);
        }
    }

    // left bottom corner
    {
        let scale = vec2(horizontal_count % 1.0, vertical_count % 1.0);
        draw_tile(horizontal_count.floor() as u32, vertical_count.floor() as u32, scale);
    }
}

fn show_event_card(ui: &mut egui::Ui, event: &Event, mut rect: Rect) {
    use egui::*;
    let margin = 6.0;
    let border_size = 4.0;
    let text_size = egui::TextStyle::Body.resolve(ui.style()).size;
    let text_color = Color32::BLACK;

    rect.set_width(rect.width().max(text_size*6.0));

    let painter = ui.painter();
    let color = get_category_bg(event.category);
    let rounding = Rounding::from(5.0);
    let border_color = color.linear_multiply(1.25);
    painter.rect_filled(rect, rounding, color);
    painter.rect_stroke(rect.shrink(border_size/2.0), rounding, (border_size, border_color));

    ui.allocate_ui_at_rect(rect.shrink(margin), |ui| {
        let font = FontId::proportional(text_size * 0.8);
        let summary_format = TextFormat {
            color: text_color,
            font_id: font.clone(),
            ..TextFormat::default()
        };
        let module_name = event.module_name.as_ref().unwrap_or(&event.summary);
        let mut job = LayoutJob::single_section(module_name.to_string(), summary_format);
        job.wrap = TextWrapping {
            max_rows: 2,
            ..Default::default()
        };
        ui.label(job);

        ui.add_space(text_size*0.2);
        let time_label = format!("{} - {}", event.start_time.format("%H:%M"), event.end_time.format("%H:%M"));
        ui.label(RichText::new(time_label).color(text_color).font(font));
    });
}

#[inline]
fn is_weekend(time: NaiveDate) -> bool {
    let day = time.weekday();
    return day == Weekday::Sat || day == Weekday::Sun;
}

#[inline]
fn shift_color(color: Color32, amount: f32) -> Color32 {
    return Color32::from_rgb(
        (color.r() as f32 * amount) as u8,
        (color.g() as f32 * amount) as u8,
        (color.b() as f32 * amount) as u8
    );
}

fn get_current_week() -> IsoWeek {
    let now = Local::now();
    if is_weekend(now.date_naive()) {
        now.add(Duration::days(7)).iso_week()
    } else {
        now.iso_week()
    }
}

#[inline]
fn show_events_table_header(
    ui: &mut egui::Ui,
    style: &EventsTableStyle,
    rect: Rect,
    week: IsoWeek
) {
    use egui::*;
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::none(), style.dark_bg_fill());

    let column_width = rect.width()/5.0;
    let header_size = rect.height();
    let text_size = egui::TextStyle::Body.resolve(ui.style()).size;

    // Draw day names
    for (i, name) in ["Pir", "Ant", "Tre", "Ket", "Pen"].iter().enumerate() {
        let offset = column_width * (i as f32 + 0.5);

        painter.text(
            rect.left_top() + vec2(offset, header_size/2.5),
            Align2::CENTER_CENTER,
            name,
            FontId::monospace(text_size*1.2),
            style.fg_stroke.color
        );
    }

    // Draw dates
    let year = week.year();
    let week = week.week();
    let mut week_date = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).expect("Invalid week or year given");
    for i in 1..=5 {
        let offset = column_width * i as f32;

        painter.text(
            rect.left_top() + vec2(offset-3.0, header_size-3.0),
            Align2::RIGHT_BOTTOM,
            week_date.format("%m-%d").to_string(),
            FontId::proportional(text_size*0.85),
            style.fg_stroke.color
        );
        week_date = week_date.add(Duration::days(1));
    }
}

#[inline]
fn show_events_table_body(
    ui: &mut egui::Ui,
    style: &EventsTableStyle,
    assets: &AppAssets,
    rect: Rect,
    week: IsoWeek,
    now: NaiveDateTime,
    events: &[Event]
) {
    use egui::*;

    let painter = ui.painter();
    let column_width = rect.width()/5.0;
    let column_gap = 3.0;

    let timestamps = ["9:00", "10:30", "11:00", "12:30", "13:30", "15:00", "15:30", "17:00"];
    let timestamps_mins = timestamps.map(count_minutes);
    let total_minutes = timestamps_mins.last().unwrap() - timestamps_mins.first().unwrap();
    let minute_to_pixel_scale = rect.height()/total_minutes as f32;

    // draw bg
    painter.rect_filled(
        rect,
        Rounding::none(),
        style.bg_fill
    );

    // Highlight current day column
    if now.iso_week() == week && !is_weekend(now.date()) {
        let days_from_monday = now.weekday().num_days_from_monday() as f32;
        let rect = Rect::from_min_max(
            rect.left_top() + vec2(column_width * days_from_monday, 0.0),
            rect.left_bottom() + vec2(column_width * (1.0+days_from_monday), 0.0)
        );
        painter.rect_filled(rect, Rounding::none(), style.highlight_color);
    }

    // Draw gaps between columns
    for i in 1..5 {
        let offset = column_width * i as f32;
        painter.line_segment([
            rect.left_top() + vec2(offset, 0.0),
            rect.left_bottom() + vec2(offset, 0.0)
        ], (column_gap, style.dark_bg_fill()))
    }

    // Mark break times
    for i in (1..timestamps_mins.len()-1).step_by(2) {
        let from = (timestamps_mins[i]   - timestamps_mins[0]) as f32 * minute_to_pixel_scale;
        let to   = (timestamps_mins[i+1] - timestamps_mins[0]) as f32 * minute_to_pixel_scale;
        draw_repeating_texture(
            ui,
            &assets.break_texture,
            Rect::from_min_size(
                rect.left_top() + vec2(0.0, from),
                vec2(rect.width(), to - from)
            ),
            style.dark_bg_fill()
        );
    }

    // Draw event cards
    for event in events {
        let day = event.date.weekday().num_days_from_monday() as usize;
        let duration = (event.end_time - event.start_time).num_minutes() as f32;
        let start_time = event.start_time.hour()*60 + event.start_time.minute() - timestamps_mins[0];
        let event_rect = Rect::from_min_size(
            rect.left_top() + vec2(column_width*day as f32, start_time as f32*minute_to_pixel_scale),
            vec2(column_width, duration*minute_to_pixel_scale)
        ).shrink2(vec2(10.0, 0.0));
        show_event_card(ui, event, event_rect);
    }

    // now line
    let painter = ui.painter();
    let current_time = now.minute() + now.hour() * 60 - timestamps_mins[0];
    if current_time > 0 && current_time < *timestamps_mins.last().unwrap_or(&0) && !is_weekend(now.date()) {
        let offset = current_time as f32 * minute_to_pixel_scale;
        let points = [
            rect.left_top()  + vec2(0.0, offset),
            rect.right_top() + vec2(0.0, offset)
        ];
        let thickness = 2.0;
        let border_size = 2.0;
        painter.line_segment(points, (thickness + 2.0 * border_size, style.dark_bg_fill()));
        painter.line_segment(points, (thickness, style.highlight_color));
    }
}

fn show_events_table(
    assets: &AppAssets,
    ui: &mut egui::Ui,
    rect: Rect,
    shown_week: IsoWeek,
    now: NaiveDateTime,
    events: &[Event]
) {
    use egui::*;

    let header_size = 50.0;

    //let now = now.checked_add_days(Days::new(1)).unwrap();
    let style = EventsTableStyle::from_visuals(&ui.style().visuals);

    show_events_table_header(
        ui, &style,
        Rect::from_min_size(
            rect.left_top(),
            vec2(rect.width(), header_size)
        ),
        shown_week
    );

    show_events_table_body(
        ui, &style, assets,
        Rect::from_min_max(
            rect.left_top() + vec2(0.0, header_size),
            rect.right_bottom()
        ),
        shown_week,
        now,
        events
    );

}

impl MainApp {
    pub fn new(storage: ConfigStorage) -> MainApp {
        MainApp {
            vidko: None,
            timetable: None,
            shown_week: get_current_week(),
            shown_events: vec![],
            assets: None,
            storage
        }
    }

    pub fn on_creation(&mut self, cc: &CreationContext) {
        self.storage.attempt_load();
        self.vidko = self.storage.config.vidko_code.clone();

        let break_image = load_image_from_memory(include_bytes!("../assets/break-area.png")).expect("Failed to decode break area texture");
        let texture_handle = cc.egui_ctx.load_texture("break-area", break_image, TextureOptions::LINEAR);
        self.assets = Some(AppAssets {
            break_texture: texture_handle
        });
    }

    pub fn refresh_timetable(&mut self) {
        if self.vidko.is_none() {
            self.shown_events = vec![];
            self.timetable = None;
            return;
        }

        let timetable = get_timetable(self.vidko.as_ref().unwrap());
        if timetable.is_err() { return; }
        let timetable = timetable.unwrap();
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
        self.storage.attempt_save();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use egui::*;

        //ctx.set_visuals(egui::Visuals::light());
        egui::CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
            if ctx.input().key_pressed(Key::D) {
                self.shift_shown_week(1);
            }
            if ctx.input().key_pressed(Key::A) {
                self.shift_shown_week(-1);
            }
            if ctx.input().key_pressed(Key::S) {
                self.set_shown_week(get_current_week());
            }
            if ctx.input().key_pressed(Key::F2) {
                if ctx.style().visuals.dark_mode {
                    ctx.set_visuals(egui::Visuals::light());
                } else {
                    ctx.set_visuals(egui::Visuals::dark());
                }
            }

            let rect = ui.allocate_rect(ui.min_rect(), Sense::hover()).rect;
            show_events_table(
                self.assets.as_ref().unwrap(),
                ui,
                rect,
                self.shown_week,
                Local::now().naive_local(),
                &self.shown_events
            );
        });
    }
}
