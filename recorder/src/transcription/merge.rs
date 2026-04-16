#![allow(dead_code)]
// Callers (transcription::run_on_session) wire up in Phase 5.

//! Merge independently transcribed channel segments into a single timeline.

use super::whisper::{Segment, Speaker};
use std::cmp::Ordering;

/// Merge segment lists from both channels, sorted ascending by start time.
///
/// On an exact timestamp tie, `"You"` sorts before `"Meeting"` so output stays
/// deterministic across runs.
pub fn merge_segments(mut you: Vec<Segment>, mut meeting: Vec<Segment>) -> Vec<Segment> {
    let mut out = Vec::with_capacity(you.len() + meeting.len());
    out.append(&mut you);
    out.append(&mut meeting);
    out.sort_by(compare_segments);
    out
}

fn compare_segments(a: &Segment, b: &Segment) -> Ordering {
    a.start_sec
        .partial_cmp(&b.start_sec)
        .unwrap_or(Ordering::Equal)
        .then_with(|| compare_speaker(a.speaker, b.speaker))
}

fn compare_speaker(a: Speaker, b: Speaker) -> Ordering {
    match (a, b) {
        (Speaker::You, Speaker::Meeting) => Ordering::Less,
        (Speaker::Meeting, Speaker::You) => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start_sec: f64, end_sec: f64, speaker: Speaker, text: &str) -> Segment {
        Segment {
            start_sec,
            end_sec,
            speaker,
            text: text.to_string(),
        }
    }

    #[test]
    fn test_interleaves_by_start() {
        let you = vec![
            segment(0.0, 0.8, Speaker::You, "hey"),
            segment(4.2, 4.8, Speaker::You, "wrapping up"),
        ];
        let meeting = vec![
            segment(1.1, 1.7, Speaker::Meeting, "hello"),
            segment(2.5, 3.0, Speaker::Meeting, "question"),
        ];

        let merged = merge_segments(you, meeting);
        let starts: Vec<f64> = merged.iter().map(|segment| segment.start_sec).collect();
        let speakers: Vec<Speaker> = merged.iter().map(|segment| segment.speaker).collect();

        assert_eq!(starts, vec![0.0, 1.1, 2.5, 4.2]);
        assert_eq!(
            speakers,
            vec![
                Speaker::You,
                Speaker::Meeting,
                Speaker::Meeting,
                Speaker::You
            ]
        );
    }

    #[test]
    fn test_tie_prefers_you() {
        let you = vec![segment(3.2, 4.0, Speaker::You, "same start")];
        let meeting = vec![segment(3.2, 3.9, Speaker::Meeting, "same start")];

        let merged = merge_segments(you, meeting);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].speaker, Speaker::You);
        assert_eq!(merged[1].speaker, Speaker::Meeting);
    }

    #[test]
    fn test_empty_inputs() {
        let merged = merge_segments(Vec::new(), Vec::new());
        assert!(merged.is_empty());
    }
}
