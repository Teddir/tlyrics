use super::LyricLine;

/// Parses a standard LRC timestamp string e.g. "01:23.45" or "01:23" or "01:23.456" into seconds.
fn parse_timestamp(ts: &str) -> Option<f64> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let mins: f64 = parts[0].trim().parse().ok()?;
    let secs: f64 = parts[1].trim().parse().ok()?;

    Some(mins * 60.0 + secs)
}

/// Parses full LRC formatted text into sorted LyricLine entries.
pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
    let mut result = Vec::new();
    let mut offset_ms: f64 = 0.0;

    for line in lrc_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for [offset: +/-ms] tag
        if trimmed.starts_with("[offset:") && trimmed.ends_with(']') {
            let val_str = &trimmed[8..trimmed.len() - 1];
            if let Ok(val) = val_str.trim().parse::<f64>() {
                offset_ms = val;
            }
            continue;
        }

        // Skip metadata tags like [ti:], [ar:], [al:], [by:], [length:]
        if trimmed.starts_with('[') {
            let colon_idx = trimmed.find(':');
            let close_bracket = trimmed.find(']');
            if let (Some(c), Some(b)) = (colon_idx, close_bracket) {
                if c < b {
                    let tag_name = &trimmed[1..c];
                    if tag_name == "ti"
                        || tag_name == "ar"
                        || tag_name == "al"
                        || tag_name == "by"
                        || tag_name == "length"
                        || tag_name == "re"
                        || tag_name == "ve"
                    {
                        continue;
                    }
                }
            }
        }

        // Find all [mm:ss.xx] timestamps in this line
        let mut timestamps = Vec::new();
        let mut text_start = 0;

        let bytes = trimmed.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'[' {
                if let Some(close_idx) = trimmed[i..].find(']') {
                    let full_close = i + close_idx;
                    let inner = &trimmed[i + 1..full_close];
                    if let Some(time) = parse_timestamp(inner) {
                        timestamps.push(time);
                        text_start = full_close + 1;
                        i = full_close + 1;
                        continue;
                    }
                }
            }
            if !timestamps.is_empty() && bytes[i] != b'[' {
                break;
            }
            i += 1;
        }

        let lyric_text = if text_start < trimmed.len() {
            trimmed[text_start..].trim().to_string()
        } else {
            String::new()
        };

        for time in timestamps {
            let adjusted_time = (time + (offset_ms / 1000.0)).max(0.0);
            result.push(LyricLine {
                time_secs: adjusted_time,
                text: lyric_text.clone(),
            });
        }
    }

    // Sort chronologically
    result.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// Parses plain (unsynced) text lyrics into evenly spaced or sequential LyricLines.
pub fn parse_plain_lyrics(plain_text: &str, duration_secs: f64) -> Vec<LyricLine> {
    let raw_lines: Vec<&str> = plain_text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if raw_lines.is_empty() {
        return Vec::new();
    }

    let interval = if duration_secs > 0.0 {
        duration_secs / (raw_lines.len() as f64)
    } else {
        3.0
    };

    raw_lines
        .into_iter()
        .enumerate()
        .map(|(i, text)| LyricLine {
            time_secs: i as f64 * interval,
            text: text.to_string(),
        })
        .collect()
}

/// Finds the active lyric line index for a given playback position in seconds.
pub fn find_current_index(lines: &[LyricLine], position_secs: f64) -> Option<usize> {
    if lines.is_empty() || position_secs < lines[0].time_secs {
        return None;
    }

    let mut current_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if line.time_secs <= position_secs {
            current_idx = Some(i);
        } else {
            break;
        }
    }

    current_idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc_basic() {
        let lrc = r#"
[ti:Test Song]
[ar:Test Artist]
[00:08.50]Line 1
[00:15.20]Line 2
[00:22.00]Line 3
"#;
        let lines = parse_lrc(lrc);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].time_secs, 8.5);
        assert_eq!(lines[0].text, "Line 1");
        assert_eq!(lines[1].time_secs, 15.2);
        assert_eq!(lines[1].text, "Line 2");
        assert_eq!(lines[2].time_secs, 22.0);
        assert_eq!(lines[2].text, "Line 3");
    }

    #[test]
    fn test_multi_timestamp() {
        let lrc = "[00:10.00][00:30.00]Chorus line";
        let lines = parse_lrc(lrc);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].time_secs, 10.0);
        assert_eq!(lines[0].text, "Chorus line");
        assert_eq!(lines[1].time_secs, 30.0);
        assert_eq!(lines[1].text, "Chorus line");
    }

    #[test]
    fn test_find_current_index() {
        let lines = vec![
            LyricLine { time_secs: 10.0, text: "A".into() },
            LyricLine { time_secs: 20.0, text: "B".into() },
            LyricLine { time_secs: 30.0, text: "C".into() },
        ];

        assert_eq!(find_current_index(&lines, 5.0), None);
        assert_eq!(find_current_index(&lines, 10.0), Some(0));
        assert_eq!(find_current_index(&lines, 15.0), Some(0));
        assert_eq!(find_current_index(&lines, 20.0), Some(1));
        assert_eq!(find_current_index(&lines, 29.9), Some(1));
        assert_eq!(find_current_index(&lines, 35.0), Some(2));
    }
}
