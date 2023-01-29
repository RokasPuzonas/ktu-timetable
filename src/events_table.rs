use std::ops::Add;

use eframe::{egui, epaint::text::TextWrapping};
use chrono::{Datelike, Timelike, NaiveDate, Weekday, IsoWeek, Duration, NaiveDateTime, Local};
use egui::{Color32, TextureHandle, Rect, text::LayoutJob, Visuals, Stroke, Widget};
use crate::timetable::{Event, EventCategory};

pub struct EventsTable<'a> {
    pub break_texture: Option<TextureHandle>,

    pub week: Option<IsoWeek>,
    pub now: Option<NaiveDateTime>,
    pub events: &'a [Event]
}

fn count_minutes(time: &str) -> u32 {
    let (time_h, time_m) = time.split_once(":").unwrap();
    return 60*time_h.parse::<u32>().unwrap() + time_m.parse::<u32>().unwrap();
}

fn get_category_bg(category: EventCategory) -> Color32 {
    match category {
        EventCategory::Default => Color32::GRAY,
        EventCategory::Yellow => Color32::from_rgb(251, 184, 41),
        EventCategory::Green => Color32::from_rgb(152, 188, 55),
        EventCategory::Red => Color32::from_rgb(247, 83, 65),
        EventCategory::Blue => Color32::from_rgb(10, 174, 179),
    }
}

fn draw_repeating_texture(ui: &mut egui::Ui, texture: Option<&TextureHandle>, rect: Rect, tint: Color32) {
    use egui::*;
    let painter = ui.painter();

    if texture.is_none() {
        painter.rect(rect, Rounding::none(), tint, (1.0, tint));
        return;
    }

    let texture = texture.unwrap();
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

const HEADER_SIZE: f32 = 50.0;

impl<'a> EventsTable<'a> {
    pub fn new(events: &'a [Event]) -> Self {
        Self {
            break_texture: None,
            week: None,
            now: None,
            events
        }
    }

    #[inline]
    fn fg_stroke(&self, visuals: &Visuals) -> Stroke {
        visuals.widgets.active.fg_stroke
    }

    #[inline]
    fn bg_fill(&self, visuals: &Visuals) -> Color32 {
        visuals.widgets.noninteractive.bg_fill
    }

    #[inline]
    fn highlight_color(&self, visuals: &Visuals) -> Color32 {
        visuals.selection.bg_fill
    }

    #[inline]
    fn dark_bg_fill(&self, visuals: &Visuals) -> Color32 {
        shift_color(self.bg_fill(visuals), 0.5)
    }

    fn show_event(&self, ui: &mut egui::Ui, event: &Event, mut rect: Rect) {
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

    fn show_header(&self, ui: &mut egui::Ui, rect: Rect, week: IsoWeek) {
        use egui::*;
        let painter = ui.painter();
        let visuals = ui.visuals();
        painter.rect_filled(rect, Rounding::none(), self.dark_bg_fill(visuals));

        let column_width = rect.width()/5.0;
        let header_size = rect.height();
        let text_size = egui::TextStyle::Body.resolve(ui.style()).size;
        let text_color = self.fg_stroke(visuals).color;

        // Draw day names
        for (i, name) in ["Pir", "Ant", "Tre", "Ket", "Pen"].iter().enumerate() {
            let offset = column_width * (i as f32 + 0.5);

            painter.text(
                rect.left_top() + vec2(offset, header_size/2.5),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(text_size*1.2),
                text_color
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
                text_color
            );
            week_date = week_date.add(Duration::days(1));
        }
    }

    fn show_body(
        &self,
        ui: &mut egui::Ui,
        rect: Rect,
        week: IsoWeek,
        now: NaiveDateTime
    ) {
        use egui::*;

        let painter = ui.painter();
        let column_width = rect.width()/5.0;
        let column_gap = 3.0;

        let visuals = ui.visuals();
        let bg_fill = self.bg_fill(visuals);
        let dark_bg_fill = self.dark_bg_fill(visuals);
        let highlight_color = self.highlight_color(visuals);

        let timestamps = ["9:00", "10:30", "11:00", "12:30", "13:30", "15:00", "15:30", "17:00"];
        let timestamps_mins = timestamps.map(count_minutes);
        let total_minutes = timestamps_mins.last().unwrap() - timestamps_mins.first().unwrap();
        let minute_to_pixel_scale = rect.height()/total_minutes as f32;

        // draw bg
        painter.rect_filled(
            rect,
            Rounding::none(),
            bg_fill
        );

        // Highlight current day column
        if now.iso_week() == week && !is_weekend(now.date()) {
            let days_from_monday = now.weekday().num_days_from_monday() as f32;
            let rect = Rect::from_min_max(
                rect.left_top() + vec2(column_width * days_from_monday, 0.0),
                rect.left_bottom() + vec2(column_width * (1.0+days_from_monday), 0.0)
            );
            painter.rect_filled(rect, Rounding::none(), highlight_color);
        }

        // Draw gaps between columns
        for i in 1..5 {
            let offset = column_width * i as f32;
            painter.line_segment([
                rect.left_top() + vec2(offset, 0.0),
                rect.left_bottom() + vec2(offset, 0.0)
            ], (column_gap, dark_bg_fill))
        }

        // Mark break times
        for i in (1..timestamps_mins.len()-1).step_by(2) {
            let from = (timestamps_mins[i]   - timestamps_mins[0]) as f32 * minute_to_pixel_scale;
            let to   = (timestamps_mins[i+1] - timestamps_mins[0]) as f32 * minute_to_pixel_scale;
            draw_repeating_texture(
                ui,
                self.break_texture.as_ref(),
                Rect::from_min_size(
                    rect.left_top() + vec2(0.0, from),
                    vec2(rect.width(), to - from)
                ),
                dark_bg_fill
            );
        }

        // Draw event cards
        for event in self.events {
            let day = event.date.weekday().num_days_from_monday() as usize;
            let duration = (event.end_time - event.start_time).num_minutes() as f32;
            let start_time = event.start_time.hour()*60 + event.start_time.minute() - timestamps_mins[0];
            let event_rect = Rect::from_min_size(
                rect.left_top() + vec2(column_width*day as f32, start_time as f32*minute_to_pixel_scale),
                vec2(column_width, duration*minute_to_pixel_scale)
            ).shrink2(vec2(10.0, 0.0));
            self.show_event(ui, event, event_rect);
        }

        // now line
        let painter = ui.painter();
        let current_time = now.minute() + now.hour() * 60;
        if current_time > timestamps_mins[0] && current_time < *timestamps_mins.last().unwrap_or(&0) && !is_weekend(now.date()) {
            let offset = (current_time - timestamps_mins[0]) as f32 * minute_to_pixel_scale;
            let points = [
                rect.left_top()  + vec2(0.0, offset),
                rect.right_top() + vec2(0.0, offset)
            ];
            let thickness = 2.0;
            let border_size = 2.0;
            painter.line_segment(points, (thickness + 2.0 * border_size, dark_bg_fill));
            painter.line_segment(points, (thickness, highlight_color));
        }
    }
}

impl<'a> Widget for EventsTable<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        use egui::*;

        let response = ui.allocate_rect(ui.min_rect(), Sense::hover());
        let rect = response.rect;

        let now  = self.now.unwrap_or(Local::now().naive_local());
        let week = self.week.unwrap_or(now.iso_week());

        self.show_header(
            ui,
            Rect::from_min_size(
                rect.left_top(),
                vec2(rect.width(), HEADER_SIZE)
            ),
            week
        );

        self.show_body(
            ui,
            Rect::from_min_max(
                rect.left_top() + vec2(0.0, HEADER_SIZE),
                rect.right_bottom()
            ),
            week,
            now
        );

        response
    }
}