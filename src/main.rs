static KEY: &str = include_str!("../apikey");
use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
#[cfg(windows)]
static CMD: &str = "./alert.ps1";
#[cfg(unix)]
static CMD: &str = "./alert.sh";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.meh.com/1/current.json?apikey={}", KEY);
    let mut sleep_time = chrono::Duration::zero();
    let mut last_id: String = String::new();
    let style = ProgressStyle::default_spinner()
        .tick_chars("█▓▒░ ░▒▓█")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    let bar = ProgressBar::new_spinner();
    bar.set_style(style.clone());
    let bar2 = ProgressBar::new_spinner();
    bar2.set_style(style.clone());
    let bar3 = ProgressBar::new_spinner();
    bar3.set_style(style.clone());
    let multi = indicatif::MultiProgress::new();
    let bar3 = multi.add(bar3);
    let bar2 = multi.add(bar2);
    let mut bar = multi.add(bar);
    let should_alert = std::path::Path::new(CMD).exists();
    std::thread::spawn(move || {
        bar.set_message("Warming up!");
        bar.enable_steady_tick(200);
        bar2.enable_steady_tick(200);
        bar3.enable_steady_tick(200);
        loop {
            if sleep_time <= chrono::Duration::zero() {
                bar.set_message("Requesting info from Meh.com");
                sleep_time = if let Some(res) = get_response(&url, &mut bar) {
                    if res.deal.id != last_id {
                        let (line1, line2) = format_deal(&res.deal);
                        if should_alert {
                            alert(&format!("{}\n{}", line1, line2));
                        }
                        bar3.set_message(&line1);
                        bar2.set_message(&line2);
                        last_id = res.deal.id;
                        res.deal.end_date - Utc::now()
                    } else {
                        chrono::Duration::minutes(1)
                    }
                } else {
                    chrono::Duration::minutes(1)
                };
            }
            sleep(sleep_time, &mut bar);
            sleep_time = chrono::Duration::zero();
        }
    });
    multi.join()?;
    Ok(())
}

fn get_response(url: &str, bar: &mut ProgressBar) -> Option<Response> {
    let res_text = get_text(url)
        .map_err(|e| {
            eprintln!("Error getting response text {}", e);
        })
        .ok()?;
    match serde_json::from_str(&res_text) {
        Ok(r) => Some(r),
        Err(e) => {
            bar.println(&format!("Error deserializng: {}", e));
            eprintln!("{}", res_text);
            None
        }
    }
}

fn get_text(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let ret = reqwest::blocking::get(url)?.text()?;
    Ok(ret)
}

fn sleep(duration: chrono::Duration, bar: &mut ProgressBar) {
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
        bar.set_message(&format!("checking again in {}", when));
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
fn alert(msg: &str) {
    let local_msg = format!("{:?}", msg);
    std::thread::spawn(move || {
        let _ = std::process::Command::new("powershell")
            .arg(CMD)
            .arg(&local_msg);
    });
}
#[cfg(unix)]
fn alert(msg: &str) {
    let local_msg = format!("{:?}", msg);
    std::thread::spawn(move || {
        let _ = std::process::Command::new(CMD).arg(&local_msg);
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
    end_date: DateTime<Utc>,
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
    sold_out_at: String,
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
