fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "bin")]
    bin::mod_main()?;

    #[cfg(not(feature = "bin"))]
    lib::mod_main()?;

    Ok(())
}

#[cfg(not(feature = "bin"))]
mod lib {
    static ERROR: &str = "Please re-run with the flag --features bin";

    pub struct E;
    impl std::fmt::Display for E {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", ERROR)
        }
    }
    impl std::fmt::Debug for E {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", ERROR)
        }
    }
    impl std::error::Error for E {}
    pub fn mod_main() -> Result<(), E> {
        Err(E)
    }
}

#[cfg(feature = "bin")]
mod bin {

    static KEY: &str = include_str!("../apikey");
    use chrono::{DateTime, Datelike, Duration, FixedOffset, Local, TimeZone, Utc};
    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    use log::debug;

    use meh::{Deal, Response};
    use std::sync::{Arc, Mutex};

    type GlobalBar<T> = Arc<Mutex<Option<T>>>;
    lazy_static::lazy_static! {
        static ref MULTI: GlobalBar<MultiProgress> = Arc::new(Mutex::new(None));
        static ref TOP_BAR: GlobalBar<ProgressBar> = Arc::new(Mutex::new(None));
        static ref MIDDLE_BAR: GlobalBar<ProgressBar> = Arc::new(Mutex::new(None));
        static ref BOTTOM_BAR: GlobalBar<ProgressBar> = Arc::new(Mutex::new(None));
    }
    pub fn mod_main() -> Result<(), Box<dyn std::error::Error>> {
        pretty_env_logger::init();
        let url = meh::construct_url(KEY);
        let args = clap_args();
        if let Some(matches) = args.subcommand_matches("watch") {
            let alert_info = if let Some(alert) = matches.value_of("alert") {
                let ty = matches.value_of("alert-type").unwrap_or_else(|| "title-price");
                debug!("got alert args: {}, {}", alert, ty);
                Some((alert.to_string(), ty.to_string()))
            } else {
                debug!("no alert args");
                None
            };
            let progress = matches.is_present("progress");

            let interval = if let Some(interval) = matches.value_of("interval") {
                if let Ok(ms) = interval.parse() {
                    Some(Duration::milliseconds(ms))
                } else {
                    None
                }
            } else {
                None
            };
            watch(alert_info, progress, url, interval)?;
        } else {
            let json = get_text(&url)?;
            println!("{}", json);
        }
        Ok(())
    }
    fn watch(
        alert_args: Option<(String, String)>,
        progress: bool,
        url: String,
        interval: Option<Duration>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sleep_time = Duration::zero();
        let mut last_id: String = String::new();
        let alert_info = if let Some((path, kind)) = alert_args {
            let p = std::path::PathBuf::from(path);
            debug!("alert arg provided {} {}", p.display(), kind);
            if p.exists() {
                debug!("alert file exists");
                Some((p.as_path().to_owned(), kind.to_string()))
            } else {
                debug!("alert file does not exist");
                None
            }
        } else {
            None
        };
        if progress {
            setup_bars();
        }

        let thread = std::thread::spawn(move || {
            if progress {
                start_bar_tick();
                update_bottom_bar("Warming up!");
            }
            let default_duration = interval.unwrap_or_else(|| Duration::minutes(1));
            loop {
                if sleep_time <= Duration::zero() {
                    if progress {
                        update_bottom_bar("Requesting info from Meh.com");
                    }
                    debug!("requesting deal");
                    sleep_time = if let Some(res) = get_response(&url) {
                        if res.deal.id != last_id {
                            debug!("new deal!");
                            let (line1, line2) = format_deal(&res.deal);
                            if let Some((p, kind)) = &alert_info {
                                debug!("sending alert!");
                                let _ = alert(&res, p, kind);
                            }
                            if progress {
                                update_top_bar(&line1);
                                update_middle_bar(&line2);
                            }
                            last_id = res.deal.id;
                            if let Ok(Some(end)) = get_complicated_end() {
                                let now = chrono::Utc::now();                                
                                end.signed_duration_since(now) + Duration::milliseconds(500)
                            } else if let Some(interval) = interval {
                                interval
                            } else if let Some(end_date) = res.deal.end_date {
                                end_date - Utc::now()
                            } else {
                                duration_until_midnight_eastern()
                            }
                        } else {
                            default_duration
                        }
                    } else {
                        default_duration
                    };
                }
                sleep(sleep_time);
                sleep_time = Duration::zero();
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
        .subcommand(clap::App::new("watch")
        .args(&[
            clap::Arg::with_name("alert")
            .long("alert")
            .short("a")
            .takes_value(true)
            .required(false)
            .number_of_values(1)
            .help("Path to file that should be executed when the deal changes")
            .long_help("Path to file that should be executed when the deal changes \
                On windows this will be executed through powershell, on unix like it will executed directly")
            .value_name("FILE"),
            clap::Arg::with_name("progress")
            .long("progress")
            .short("p")
            .takes_value(false)
            .required(false)
            .help("If the cli should constantly report the current deal and when the next check will happen")
            .long_help("If the cli should constantly report the current deal and when the next check will happen \
                This is useful for active monitoring, if not passed you may want to pass the alert otherwise the application will not have any way to communicate"),
            clap::Arg::with_name("alert-type")
            .long("alert_type")
            .short("t")
            .takes_value(true)
            .required(false)
            .number_of_values(1)
            .help("Shape of the alert argument")
            .possible_values(&["title", "title-price", "json", "csv"])
            .default_value("title-price")
            .long_help("Shape of the alert argument. \
                title: single line string with title only \
                title-price: same as title but list of prices appended to the end \
                json: json blob of the full resonse body"),
            clap::Arg::with_name("interval")
            .long("interval")
            .short("i")
            .takes_value(true)
            .required(false)
            .number_of_values(1)
            .validator(|s| {
                s.parse::<i64>().map_err(|_| "interval must be a number")?;
                Ok(())
            })
            .help("How long to wait between checks")
            .long_help("How many ms to wait between checks. \
            If not provided, uses the expiration date of the response from meh")
            ]))
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
            let multi = MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::stdout());
            if let Ok(mut b) = TOP_BAR.lock() {
                *b = Some(multi.add(top));
            }
            if let Ok(mut b) = MIDDLE_BAR.lock() {
                *b = Some(multi.add(middle));
            }
            if let Ok(mut b) = BOTTOM_BAR.lock() {
                *b = Some(multi.add(bottom));
            }
            if let Ok(mut b) = MULTI.lock() {
                *b = Some(multi);
            }
        });
    }

    fn start_bar_tick() {
        let tick = 200;
        if let Ok(m) = BOTTOM_BAR.lock() {
            if let Some(b) = &*m {
                b.enable_steady_tick(tick)
            }
        }
        if let Ok(m) = MIDDLE_BAR.lock() {
            if let Some(b) = &*m {
                b.enable_steady_tick(tick)
            }
        }
        if let Ok(m) = TOP_BAR.lock() {
            if let Some(b) = &*m {
                b.enable_steady_tick(tick)
            }
        }
    }

    fn update_bottom_bar(msg: &str) {
        if let Ok(m) = BOTTOM_BAR.lock() {
            update_bar(msg, &m);
        }
    }

    fn update_middle_bar(msg: &str) {
        if let Ok(m) = MIDDLE_BAR.lock() {
            update_bar(msg, &m);
        }
    }
    fn update_top_bar(msg: &str) {
        if let Ok(m) = TOP_BAR.lock() {
            update_bar(msg, &m);
        }
    }

    fn update_bar(msg: &str, bar: &Option<ProgressBar>) {
        if let Some(ref b) = bar {
            b.set_message(msg)
        }
    }

    fn join_bars() {
        if let Ok(m) = MULTI.lock() {
            if let Some(ref b) = &*m {
                b.join().unwrap()
            }
        }
    }

    fn duration_until_midnight_eastern() -> Duration {
        use std::ops::Add;
        let hour = 3600;

        let offset = FixedOffset::west(hour * 5);
        let now = Utc::now().with_timezone(&offset);
        let tomorrow = now.add(Duration::days(1));
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
                let msg = format!("[\"Error deserializng: {}\",\n{}\n]", e, res_text);
                let filename = format!("{}.json", Local::now());
                std::fs::write(filename, msg).expect("failed to write bad json to file");
                None
            }
        }
    }

    fn get_complicated_end() -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
        log::trace!("get_complicated_end");
        let text = get_text("https://meh.com")?;
        static TIME_START: &str = "$countdown.countdown(new Date('";
        static TIME_END: &str = "+0000 (Greenwich Mean Time)'))";
        if let Some(idx) = text.find(TIME_START) {
            let rest = &text[idx + TIME_START.len()..];
            let end_idx = rest.find(TIME_END).unwrap_or(rest.len());
            let dt = &rest[..end_idx];
            let mut iter = dt.split(' ');
            iter.next();
            // Wed Jan 20 2021 19:54:00 GMT+0000 (Greenwich Mean Time)
            let month = match iter.next() {
                Some("Feb") => 2,
                Some("Mar") => 3,
                Some("Apr") => 4,
                Some("May") => 5,
                Some("Jun") => 6,
                Some("Jul") => 7,
                Some("Aug") => 8,
                Some("Sep") => 9,
                Some("Oct") => 10,
                Some("Nov") => 11,
                Some("Dec") => 12,
                _ => 1,
            };
            let day = match iter.next() {
                Some(s) => s.parse().unwrap_or(0u32),
                _ => 0,
            };
            let year = match iter.next() {
                Some(s) => s.parse().unwrap_or(0i32),
                _ => 0
            };
            let (h,m,s) = match iter.next() {
                Some(s) => {
                    let mut iter = s.split(":");
                    let h = iter.next().map(|s| s.parse().unwrap_or(0u32)).unwrap_or(0u32);
                    let m = iter.next().map(|s| s.parse().unwrap_or(0u32)).unwrap_or(0u32);
                    let s = iter.next().map(|s| s.parse().unwrap_or(0u32)).unwrap_or(0u32);
                    (h,m,s)
                }
                _ => (0,0,0)
            };
            Ok(Some(Utc.ymd(year, month, day).and_hms(h, m, s)))
            
        } else {
            log::error!("No time start found in payload");
            Ok(None)
        }
    }

    fn get_text(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let ret = reqwest::blocking::get(url)?.text()?;
        Ok(ret)
    }

    fn sleep(duration: Duration) {
        use std::ops::Add;
        let end = Utc::now().add(duration);
        let one_hour: Duration = Duration::hours(1);
        let one_minute: Duration = Duration::minutes(1);
        let five_minutes = Duration::minutes(5);
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
            } else if diff > Duration::zero() {
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
        let local_time: DateTime<Local> = date.into();
        format!("{}", local_time.format("%-l:%M %p"))
    }
    #[cfg(windows)]
    fn alert(
        response: &Response,
        path: &std::path::Path,
        kind: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        trace!("alert {} {}", kind, path.display());
        let msg = match kind {
            "json" => serialize_json(response)?,
            "title-price" => format_deal(&deal).0,
            _ => reqwest.deal.title.clone(),
        };
        let local_path = path.to_owned();
        std::thread::spawn(move || {
            let _ = std::process::Command::new("powershell")
                .arg(&local_path)
                .arg(&msg)
                .output()
                .map_err(|e| {
                    error!("failed to execute alert: {}", e)
                });
        });
        Ok(())
    }
    #[cfg(unix)]
    fn alert(
        response: &Response,
        path: &std::path::Path,
        kind: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msgs = match kind {
            "json" => vec![serialize_json(response)?],
            "title-price" => format_deal(&response.deal).0.replace(")", "").split("(").map(|s|s.to_string()).collect::<Vec<String>>(),
            _ => vec![response.deal.title.clone()],
        };
        let local_path = path.to_owned();
        std::thread::spawn(move || {
            let out = std::process::Command::new("bash")
                .arg(&local_path)
                .args(&msgs)
                .output()
                .map_err(|e| {
                    log::error!("failed to execute alert: {}", e)
                })
                .unwrap();
            debug!(
                "output:\n{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        });
        Ok(())
    }

    fn serialize_json(r: &Response) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(r)?;
        Ok(json)
    }
}
