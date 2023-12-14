slint::include_modules!();
use slint::VecModel;
use slint::{Timer, TimerMode};
use std::rc::Rc;

use chrono::prelude::*;
use chrono::{Days, Months, Duration, Datelike, Local};

use rusqlite::Connection;
use std::env;
use std::collections::HashMap;
use clap::Parser;
use rand::{thread_rng, Rng};

use slint::Color;

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref COLORS: Mutex<Vec<Color>> = Mutex::new(vec![Color::from_rgb_u8(255, 0, 0), Color::from_rgb_u8(0, 255, 0), Color::from_rgb_u8(0, 0, 255), Color::from_rgb_u8(255, 255, 0), Color::from_rgb_u8(255, 0, 255), Color::from_rgb_u8(0, 255, 255), Color::from_rgb_u8(255, 255, 255), Color::from_rgb_u8(0, 0, 0), Color::from_rgb_u8(85, 85, 85), Color::from_rgb_u8(170, 170, 170), Color::from_rgb_u8(128, 255, 0), Color::from_rgb_u8(128, 0, 255), Color::from_rgb_u8(255, 128, 0)]);
}



#[derive(Debug)]
struct Res {
    class: String,
    count: i64,
}

#[derive(Debug)]
struct Row {
    class: String,
    ts: i64,
}




#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "The database to connect to, defaults to 'tracking.db' can also be set with TRACKING_DB environment variable")]
    database: Option<String>,

}
/*
fn min(a: usize, b: usize) -> usize {
    if a < b {
        return a;
    }
    b
}
*/

fn fmt(ts: Duration) -> String {
    format!("{}:{:0>2}:{:0>2} ", ts.num_hours(), ts.num_minutes() % 60, ts.num_seconds() % 60)
}

fn datetime_to_frac_of_day<T: TimeZone>(dt: DateTime<T>) -> f32 {
    dt.hour() as f32 / 24 as f32
    + dt.minute() as f32 / (24 * 60) as f32
    + dt.second() as f32 / (24 * 60 * 60) as f32
}

fn main() -> Result<(), slint::PlatformError> {

    let mut db = if let Ok(s) = env::var("TRACKING_DB") {
        s
    } else {
        "tracking.db".to_string()
    };

    let args = Args::parse();
    if args.database.is_some() {
        db = args.database.unwrap();
    }


    let ui = AppWindow::new()?;

    let ui_handle = ui.as_weak();
    let db_ = db.clone();
    ui.on_month_overview_change_month(move |months| {
        let ui = ui_handle.unwrap();
        let day = DateTime::from_timestamp(ui.get_current_timestamp() as i64, 0).unwrap();
        if months < 0 {
            set_date(&ui, day.checked_sub_months(Months::new(-months as u32)).unwrap().into(), Local::now(), &db_);
        } else {
            set_date(&ui, day.checked_add_months(Months::new(months as u32)).unwrap().into(), Local::now(), &db_);
        }
    });

    let ui_handle = ui.as_weak();
    let db_ = db.clone();

    ui.global::<State>().on_select(move |timestamp| {
        let ui = ui_handle.unwrap();
        let now: DateTime<Local> = Local::now();
        ui.global::<State>().set_current_timestamp(timestamp);
        set_date(&ui, DateTime::from_timestamp(timestamp as i64, 0).unwrap().into(), now, &db_);
    });

    let ui_handle = ui.as_weak();
    let db_ = db.clone();

    ui.on_month_overview_select_today(move || {
        let ui = ui_handle.unwrap();
        let now: DateTime<Local> = Local::now();
        set_date(&ui, now, now, &db_);
    });

    let now: DateTime<Local> = Local::now();
    ui.global::<State>().set_now_as_frac(datetime_to_frac_of_day(now));
    set_date(&ui, now, now, &db);
    let ui_handle = ui.as_weak();
    let timer = Timer::default();
    {
    timer.start(TimerMode::Repeated, std::time::Duration::from_secs(10), move || {
        let ui = ui_handle.unwrap();
        let now: DateTime<Local> = Local::now();
        ui.global::<State>().set_now_as_frac(datetime_to_frac_of_day(now));
        if ui.global::<State>().get_auto_update() {
            set_date(&ui, DateTime::from_timestamp(ui.global::<State>().get_current_timestamp() as i64, 0).unwrap().into(), now, &db);
        }
    });
    }
    ui.run()
}

fn next_day<T: TimeZone>(d: DateTime<T>) -> DateTime<T> {
    //let s = d.with_hour(0).unwrap().offset().fix().utc_minus_local() as i64;
    d.checked_add_signed(Duration::hours(26)).unwrap()
    .with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
    //.checked_add_signed(Duration::seconds(s)).unwrap()
}

fn prev_day<T: TimeZone>(d: DateTime<T>) -> DateTime<T> {
    //let s = d.with_hour(0).unwrap().offset().fix().utc_minus_local() as i64;
    d.checked_sub_signed(Duration::hours(22)).unwrap()
    .with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
    //.checked_add_signed(Duration::seconds(s)).unwrap()
}

fn next_week<T: TimeZone>(d: DateTime<T>) -> DateTime<T> {
    //let s = d.with_hour(0).unwrap().offset().fix().utc_minus_local() as i64;
    d.checked_add_signed(Duration::hours(6 * 24 + 26)).unwrap()
    .with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
    //.checked_add_signed(Duration::seconds(s)).unwrap()
}

fn prev_week<T: TimeZone>(d: DateTime<T>) -> DateTime<T> {
    //let s = d.with_hour(0).unwrap().offset().fix().utc_minus_local() as i64;
    d.checked_sub_signed(Duration::hours(6 * 24 + 22)).unwrap()
    .with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()
    //.checked_add_signed(Duration::seconds(s)).unwrap()
}

fn update_data<T: TimeZone>(ui: &AppWindow, db: &String, now: DateTime<T>) {
    let selection_start_ts = ui.global::<State>().get_selection_start() as u32;
    let selection_end_ts = ui.global::<State>().get_selection_end() as u32;
    let week = ui.global::<State>().get_week() as u32;
    let selection_start: DateTime<Local> = DateTime::from_timestamp(selection_start_ts as i64, 0).unwrap().into();
    let selection_end: DateTime<Local> = DateTime::from_timestamp(selection_end_ts as i64, 0).unwrap().into();
    let mut legend_data = Vec::new();
    let mut daylight_transition = false;
    let conn = Connection::open(db).unwrap();

    let query = &format!("select class, count (*) FROM tracking where ts > {} and ts < {} group by class order by count (*) desc;", selection_start_ts, selection_end_ts);
    let mut stmt = conn.prepare(query).unwrap();

    let res = stmt.query_map((), |row| {
        Ok(Res {
            class: row.get(0)?,
            count: row.get(1)?,
        })
    }).unwrap();
    let mut classes = Vec::new();
    for r in res {
        let r = r.unwrap();
        classes.push((r.class, Duration::seconds(10 * r.count)));
    }
    let mut color_map = HashMap::new();
    while COLORS.lock().unwrap().len() < classes.len() {
        let mut rng = thread_rng();
        let gray = rng.gen_range(10..245);
        COLORS.lock().unwrap().push(Color::from_rgb_u8(gray, gray, gray));
    }
    let mut i = 0;
    for c in classes {
        color_map.insert(c.0.clone(), COLORS.lock().unwrap()[i]);
        legend_data.push(LegendData{class: c.0.into(), time: fmt(c.1).into(), color: COLORS.lock().unwrap()[i]});
        i += 1;
    }

    let mut current_start = selection_start.clone();
    let mut current_end = current_start.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap().checked_sub_signed(Duration::seconds(1)).unwrap();
    let mut week_day_data = Vec::new();

    let mut week_start = selection_start.clone();
    while week_start.weekday() != Weekday::Mon {
        week_start = week_start.checked_sub_signed(Duration::hours(22)).unwrap().with_hour(0).unwrap();
    }
    while week_start.iso_week().week() != week {
        week_start = week_start.checked_add_signed(Duration::hours(24 * 6 + 26)).unwrap().with_hour(0).unwrap();
    }
    let mut week_end = week_start.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap();
    while week_end.weekday() != Weekday::Mon {
        week_end = week_end.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap().checked_sub_signed(Duration::seconds(1)).unwrap();
    }
    if week_start < selection_start {
        let mut pad_day = week_start.clone();
        while pad_day < selection_start {
            week_day_data.push(WeekDayData{
                day_of_month: pad_day.day() as i32,
                day_of_week_str: pad_day.weekday().to_string().into(),
                tasks: Rc::new(VecModel::from(Vec::new())).into(),
                today: pad_day.date_naive().eq(&now.date_naive()),
            });
            pad_day = pad_day.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap();
        }
    }

    while current_start < selection_end {
        let query = &format!("select class, ts FROM tracking where ts > {} and ts < {} order by ts asc;", current_start.timestamp(), current_end.timestamp());
        let mut stmt = conn.prepare(query).unwrap();

        let row = stmt.query_map((), |row| {
            Ok(Row {
                class: row.get(0)?,
                ts: row.get(1)?,
            })
        }).unwrap();

        if week_start <= current_start && current_start < week_end {
            let mut tasks = Vec::new();
            let mut prev_row = Row{class: "".into(), ts: 0};
            let mut task_start = current_start.clone();
            for r in row {
                let r = r.unwrap();
                if prev_row.ts == 0 {
                    task_start = DateTime::from_timestamp(r.ts, 0).unwrap().into();
                    prev_row = r;
                    continue;
                }
                if r.class.eq(&prev_row.class) && r.ts - 11 <= prev_row.ts {
                    prev_row = r;
                    continue;
                }
                // new task found, finish up the old one
                let task_end: DateTime<Local> = DateTime::from_timestamp(r.ts, 0).unwrap().into();
                let task = Task {
                    class: prev_row.class.clone().into(),
                    title: "".into(),
                    color: color_map.get(&prev_row.class).unwrap().clone(),
                    start: datetime_to_frac_of_day(task_start),
                    end: datetime_to_frac_of_day(task_end),
                };
                tasks.push(task);
                task_start = task_end;
                prev_row = r;
            }

            // last task of the day
            if prev_row.class != "" {
                let task_end: DateTime<Local> = DateTime::from_timestamp(prev_row.ts + 10, 0).unwrap().into();
                let task = Task {
                    class: prev_row.class.clone().into(),
                    title: "".into(),
                    color: color_map.get(&prev_row.class).unwrap().clone(),
                    start: datetime_to_frac_of_day(task_start),
                    end: datetime_to_frac_of_day(task_end),
                };
                tasks.push(task);
            }
            let tasks = Rc::new(VecModel::from(tasks));

            week_day_data.push(WeekDayData{
                day_of_month: task_start.day() as i32,
                day_of_week_str: task_start.weekday().to_string().into(),
                tasks: tasks.into(),
                today: task_start.date_naive().eq(&now.date_naive())
            });
        }

        if let Some(t) = current_start.checked_add_days(Days::new(1)) {
            current_start = t;
            current_end = current_end.checked_add_days(Days::new(1)).unwrap();
            daylight_transition = false;
        } else {
            let current_offset = current_start.offset().local_minus_utc();
            let next_offset = current_start.checked_add_signed(Duration::hours(26)).unwrap().offset().local_minus_utc();
            if current_offset > next_offset {
                daylight_transition = true;
            }
            current_start = current_start.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap();
            current_end = current_end.checked_add_signed(Duration::hours(26)).unwrap().with_hour(0).unwrap();
        }

    }
    let week_day_data = Rc::new(VecModel::from(week_day_data));
    ui.set_week_day_data(week_day_data.into());
    let legend_data = Rc::new(VecModel::from(legend_data));
    ui.global::<State>().set_legend_data(legend_data.into());

}

fn set_date(ui: &AppWindow, day: DateTime<Local>, now: DateTime<Local>, db: &String) {

    // set month overview
    let mut overview_first_day = day.clone();
    while overview_first_day.day() > 7 {
        overview_first_day = prev_week(overview_first_day);
    }
    if day.weekday() == Weekday::Mon {
        overview_first_day = prev_week(overview_first_day);
    } else {
        while overview_first_day.weekday() != Weekday::Mon {
            overview_first_day = prev_day(overview_first_day);
        }
        if overview_first_day.day() < 7 {
            overview_first_day = prev_week(overview_first_day);
        }
    }

    let mut days = Vec::new();
    let mut cur_day = overview_first_day.clone();

    for _i in 0..42 {
        days.push(OverviewMonthDayData{
            day_of_month: cur_day.day() as i32,
            today: cur_day.date_naive().eq(&now.date_naive()),
            current_month: cur_day.month() == day.month(),
            selected: cur_day.date_naive().eq(&day.date_naive()),
            timestamp: cur_day.timestamp() as i32,
        });
        cur_day = cur_day.checked_add_days(Days::new(1)).unwrap();
    }

    let mut weeks = Vec::new();
    let mut cur_week = next_day(overview_first_day);

    for _i in 0..6 {
        weeks.push(cur_week.iso_week().week() as i32);
        cur_week = next_week(cur_week);
    }

    let days = Rc::new(VecModel::from(days));
    let weeks = Rc::new(VecModel::from(weeks));


    ui.set_month_overview_days(days.clone().into());
    ui.set_month_overview_weeks(weeks.clone().into());


    let selection_start = next_day(prev_day(day.with_day(1).unwrap()));
    let selection_end = selection_start.checked_add_months(Months::new(1)).unwrap().checked_sub_signed(Duration::seconds(1)).unwrap();

    ui.global::<State>().set_selection_start(selection_start.timestamp() as i32);
    ui.global::<State>().set_selection_end(selection_end.timestamp() as i32);
    ui.global::<State>().set_status(format!("Selected range: {} to {}", selection_start, selection_end).into());

    ui.global::<State>().set_now(now.timestamp() as i32);
    ui.global::<State>().set_current_timestamp(day.timestamp() as i32);
    ui.global::<State>().set_day_of_month(day.day() as i32);
    ui.global::<State>().set_month(day.month() as i32);
    ui.global::<State>().set_year(day.year());
    ui.global::<State>().set_month_str(chrono::Month::try_from(u8::try_from(day.month()).unwrap()).unwrap().name().into());
    ui.global::<State>().set_week(day.iso_week().week() as i32);
    update_data(&ui, db, now);
}
