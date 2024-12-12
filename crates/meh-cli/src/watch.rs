
use std::{borrow::Cow, collections::HashSet, time::{Duration, SystemTime}};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use meh::{Response, Deal};


#[derive(Default)]
struct WatchProgressBarWrapper(Option<WatchProgressBar>);

impl WatchProgressBarWrapper {
    fn enable(&mut self) {
        self.0 = Some(WatchProgressBar::new());
    }


    fn update_top_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(inner) = &self.0 {
            inner.update_top_message(msg);
        }
    }

    fn update_middle_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(inner) = &self.0 {
            inner.update_middle_message(msg);
        }
    }

    fn update_bottom_message(&self, msg: impl Into<Cow<'static, str>>) {
        if let Some(inner) = &self.0 {
            inner.update_bottom_message(msg);
        }
    }
}

struct WatchProgressBar {
    _multi: MultiProgress,
    top: ProgressBar,
    middle: ProgressBar,
    bottom: ProgressBar,
}

impl WatchProgressBar {
    fn new() -> Self {
        let style = ProgressStyle::default_spinner()
            .tick_chars("█▓▒░  ░▒▓█")
            .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .expect("valid style");
        let multi = MultiProgress::with_draw_target(indicatif::ProgressDrawTarget::stdout());
        let top = multi.add(ProgressBar::new_spinner().with_style(style.clone()));
        top.enable_steady_tick(Duration::from_millis(200));
        let middle = multi.add(ProgressBar::new_spinner().with_style(style.clone()));
        middle.enable_steady_tick(Duration::from_millis(150));
        let bottom = multi.add(ProgressBar::new_spinner().with_style(style.clone()));
        bottom.enable_steady_tick(Duration::from_millis(210));
        Self {
            _multi: multi,
            top,
            middle,
            bottom,
        }
    }

    fn update_top_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.top.set_message(msg);
    }

    fn update_middle_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.middle.set_message(msg);
    }

    fn update_bottom_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.bottom.set_message(msg);
    }
}

pub async fn watch(
    api_key: &str,
    progress: bool,
    interval: u64
) -> Result<(), Box<dyn std::error::Error>> {
    let mut progs = WatchProgressBarWrapper::default();
    if progress {
        progs.enable();
    }
    progs.update_bottom_message("warming up");
    let url = meh::construct_url(api_key);
    let mut last_id = String::new();
    let mut next_request = SystemTime::now() - Duration::from_secs(interval);
    loop {
        while SystemTime::now() < next_request {
            let rem = next_request.duration_since(SystemTime::now())?;
            progs.update_bottom_message(format!("Next request in {}", format_duration(rem)));
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        progs.update_bottom_message("Requesting info from Meh.com");
        let deal = try_fetch_deal(&url).await?;
        if deal.id != last_id {
            let (desc, link) = format_deal(&deal);
            last_id = deal.id;
            progs.update_top_message(desc);
            progs.update_middle_message(link);
        }
        next_request = SystemTime::now() + Duration::from_secs(interval);
        
    }
}

fn format_duration(d: Duration) -> String {
    let mut secs = d.as_secs();
    let mut hours = 0;
    let mut mins = 0;
    while secs > 60 * 60 {
        hours += 1;
        secs -= 60 * 60;
    }
    while secs > 60 {
        mins += 1;
        secs -= secs;
    }
    let mut ret = String::new();
    if hours > 0 {
        ret.push_str(&format!("{hours}h "));
    }
    if mins > 0 {
        ret.push_str(&format!("{mins}m "));
    }
    if secs > 0 {
        ret.push_str(&format!("{secs}s"));
    }
    ret.trim().to_string()
}

fn format_deal(d: &Deal) -> (String, String) {
    let costs: HashSet<String> = d.items.iter().map(|i| format!("{:.2}", i.price)).collect();
    let costs = Vec::from_iter(costs).join(", ");
    (format!("{} ({})", d.title, costs), d.url.to_string())
}

async fn try_fetch_deal(url: &str) -> Result<Deal, Box<dyn std::error::Error>> {
    let raw_json = reqwest::get(url).await?.text().await?;
    
    Ok(serde_json::from_str::<Response>(&raw_json).map(|r| r.deal).inspect_err(|e| {
        eprintln!("Error parsing Response: {e}\n{raw_json}")
    })?)
}
