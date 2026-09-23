#!/usr/bin/env python3
"""Fetch bounded public YouTube captions; leave interpretation to the Fénix agent."""

import argparse
import hashlib
import json
import math
import re
import signal
import sys
import time
from datetime import datetime, timezone
from importlib.metadata import version
from urllib.parse import parse_qs, urlsplit

MAX_RESPONSE_BYTES = 4 * 1024 * 1024
MAX_TEXT_BYTES = 2 * 1024 * 1024
MAX_SEGMENTS = 20_000
MAX_REQUESTS = 5
DEADLINE_SECONDS = 60
VIDEO_ID = re.compile(r"[A-Za-z0-9_-]{11}\Z")


class TranscriptError(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def video_id_from_url(value):
    """Accept one video, canonicalizing away tracking, fragments and playlist IDs."""
    if not isinstance(value, str) or len(value) > 2048 or any(c.isspace() for c in value):
        raise TranscriptError("invalid_url", "Provide one valid YouTube video URL.")
    try:
        url = urlsplit(value)
        port = url.port
    except ValueError:
        raise TranscriptError("invalid_url", "The video URL is malformed.") from None
    if url.scheme not in ("http", "https") or url.username or url.password or port is not None:
        raise TranscriptError("invalid_url", "Use a YouTube URL without credentials or a port.")
    host = (url.hostname or "").lower()
    parts = url.path.strip("/").split("/")
    if host == "youtu.be" and len(parts) == 1:
        candidate = parts[0]
    elif host in ("youtube.com", "www.youtube.com", "m.youtube.com", "music.youtube.com"):
        if url.path == "/watch":
            values = parse_qs(url.query).get("v", [])
            candidate = values[0] if len(values) == 1 else ""
        elif len(parts) == 2 and parts[0] in ("shorts", "live", "embed"):
            candidate = parts[1]
        else:
            candidate = ""
    elif host in ("youtube-nocookie.com", "www.youtube-nocookie.com") and len(parts) == 2 and parts[0] == "embed":
        candidate = parts[1]
    else:
        candidate = ""
    if not VIDEO_ID.fullmatch(candidate):
        raise TranscriptError("invalid_url", "The URL must identify a single YouTube video.")
    return candidate


def language_preferences(value):
    codes = [code.strip().lower() for code in value.split(",")]
    if not 1 <= len(codes) <= 8 or any(not re.fullmatch(r"[a-z]{2,3}(?:-[a-z0-9]{2,8})*", code) for code in codes):
        raise TranscriptError("invalid_languages", "Provide up to eight comma-separated language codes.")
    return list(dict.fromkeys(codes))


def select_track(tracks, preferences, strict=False):
    if not tracks:
        raise TranscriptError("no_captions", "YouTube returned no caption tracks.")
    for preferred in preferences:
        matches = [t for t in tracks if t.language_code.lower() == preferred or
                   ("-" not in preferred and t.language_code.lower().startswith(preferred + "-"))]
        if matches:
            return min(matches, key=lambda t: (t.is_generated, t.language_code.lower() != preferred, t.language_code)), False
    if strict:
        raise TranscriptError("language_unavailable", "No caption track matches the requested languages.")
    # No implicit translation endpoint: the agent can summarize the actual source language.
    return min(tracks, key=lambda t: (t.is_generated, t.language_code)), True


def normalize_segments(snippets, video_id):
    segments = []
    text_bytes = 0
    previous_start = -1.0
    for index, snippet in enumerate(snippets):
        if index >= MAX_SEGMENTS:
            raise TranscriptError("limit_exceeded", "The transcript exceeds the segment limit.")
        start, duration = snippet.start, snippet.duration
        if any(isinstance(n, bool) or not isinstance(n, (int, float)) or not math.isfinite(n) or n < 0 for n in (start, duration)):
            raise TranscriptError("invalid_captions", "Caption timings are invalid.")
        if start < previous_start or start + duration > 31 * 24 * 3600:
            raise TranscriptError("invalid_captions", "Caption timings are out of order or out of range.")
        previous_start = start
        if not isinstance(snippet.text, str):
            raise TranscriptError("invalid_captions", "Caption text is invalid.")
        text = " ".join(snippet.text.split())
        text_bytes += len(text.encode("utf-8"))
        if text_bytes > MAX_TEXT_BYTES:
            raise TranscriptError("limit_exceeded", "The transcript exceeds the text limit.")
        if not text:
            continue
        # Preserve repeats and overlap: these can be actual speech, not duplicate rows.
        segments.append({"start_seconds": start, "duration_seconds": duration, "text": text,
                         "url": f"https://www.youtube.com/watch?v={video_id}&t={int(start)}s"})
    if not segments:
        raise TranscriptError("empty_captions", "The selected track contains no usable caption text.")
    return segments


def bounded_session(video_id):
    import requests

    class CaptionSession(requests.Session):
        def __init__(self):
            super().__init__()
            self.trust_env = False  # No inherited proxies, .netrc credentials or browser cookies.
            self.deadline = time.monotonic() + DEADLINE_SECONDS
            self.request_count = 0

        def request(self, method, url, **kwargs):
            parsed = urlsplit(url)
            valid = (parsed.scheme == "https" and parsed.hostname in ("www.youtube.com", "youtube.com")
                     and not parsed.username and not parsed.password and parsed.port in (None, 443))
            allowed = {("GET", "/watch"), ("GET", "/api/timedtext"), ("POST", "/youtubei/v1/player")}
            if not valid or (method.upper(), parsed.path) not in allowed:
                raise TranscriptError("unsafe_caption_url", "The provider returned an unexpected caption endpoint.")
            if method.upper() == "GET" and parse_qs(parsed.query).get("v") != [video_id]:
                raise TranscriptError("unsafe_caption_url", "The caption endpoint does not match the video.")
            if method.upper() == "POST" and kwargs.get("json", {}).get("videoId") != video_id:
                raise TranscriptError("unsafe_caption_url", "The player request does not match the video.")
            self.request_count += 1
            remaining = self.deadline - time.monotonic()
            if self.request_count > MAX_REQUESTS or remaining <= 0:
                raise TranscriptError("limit_exceeded", "The caption request budget was exhausted.")
            kwargs.update(timeout=(min(5, remaining), min(12, remaining)), stream=True, allow_redirects=False)
            response = super().request(method, url, **kwargs)
            try:
                if response.status_code in (401, 403, 429) or 300 <= response.status_code < 400:
                    raise TranscriptError("access_blocked", "YouTube blocked access or requires an unsupported redirect/login.")
                response.raise_for_status()
                chunks = []
                size = 0
                for chunk in response.iter_content(64 * 1024):
                    size += len(chunk)
                    if size > MAX_RESPONSE_BYTES:
                        raise TranscriptError("limit_exceeded", "The caption response exceeded its byte limit.")
                    if time.monotonic() >= self.deadline:
                        raise TranscriptError("timeout", "Caption retrieval exceeded its deadline.")
                    chunks.append(chunk)
                response._content = b"".join(chunks)
                response._content_consumed = True
                return response
            finally:
                response.close()

    return CaptionSession()


def fetch_transcript(url, languages="es,en", strict=False):
    video_id = video_id_from_url(url)
    preferences = language_preferences(languages)
    try:
        from youtube_transcript_api import YouTubeTranscriptApi
    except ImportError:
        raise TranscriptError("dependency_missing", "Install the pinned YouTube caption dependencies in the execution image.") from None
    with bounded_session(video_id) as session:
        tracks = list(YouTubeTranscriptApi(http_client=session).list(video_id))
        track, fallback = select_track(tracks, preferences, strict)
        segments = normalize_segments(track.fetch(), video_id)
    content = json.dumps(segments, ensure_ascii=False, separators=(",", ":"))
    warnings = ["Captions describe speech; visual content has not been inspected.",
                "A complete fetched track does not prove all speech in the video was captioned."]
    if track.is_generated:
        warnings.append("Automatic captions can misrecognize names, numbers and technical terms.")
    if fallback:
        warnings.append("Preferred caption languages were unavailable; another existing track was selected.")
    return {"status": "ok", "video_id": video_id, "url": f"https://www.youtube.com/watch?v={video_id}",
            "retrieved_at": datetime.now(timezone.utc).isoformat(), "language": track.language_code,
            "is_generated": track.is_generated, "language_fallback": fallback,
            "source": "youtube_captions", "adapter_version": version("youtube-transcript-api"),
            "track_fetched_completely": True, "segment_count": len(segments),
            "caption_end_seconds": max(s["start_seconds"] + s["duration_seconds"] for s in segments),
            "sha256": hashlib.sha256(content.encode()).hexdigest(), "segments": segments, "warnings": warnings}


def error_result(error):
    if isinstance(error, TranscriptError):
        return {"status": "error", "code": error.code, "message": str(error)}
    # Never print upstream exception messages: they may contain signed URLs or response bodies.
    name = type(error).__name__
    groups = {
        "access_blocked": {"RequestBlocked", "IpBlocked", "AgeRestricted", "PoTokenRequired", "FailedToCreateConsentCookie"},
        "no_captions": {"TranscriptsDisabled", "NoTranscriptFound"},
        "video_unavailable": {"VideoUnavailable", "VideoUnplayable"},
        "provider_changed": {"YouTubeDataUnparsable", "ParseError", "JSONDecodeError", "KeyError"},
        "network_error": {"YouTubeRequestFailed", "ConnectionError", "HTTPError", "SSLError", "Timeout", "ReadTimeout", "ConnectTimeout"},
    }
    code = next((code for code, names in groups.items() if name in names), "retrieval_failed")
    return {"status": "error", "code": code, "message": "No transcript was retrieved. Use authorized visible captions or supply a subtitle file; do not infer the video content.", "error_type": name}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url")
    parser.add_argument("--languages", default="es,en")
    parser.add_argument("--strict-language", action="store_true")
    args = parser.parse_args()

    def deadline_expired(_signum, _frame):
        raise TranscriptError("timeout", "Caption retrieval exceeded its deadline.")

    if hasattr(signal, "SIGALRM"):
        signal.signal(signal.SIGALRM, deadline_expired)
        signal.alarm(DEADLINE_SECONDS)
    try:
        result = fetch_transcript(args.url, args.languages, args.strict_language)
    except Exception as error:
        result = error_result(error)
    finally:
        if hasattr(signal, "SIGALRM"):
            signal.alarm(0)
    print(json.dumps(result, ensure_ascii=False, allow_nan=False))
    return 0 if result["status"] == "ok" else 2


if __name__ == "__main__":
    sys.exit(main())
