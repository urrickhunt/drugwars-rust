use super::paths::app_data_dir;
use chrono::{Datelike, Local};
use std::cmp::Reverse;
use std::fs::{File, create_dir_all};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct HighScore {
    pub total_money: i64,
    pub dealer_rank: String,
    pub date: String,
}

pub struct HighScoreTable {
    entries: Vec<HighScore>,
    path: PathBuf,
}

impl HighScoreTable {
    pub fn load() -> Self {
        let path = score_path();
        Self {
            entries: load_high_scores(&path),
            path,
        }
    }

    pub fn record(&mut self, total_money: i64, dealer_rank: &str) -> io::Result<()> {
        self.entries.push(HighScore {
            total_money,
            dealer_rank: dealer_rank.to_string(),
            date: current_date(),
        });
        self.entries.sort_by_key(|score| Reverse(score.total_money));
        self.entries.truncate(10);
        self.save()
    }

    pub fn entries(&self) -> &[HighScore] {
        &self.entries
    }

    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            create_dir_all(parent)?;
        }
        let mut file = File::create(&self.path)?;
        for score in &self.entries {
            writeln!(
                file,
                "{},{},{}",
                score.total_money, score.dealer_rank, score.date
            )?;
        }
        Ok(())
    }
}

fn current_date() -> String {
    let now = Local::now();
    format!(
        "{:02} / {:02} / {:02}",
        now.month(),
        now.day(),
        now.year() % 100
    )
}

fn score_path() -> PathBuf {
    app_data_dir().map_or_else(
        || PathBuf::from("highscores.txt"),
        |dir| dir.join("highscores.txt"),
    )
}

fn load_high_scores(path: &Path) -> Vec<HighScore> {
    let mut scores = Vec::new();

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let parts: Vec<&str> = line.trim().splitn(3, ',').collect();
            if parts.len() != 3 {
                continue;
            }

            if let Ok(total_money) = parts[0].parse::<i64>() {
                scores.push(HighScore {
                    total_money,
                    dealer_rank: parts[1].to_string(),
                    date: parts[2].to_string(),
                });
            }
        }
    }

    scores
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::HighScoreTable;

    #[test]
    fn record_returns_error_when_score_file_cannot_be_created() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("drugwars-scoreboard-{unique}"));
        fs::create_dir(&path).expect("test directory should be created");

        let mut table = HighScoreTable {
            entries: Vec::new(),
            path: path.clone(),
        };

        assert!(table.record(1_000, "TEST DEALER").is_err());

        fs::remove_dir(&path).expect("test directory should be removed");
    }
}
