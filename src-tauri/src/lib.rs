use mpris::ProgressTick;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri_plugin_http::reqwest;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongData {
    pub id: Option<u64>,
    pub name: Option<String>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum PlayerError {
    #[error("No players found.")]
    NoPlayer,
    #[error("Couldn't find lyrics for this song.")]
    NoLyrics,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct MessageBuilder {
    interval: Vec<(u64, u64, String)>,
    error_message: Option<PlayerError>,
}

impl MessageBuilder {
    fn builder() -> Self {
        MessageBuilder::default()
    }

    fn interval(mut self, interval: &mut [(u64, u64, String)]) -> Self {
        self.interval = interval.to_vec();
        self
    }
}

fn find_in_interval(intervals: &[(u64, u64, String)], value: u64) -> Option<usize> {
    let mut left = 0;
    let mut right = intervals.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if intervals[mid].0 <= value {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    if left > 0 {
        let interval = &intervals[left - 1];
        if interval.0 <= value && value < interval.1 {
            return Some(left - 1);
        }
    }

    None
}

fn interval_lyrics(input: &str) -> Vec<(u64, u64, String)> {
    let re = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2})\]\s+(.+)").unwrap();
    let re2 = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2})\]").unwrap();

    let lyric_count = input.chars().filter(|c| *c == '\n').count() + 1;
    let mut lyric_vec: Vec<(u64, u64, String)> = Vec::with_capacity(lyric_count);
    let first_line = input.lines().next().unwrap();
    let v = re.captures_iter(first_line);

    for g in v {
        let minutes = g[1].parse::<u64>().unwrap();
        let seconds = g[2].parse::<u64>().unwrap();
        let hundredths = g[3].parse::<u64>().unwrap();
        let lyrics = "";

        let time_in_microseconds =
            (minutes * 60 * 1_000_000) + (seconds * 1_000_000) + (hundredths * 10_000);

        lyric_vec.push((0, time_in_microseconds, lyrics.into()));
    }

    let mut iter = re.captures_iter(input).peekable();

    while let Some(group) = iter.next() {
        let minutes: u64 = group[1].parse().unwrap();
        let seconds: u64 = group[2].parse().unwrap();
        let hundredths: u64 = group[3].parse().unwrap();
        let lyrics = &group[4];

        let time_in_microseconds =
            (minutes * 60 * 1_000_000) + (seconds * 1_000_000) + (hundredths * 10_000);

        let after = iter.peek();

        if after.is_none() {
            let last_timestamp = input.lines().last().unwrap();

            for g in re2.captures_iter(last_timestamp) {
                let minutes_last: u64 = g[1].parse().unwrap();
                let seconds_last: u64 = g[2].parse().unwrap();
                let hundredths_last: u64 = g[3].parse().unwrap();

                let time_in_microseconds_last = (minutes_last * 60 * 1_000_000)
                    + (seconds_last * 1_000_000)
                    + (hundredths_last * 10_000);

                lyric_vec.push((
                    time_in_microseconds,
                    time_in_microseconds_last,
                    lyrics.into(),
                ));
            }

            break;
        }

        let after = after.unwrap();

        let minutes_next: u64 = after[1].parse().unwrap();
        let seconds_next: u64 = after[2].parse().unwrap();
        let hundredths_next: u64 = after[3].parse().unwrap();

        let time_in_microseconds_next = (minutes_next * 60 * 1_000_000)
            + (seconds_next * 1_000_000)
            + (hundredths_next * 10_000);

        lyric_vec.push((
            time_in_microseconds,
            time_in_microseconds_next,
            lyrics.into(),
        ));
    }

    lyric_vec
}

type Progress = usize;

#[tauri::command]
async fn get(c: Channel<MessageBuilder>, p: Channel<Progress>) {
    loop {
        let exists = std::process::Command::new("playerctl")
            .arg("status")
            .output()
            .unwrap()
            .status;

        if !exists.success() {
            let mut builder = MessageBuilder::builder();
            builder.error_message = Some(PlayerError::NoPlayer);
            c.send(builder).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        }

        let mut builder = MessageBuilder::builder();

        let metadata = std::process::Command::new("playerctl")
            .arg("metadata")
            .args(["--format", "{{ artist }}\n{{ title }}"])
            .output()
            .unwrap()
            .stdout;

        let data = String::from_utf8(metadata).unwrap();
        let mut data = data.lines();

        let artist = {
            let artist = data.next();

            if let Some(artist) = artist {
                artist
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
                continue;
            }
        };

        let artist = if artist.contains(" - Topic") {
            artist.replace(" - Topic", "")
        } else {
            artist.to_owned()
        };

        let title = data.next().unwrap().to_string();

        let query = format!("{artist} {title}");

        let url = reqwest::Url::parse_with_params("https://lrclib.net/api/search", [("q", &query)])
            .unwrap()
            .to_string();

        let lyrics = {
            let mut response = reqwest::get(&url)
                .await
                .unwrap()
                .json::<Vec<SongData>>()
                .await
                .unwrap();

            if response.is_empty() {
                let query = title.to_string();

                let url = reqwest::Url::parse_with_params(
                    "https://lrclib.net/api/search",
                    [("q", &query)],
                )
                .unwrap()
                .to_string();

                response = reqwest::get(&url)
                    .await
                    .unwrap()
                    .json::<Vec<SongData>>()
                    .await
                    .unwrap();
            }

            if response.is_empty() {
                builder.error_message = Some(PlayerError::NoLyrics);
                c.send(builder).unwrap();
                return;
            }

            response
        };

        let first_song = lyrics[0].to_owned();
        let lyrics = first_song.synced_lyrics;

        let lyrics = if let Some(lyrics) = lyrics {
            lyrics
        } else {
            builder.error_message = Some(PlayerError::NoLyrics);
            c.send(builder).unwrap();
            return;
        };

        let mut interval = interval_lyrics(&lyrics);
        let message = MessageBuilder::builder().interval(&mut interval);
        c.send(message).unwrap();

        let mut current_title: String;

        let r = mpris::PlayerFinder::new().unwrap().find_active().unwrap();
        let mut t = r.track_progress(50).unwrap();

        loop {
            let ProgressTick { progress, .. } = t.tick();
            let position = progress.position().as_micros();
            current_title = progress.metadata().title().unwrap().to_string();

            if current_title != title {
                break;
            }

            let idx = find_in_interval(&interval, position.try_into().unwrap());

            if let Some(idx) = idx {
                p.send(idx).unwrap();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
