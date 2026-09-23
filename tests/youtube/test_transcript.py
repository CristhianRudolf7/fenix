"""Offline contract tests for Fénix's caption helper and real adapter parser."""

import importlib.util
import json
import math
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import requests

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "resources/skills/youtube-summary/scripts/youtube_transcript.py"
spec = importlib.util.spec_from_file_location("youtube_helper", HELPER)
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)
ID = "AbCdEf_12-3"
URL = f"https://www.youtube.com/watch?v={ID}"
CAPTION_URL = f"https://www.youtube.com/api/timedtext?v={ID}&lang=es"


def response(body, status=200):
    result = requests.Response()
    result.status_code = status
    result.url = URL
    result._content = body.encode() if isinstance(body, str) else body
    result._content_consumed = True
    result.encoding = "utf-8"
    return result


def track(language, generated=False):
    return SimpleNamespace(language_code=language, is_generated=generated)


def snippet(text="Example", start=0, duration=1):
    return SimpleNamespace(text=text, start=start, duration=duration)


def player(captions=True, generated=False):
    result = {"playabilityStatus": {"status": "OK"}}
    if captions:
        item = {"baseUrl": CAPTION_URL, "name": {"runs": [{"text": "Spanish"}]}, "languageCode": "es"}
        if generated:
            item["kind"] = "asr"
        result["captions"] = {"playerCaptionsTracklistRenderer": {"captionTracks": [item]}}
    return json.dumps(result)


class TranscriptTests(unittest.TestCase):
    def test_supported_urls_canonicalize_to_one_id(self):
        for url in [URL + "&list=other&t=2", f"https://youtu.be/{ID}?si=tracking",
                    f"https://m.youtube.com/shorts/{ID}", f"https://youtube.com/live/{ID}",
                    f"https://www.youtube-nocookie.com/embed/{ID}", URL.replace("https:", "http:")]:
            with self.subTest(url=url):
                self.assertEqual(helper.video_id_from_url(url), ID)

    def test_foreign_ambiguous_and_credential_urls_rejected(self):
        for url in [f"https://youtube.com.evil.test/watch?v={ID}", f"https://youtube.com@evil.test/watch?v={ID}",
                    f"https://x@youtube.com/watch?v={ID}", URL + f"&v={ID}",
                    "https://youtube.com/playlist?list=abc", "http://127.0.0.1/watch?v=" + ID,
                    "file:///etc/passwd", URL.replace("youtube.com", "youtube.com:443"),
                    f"https://youtu.be/{ID}/other", "https://youtube.com/watch?v=bad", URL + "\n"]:
            with self.subTest(url=url), self.assertRaises(helper.TranscriptError):
                helper.video_id_from_url(url)

    def test_preferred_language_beats_unrelated_manual_track(self):
        selected, fallback = helper.select_track([track("en"), track("es", True)], ["es", "en"])
        self.assertEqual(selected.language_code, "es")
        self.assertTrue(selected.is_generated)
        self.assertFalse(fallback)

    def test_manual_and_regional_preference(self):
        selected, _ = helper.select_track([track("es", True), track("es-419")], ["es"])
        self.assertEqual(selected.language_code, "es-419")
        selected, fallback = helper.select_track([track("es-ES"), track("en")], ["es-MX", "en"])
        self.assertEqual(selected.language_code, "en")
        self.assertFalse(fallback)

    def test_fallback_is_explicit_and_strict_can_refuse(self):
        selected, fallback = helper.select_track([track("fr", True), track("de")], ["es"])
        self.assertEqual(selected.language_code, "de")
        self.assertTrue(fallback)
        with self.assertRaises(helper.TranscriptError) as error:
            helper.select_track([track("fr")], ["es"], True)
        self.assertEqual(error.exception.code, "language_unavailable")

    def test_empty_track_list_and_bad_preferences(self):
        with self.assertRaises(helper.TranscriptError):
            helper.select_track([], ["es"])
        for value in ["", "es,,en", "es;curl", "../../", ",".join(["en"] * 9)]:
            with self.subTest(value=value), self.assertRaises(helper.TranscriptError):
                helper.language_preferences(value)
        self.assertEqual(helper.language_preferences("ES,en,es"), ["es", "en"])

    def test_normalization_retains_repetition_overlap_and_timestamps(self):
        result = helper.normalize_segments([snippet("Hello\n world", 2.9, 4), snippet("Hello world", 3, 1)], ID)
        self.assertEqual(len(result), 2)
        self.assertEqual(result[0]["text"], "Hello world")
        self.assertEqual(result[0]["start_seconds"], 2.9)
        self.assertTrue(result[0]["url"].endswith("&t=2s"))

    def test_invalid_timing_and_empty_text_fail(self):
        for snippets in [[snippet(start=-1)], [snippet(duration=math.nan)], [snippet(start=True)],
                         [snippet(start=2), snippet(start=1)], [snippet(" \n ")], [snippet(None)]]:
            with self.subTest(snippets=snippets), self.assertRaises(helper.TranscriptError):
                helper.normalize_segments(snippets, ID)

    def test_segment_and_text_limits_fail_without_partial_result(self):
        with patch.object(helper, "MAX_SEGMENTS", 1), self.assertRaises(helper.TranscriptError):
            helper.normalize_segments([snippet(), snippet()], ID)
        with patch.object(helper, "MAX_TEXT_BYTES", 2), self.assertRaises(helper.TranscriptError):
            helper.normalize_segments([snippet("abc")], ID)

    def test_network_refuses_foreign_paths_video_mismatch_and_redirects(self):
        for url in ["https://evil.test/api/timedtext", "https://www.youtube.com/other",
                    CAPTION_URL.replace(ID, "other-id123"), CAPTION_URL.replace("https:", "http:")]:
            with helper.bounded_session(ID) as session, patch.object(requests.Session, "request") as send:
                with self.subTest(url=url), self.assertRaises(helper.TranscriptError):
                    session.get(url)
                send.assert_not_called()
        for status in [302, 401, 403, 429]:
            with helper.bounded_session(ID) as session, patch.object(requests.Session, "request", return_value=response("", status)):
                with self.assertRaises(helper.TranscriptError) as error:
                    session.get(CAPTION_URL)
                self.assertEqual(error.exception.code, "access_blocked")

    def test_network_bounds_size_time_requests_and_disables_ambient_auth(self):
        with helper.bounded_session(ID) as session, patch.object(requests.Session, "request", return_value=response(b"abcd")) as send:
            self.assertFalse(session.trust_env)
            with patch.object(helper, "MAX_RESPONSE_BYTES", 3), self.assertRaises(helper.TranscriptError):
                session.get(CAPTION_URL)
            self.assertFalse(send.call_args.kwargs["allow_redirects"])
            self.assertTrue(send.call_args.kwargs["stream"])
            self.assertEqual(send.call_args.kwargs["timeout"], (5, 12))
        with helper.bounded_session(ID) as session, patch.object(requests.Session, "request") as send:
            session.deadline = 0
            with self.assertRaises(helper.TranscriptError):
                session.get(CAPTION_URL)
            send.assert_not_called()
        with helper.bounded_session(ID) as session, patch.object(requests.Session, "request") as send:
            session.request_count = helper.MAX_REQUESTS
            with self.assertRaises(helper.TranscriptError):
                session.get(CAPTION_URL)
            send.assert_not_called()

    def test_real_adapter_parses_synthetic_captions_through_bounded_transport(self):
        xml = '<transcript><text start="0.5" dur="1.25">Synthetic &amp; caption</text><text start="2" dur="1">Final point</text></transcript>'
        replies = [response('{"INNERTUBE_API_KEY":"fixture"}'), response(player()), response(xml)]
        with patch.object(requests.Session, "request", side_effect=replies) as send:
            result = helper.fetch_transcript(URL)
        self.assertEqual(send.call_count, 3)
        self.assertEqual(result["status"], "ok")
        self.assertEqual(result["language"], "es")
        self.assertFalse(result["is_generated"])
        self.assertEqual(result["segments"][0]["text"], "Synthetic & caption")
        self.assertEqual(result["caption_end_seconds"], 3)
        self.assertEqual(result["segment_count"], 2)
        self.assertEqual(len(result["sha256"]), 64)

    def test_automatic_captions_and_other_language_are_disclosed(self):
        replies = [response('{"INNERTUBE_API_KEY":"fixture"}'), response(player(generated=True)),
                   response('<transcript><text start="1" dur="1">Synthetic</text></transcript>')]
        with patch.object(requests.Session, "request", side_effect=replies):
            result = helper.fetch_transcript(URL, "fr")
        self.assertTrue(result["is_generated"])
        self.assertTrue(result["language_fallback"])
        self.assertTrue(any("Automatic" in warning for warning in result["warnings"]))

    def test_disabled_captions_remain_distinct_from_provider_format_failure(self):
        replies = [response('{"INNERTUBE_API_KEY":"fixture"}'), response(player(captions=False))]
        with patch.object(requests.Session, "request", side_effect=replies):
            try:
                helper.fetch_transcript(URL)
            except Exception as error:
                result = helper.error_result(error)
        self.assertEqual(result["code"], "no_captions")
        with patch.object(requests.Session, "request", return_value=response("unrecognized HTML")):
            try:
                helper.fetch_transcript(URL)
            except Exception as error:
                result = helper.error_result(error)
        self.assertEqual(result["code"], "provider_changed")

    def test_upstream_error_bodies_are_not_disclosed(self):
        result = helper.error_result(requests.HTTPError("private signed-url-secret"))
        self.assertEqual(result["code"], "network_error")
        self.assertNotIn("signed-url-secret", json.dumps(result))

    def test_cli_rejects_bad_url_with_machine_readable_error(self):
        result = subprocess.run([sys.executable, "-I", str(HELPER), "https://localhost/private"], capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(json.loads(result.stdout)["code"], "invalid_url")
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
