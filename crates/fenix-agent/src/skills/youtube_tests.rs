use super::*;

#[tokio::test]
async fn youtube_summary_routes_video_requests_and_bundles_its_helper() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rt = Runtime::open(&dir.path().join("data"), Some(&dir.path().join("skills"))).await?;
    for query in [
        "Resume este video de YouTube usando sus subtítulos",
        "Summarize this YouTube video and link the key timestamps",
        "Dime de qué trata este enlace de YouTube",
        "Get the transcript of this youtu.be link",
        "Resumen de un video con subtítulos automáticos",
        "YouTube video summary in Spanish",
        "Resume https://www.youtube.com/watch?v=jNQXAC9IVRw",
        "Summarize https://youtu.be/jNQXAC9IVRw",
    ] {
        let result = rt.search_skills(query, 3)?;
        assert!(
            result["skills"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["skill"]["name"] == "youtube-summary"),
            "{query}: {result}"
        );
    }
    for (query, expected) in [
        (
            "Transcribe this local audio recording with timestamps",
            "audio-transcription",
        ),
        (
            "Edit this video timeline and export the result",
            "video-editing",
        ),
        (
            "Summarize previous conversation decisions",
            "conversation-recap",
        ),
        ("Search the internet for recent news", "web-research"),
    ] {
        let result = rt.search_skills(query, 3)?;
        let rows = result["skills"].as_array().unwrap();
        assert!(
            rows.iter().any(|row| row["skill"]["name"] == expected),
            "{query}: {result}"
        );
        assert_ne!(rows[0]["skill"]["name"], "youtube-summary", "{query}");
    }
    for resource in [
        "scripts/youtube_transcript.py",
        "scripts/requirements.txt",
        "references/caption-protocol.md",
    ] {
        assert!(!rt.skill_resource("youtube-summary", resource)?.is_empty());
    }
    rt.set_skill_enabled("youtube-summary", false)?;
    rt.install_resources()?;
    assert!(rt.skill_load("youtube-summary").is_err());
    assert!(
        rt.skill_resource("youtube-summary", "scripts/youtube_transcript.py")
            .is_err()
    );
    rt.shutdown().await?;
    Ok(())
}
