//! Serialize the merged transcript into the on-disk JSON contract.

use super::whisper::Segment;
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const TRANSCRIPT_VERSION: u32 = 1;
const TRANSCRIPT_MODEL: &str = "ggml-small.en";
const TRANSCRIPT_LANGUAGE: &str = "en";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TranscriptFile {
    pub version: u32,
    pub audio_file: String,
    pub duration_sec: f64,
    pub model: String,
    pub model_sha256: String,
    pub language: String,
    pub transcribed_at: String,
    pub transcription_wall_sec: f64,
    pub accelerator: String,
    pub segments: Vec<TranscriptSegment>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TranscriptSegment {
    pub start: f64,
    pub end: f64,
    pub speaker: String,
    pub text: String,
}

pub fn write_transcript_json(
    path: &Path,
    audio_file: &str,
    duration_sec: f64,
    model_sha256: &str,
    wall_sec: f64,
    accelerator: &str,
    segments: &[Segment],
) -> Result<()> {
    let file = transcript_file(
        audio_file,
        duration_sec,
        model_sha256,
        wall_sec,
        accelerator,
        segments,
    );
    let pretty = serde_json::to_string_pretty(&file).context("failed to serialize transcript")?;
    let temp_path = temp_output_path(path);
    fs::write(&temp_path, pretty)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "failed to move completed transcript into place at {}",
            path.display()
        )
    })?;
    Ok(())
}

fn transcript_file(
    audio_file: &str,
    duration_sec: f64,
    model_sha256: &str,
    wall_sec: f64,
    accelerator: &str,
    segments: &[Segment],
) -> TranscriptFile {
    TranscriptFile {
        version: TRANSCRIPT_VERSION,
        audio_file: audio_file.to_string(),
        duration_sec,
        model: TRANSCRIPT_MODEL.to_string(),
        model_sha256: model_sha256.to_string(),
        language: TRANSCRIPT_LANGUAGE.to_string(),
        transcribed_at: Local::now().to_rfc3339(),
        transcription_wall_sec: wall_sec,
        accelerator: accelerator.to_string(),
        segments: segments
            .iter()
            .map(|segment| TranscriptSegment {
                start: segment.start_sec,
                end: segment.end_sec,
                speaker: segment.speaker.as_str().to_string(),
                text: segment.text.clone(),
            })
            .collect(),
    }
}

fn temp_output_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcription::whisper::{Segment, Speaker};
    use chrono::DateTime;
    use jsonschema::{Draft, JSONSchema};
    use serde_json::{json, Value};

    fn tempdir(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("domino-test-output-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_segments() -> Vec<Segment> {
        vec![
            Segment {
                start_sec: 0.0,
                end_sec: 1.25,
                speaker: Speaker::You,
                text: "Hey, thanks for joining.".to_string(),
            },
            Segment {
                start_sec: 1.4,
                end_sec: 2.8,
                speaker: Speaker::Meeting,
                text: "Yeah, happy to.".to_string(),
            },
        ]
    }

    fn transcript_schema() -> Value {
        json!({
            "type": "object",
            "required": [
                "version",
                "audio_file",
                "duration_sec",
                "model",
                "model_sha256",
                "language",
                "transcribed_at",
                "transcription_wall_sec",
                "accelerator",
                "segments"
            ],
            "additionalProperties": false,
            "properties": {
                "version": { "const": 1 },
                "audio_file": { "type": "string" },
                "duration_sec": { "type": "number" },
                "model": { "const": "ggml-small.en" },
                "model_sha256": { "type": "string" },
                "language": { "const": "en" },
                "transcribed_at": { "type": "string" },
                "transcription_wall_sec": { "type": "number" },
                "accelerator": { "type": "string" },
                "segments": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["start", "end", "speaker", "text"],
                        "additionalProperties": false,
                        "properties": {
                            "start": { "type": "number" },
                            "end": { "type": "number" },
                            "speaker": { "enum": ["You", "Meeting"] },
                            "text": { "type": "string" }
                        }
                    }
                }
            }
        })
    }

    #[test]
    fn test_roundtrip() {
        let dir = tempdir("roundtrip");
        let path = dir.join("transcript.json");
        let segments = sample_segments();

        write_transcript_json(
            &path,
            "meeting.opus",
            62.5,
            "abcdef1234",
            3.75,
            "metal",
            &segments,
        )
        .unwrap();

        let file: TranscriptFile = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();

        assert_eq!(file.version, 1);
        assert_eq!(file.audio_file, "meeting.opus");
        assert_eq!(file.duration_sec, 62.5);
        assert_eq!(file.model, "ggml-small.en");
        assert_eq!(file.model_sha256, "abcdef1234");
        assert_eq!(file.language, "en");
        assert_eq!(file.transcription_wall_sec, 3.75);
        assert_eq!(file.accelerator, "metal");
        assert_eq!(
            file.segments,
            vec![
                TranscriptSegment {
                    start: 0.0,
                    end: 1.25,
                    speaker: "You".to_string(),
                    text: "Hey, thanks for joining.".to_string(),
                },
                TranscriptSegment {
                    start: 1.4,
                    end: 2.8,
                    speaker: "Meeting".to_string(),
                    text: "Yeah, happy to.".to_string(),
                }
            ]
        );
        DateTime::parse_from_rfc3339(&file.transcribed_at).unwrap();

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_json_schema_conformance() {
        let dir = tempdir("schema");
        let path = dir.join("transcript.json");

        write_transcript_json(
            &path,
            "meeting.opus",
            12.0,
            "deadbeef",
            1.25,
            "cpu",
            &sample_segments(),
        )
        .unwrap();

        let instance: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let schema = transcript_schema();
        let validator = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(&schema)
            .unwrap();

        if let Err(errors) = validator.validate(&instance) {
            let messages: Vec<String> = errors.map(|error| error.to_string()).collect();
            panic!("schema validation failed: {}", messages.join("; "));
        }

        fs::remove_dir_all(&dir).ok();
    }
}
