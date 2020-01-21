static KEY: &str = include_str!("../apikey");
use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use serde::Deserialize;


static mut MULTI: Option<MultiProgress> = None;
static mut TOP_BAR: Option<ProgressBar> = None;
static mut MIDDLE_BAR: Option<ProgressBar> = None;
static mut BOTTOM_BAR: Option<ProgressBar> = None;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.meh.com/1/current.json?apikey={}", KEY);
    let mut sleep_time = chrono::Duration::zero();
    let mut last_id: String = String::new();
    let args = clap_args();
    let alert_path = if let Some(path) = args.value_of("alert") {
        Some(path.to_string())
    } else {
        None
    };
    let progress = if args.is_present("progress") {
        setup_bars();
        true
    } else {
        false
    };
    let thread = std::thread::spawn(move || {
        if progress {
            update_bottom_bar("Warming up!");
            start_bar_tick();
        }
        loop {
            if sleep_time <= chrono::Duration::zero() {
                if progress {
                    update_bottom_bar("Requesting info from Meh.com");
                }
                sleep_time = if let Some(res) = get_response(&url) {
                    if res.deal.id != last_id {
                        let (line1, line2) = format_deal(&res.deal);
                        if let Some(ref p) = alert_path {
                            alert(&line1, &line2, p);
                        }
                        if progress {
                            update_top_bar(&line1);
                            update_middle_bar(&line2);
                        }
                        last_id = res.deal.id;
                        if let Some(end_date) = res.deal.end_date {
                            end_date - Utc::now()
                        } else {
                            duration_until_midnight_eastern()
                        }
                    } else {
                        chrono::Duration::minutes(1)
                    }
                } else {
                    chrono::Duration::minutes(1)
                };
            }
            sleep(sleep_time);
            sleep_time = chrono::Duration::zero();
        }
    });
    if progress {
        join_bars();
    } else {
        thread.join().unwrap();
    }
    Ok(())
}

fn clap_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("meh")
        .about("A cli tool for watching meh.com deals")
        .args(&[
            clap::Arg::with_name("alert")
                .long("alert")
                .short("a")
                .takes_value(true)
                .required(false)
                .number_of_values(1)
                .help("path to file that should be executed when the deal changes")
                .long_help("On windows this will be executed through powershell, on unix like it will executed directly")
                .value_name("FILE"),
            clap::Arg::with_name("progress")
                .long("progress")
                .short("p")
                .takes_value(false)
                .required(false)
                .help("if the cli should constantly report the current deal and when the next check will happen")
                .long_help("This is useful for active monitoring, if not passed you may want to pass the alert otherwise the application will not have any way to communicate"),
        ])
        .get_matches()
}

fn setup_bars() {
    use std::sync::Once;
    static SETUP: Once = Once::new();
    SETUP.call_once(|| {
        let style = ProgressStyle::default_spinner()
            .tick_chars("█▓▒░  ░▒▓█")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}");
        let bottom = ProgressBar::new_spinner().with_style(style.clone());
        let middle = ProgressBar::new_spinner().with_style(style.clone());
        let top = ProgressBar::new_spinner().with_style(style);
        let multi = MultiProgress::new();
        unsafe {
            TOP_BAR = Some(multi.add(top));
            MIDDLE_BAR = Some(multi.add(middle));
            BOTTOM_BAR = Some(multi.add(bottom));
            MULTI = Some(multi);
        }
    });
}

fn start_bar_tick() {
    let tick = 200;
    unsafe {
        if let Some(ref b) = BOTTOM_BAR {
            b.enable_steady_tick(tick)
        }
        if let Some(ref b) = MIDDLE_BAR {
            b.enable_steady_tick(tick)
        }
        if let Some(ref b) = TOP_BAR {
            b.enable_steady_tick(tick)
        }
    }
}

fn update_bottom_bar(msg: &str) {
    unsafe {
        update_bar(msg, &BOTTOM_BAR)
    }
}

fn update_middle_bar(msg: &str) {
    unsafe {
        update_bar(msg, &MIDDLE_BAR)
    }
}
fn update_top_bar(msg: &str) {
    unsafe {
        update_bar(msg, &TOP_BAR)
    }
}

fn update_bar(msg: &str, bar: &Option<ProgressBar>){
    if let Some(ref b) = bar {
        b.set_message(msg)
    }
}

fn join_bars() {
    unsafe {
        if let Some(ref b) = MULTI {
            b.join().unwrap();
        }
    }
}

fn duration_until_midnight_eastern() -> chrono::Duration {
    use chrono::{TimeZone, Datelike};
    use std::ops::Add;
    let hour = 3600;
    
    let offset = chrono::FixedOffset::west(hour * 5);
    let now = Utc::now().with_timezone(&offset);
    let tomorrow = now.add(chrono::Duration::days(1));
    let midnight_est = offset
        .ymd(tomorrow.year(), tomorrow.month(), tomorrow.day())
        .and_hms(0, 0, 0);
    midnight_est.signed_duration_since(now)
}

fn get_response(url: &str) -> Option<Response> {
    let res_text = get_text(url)
        .map_err(|e| {
            eprintln!("Error getting response text {}", e);
        })
        .ok()?;
    match serde_json::from_str(&res_text) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("Error deserializng: {}", e);
            eprintln!("{}", res_text);
            None
        }
    }
}

fn get_text(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let ret = reqwest::blocking::get(url)?.text()?;
    Ok(ret)
}

fn sleep(duration: chrono::Duration) {
    use std::ops::Add;
    let end = Utc::now().add(duration);
    let one_hour: chrono::Duration = chrono::Duration::hours(1);
    let one_minute: chrono::Duration = chrono::Duration::minutes(1);
    let five_minutes = chrono::Duration::minutes(5);
    loop {
        let diff = end.signed_duration_since(Utc::now());
        let (when, how_long) = if diff > one_hour {
            let t = format_time(end);
            (format!("{} hours at {}", diff.num_hours(), t), one_hour)
        } else if diff > one_minute {
            let t = format_time(end);
            let sleep_for = if diff > five_minutes {
                five_minutes
            } else {
                one_minute
            };
            (
                format!("{} minutes at {}", diff.num_minutes(), t),
                sleep_for,
            )
        } else if diff > chrono::Duration::zero() {
            let t = format_time(end);
            (format!("{} s at {}", diff.num_seconds(), t), diff)
        } else {
            break;
        };
        update_bottom_bar(&format!("checking again in {}", when));
        std::thread::sleep(how_long.to_std().unwrap());
    }
}

fn format_deal(d: &Deal) -> (String, String) {
    let costs: Vec<String> = d.items.iter().map(|i| format!("{:.2}", i.price)).collect();
    let costs = costs.join(", ");
    (format!("{} ({})", d.title, costs), format!("{}", d.url))
}

fn format_time(date: DateTime<Utc>) -> String {
    let local_time: chrono::DateTime<chrono::Local> = date.into();
    format!("{}", local_time.format("%-l:%M %p"))
}
#[cfg(windows)]
fn alert(line_one: &str, line_two: &str, path: &str) {
    let local_msg = format!(r#""{}
{}""#, line_one, line_two);
    let local_path = path.to_string();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("powershell")
            .arg(&local_path)
            .arg(&local_msg)
            .output();
    });
}
#[cfg(unix)]
fn alert(line_one: &str, line_two: &str, path: &str) {
    let local_msg = format!(r#""{}\\n{}""#, line_one, line_two);
    let local_path = path.to_string();
    std::thread::spawn(move || {
        let _ = std::process::Command::new(&local_path).arg(&local_msg)
        .output();
    });
}

#[derive(Debug, Deserialize)]
struct Response {
    deal: Deal,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Deal {
    features: String,
    id: String,
    items: Vec<Item>,
    #[serde(default)]
    launches: Vec<Launch>,
    photos: Vec<String>,
    purchase_quantity: PurchaseQuantity,
    title: String,
    specifications: String,
    story: Story,
    theme: Theme,
    url: String,
    end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct Item {
    attributes: Vec<Attribute>,
    condition: String,
    id: String,
    price: f32,
    photo: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Launch {
    sold_out_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PurchaseQuantity {
    maximum_limit: u32,
    minimum_limit: u32,
}
#[derive(Debug, Deserialize)]
struct Story {
    title: String,
    body: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Theme {
    accent_color: String,
    background_color: String,
    background_image: String,
    foreground: String,
}

#[derive(Debug, Deserialize)]
struct Attribute {
    key: String,
    value: String,
}
