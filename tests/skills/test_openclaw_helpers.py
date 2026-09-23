"""Offline behavioral checks for the original OpenClaw-inspired helpers."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
SKILLS = ROOT / "resources/skills"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, SKILLS / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


feed = load("feed", "feed-monitoring/scripts/compare_feeds.py")
times = load("times", "calendar-scheduling/scripts/local_time_candidates.py")
statement = load("statement", "invoice-and-expense-records/scripts/reconcile_statement.py")
srt = load("srt", "audio-transcription/scripts/check_srt.py")


def rss(items):
    return ('<rss version="2.0"><channel>' + items + '</channel></rss>').encode()


def item(key, text="Text", date="Tue, 01 Sep 2026 12:00:00 GMT"):
    return f'<item><guid isPermaLink="false">{key}</guid><title>{text}</title><link>/{key}</link><pubDate>{date}</pubDate></item>'


class FeedTests(unittest.TestCase):
    def parse(self, data):
        return feed.parse(data, "https://example.org/feed")

    def test_late_arrivals_and_edits_do_not_depend_on_high_water_date(self):
        before = self.parse(rss(item("a") + item("gone")))
        after = self.parse(rss(item("a", "Edited") + item("late", date="Mon, 31 Aug 2026 12:00:00 GMT")))
        result = feed.compare(after, before)
        self.assertEqual(result["new"], ["late"])
        self.assertEqual(result["updated"], ["a"])
        self.assertEqual(result["absent_from_current"], ["gone"])
        self.assertFalse(result["deletions_verified"])
        self.assertFalse(result["read_state_changed"])

    def test_initial_scan_and_idempotent_comparison(self):
        snapshot = self.parse(rss(item("a")))
        self.assertTrue(feed.compare(snapshot)["initial_baseline"])
        result = feed.compare(snapshot, snapshot)
        self.assertEqual(result["new"], [])
        self.assertEqual(result["updated"], [])
        self.assertEqual(result["unchanged"], ["a"])

    def test_atom_namespace_xml_base_and_dates(self):
        data = b'''<feed xmlns="http://www.w3.org/2005/Atom" xml:base="https://example.net/root/">
        <entry xml:base="posts/"><id>urn:item:1</id><title>Entry</title><link href="one"/>
        <published>2026-09-01T10:00:00-05:00</published><updated>2026-09-02T01:00:00Z</updated>
        <content type="text">Body</content></entry></feed>'''
        row = self.parse(data)["items"]["urn:item:1"]
        self.assertEqual(row["link"], "https://example.net/root/posts/one")
        self.assertEqual(row["published"]["utc"], "2026-09-01T15:00:00+00:00")
        self.assertEqual(row["content"], "Body")

    def test_duplicate_identity_conflicts_are_not_silently_merged(self):
        self.assertEqual(self.parse(rss(item("a") * 2))["duplicate_rows"], 1)
        with self.assertRaisesRegex(ValueError, "Conflicting"):
            self.parse(rss(item("a") + item("a", "Different")))

    def test_missing_identity_and_unsupported_xml_fail(self):
        for data in (rss('<item><title>No ID</title></item>'), b'<RDF/>', b'<!DOCTYPE rss><rss/>', b'\xff\xfe<\x00'):
            with self.subTest(data=data), self.assertRaises(ValueError):
                self.parse(data)

    def test_unknown_dates_and_invalid_link_scheme(self):
        row = self.parse(rss(item("a", date="unknown")))["items"]["a"]
        self.assertIsNone(row["published"]["utc"])
        self.assertIsNone(feed.date_value("2026-09-01T12:00:00")["utc"])
        with self.assertRaises(ValueError):
            self.parse(rss('<item><guid>a</guid><link>javascript:alert(1)</link></item>'))

    def test_feed_identity_mismatch(self):
        a = self.parse(rss(item("a")))
        b = feed.parse(rss(item("a")), "https://elsewhere.example/feed")
        with self.assertRaises(ValueError):
            feed.compare(a, b)


class TimeTests(unittest.TestCase):
    def test_lima_has_one_known_instant(self):
        result = times.candidates("2026-09-19T09:00", "America/Lima")
        self.assertEqual(result["status"], "unique")
        self.assertEqual(result["candidates"][0]["utc"], "2026-09-19T14:00:00+00:00")

    def test_fall_back_preserves_two_different_instants(self):
        result = times.candidates("2026-11-01T01:30", "America/New_York")
        self.assertEqual(result["status"], "ambiguous")
        self.assertEqual([x["utc"] for x in result["candidates"]], ["2026-11-01T05:30:00+00:00", "2026-11-01T06:30:00+00:00"])

    def test_spring_gap_and_skipped_civil_day(self):
        for wall, zone in (("2026-03-08T02:30", "America/New_York"), ("2011-12-30T12:00", "Pacific/Apia")):
            with self.subTest(zone=zone):
                self.assertEqual(times.candidates(wall, zone)["status"], "nonexistent")

    def test_half_hour_transition_is_not_assumed_to_be_an_hour(self):
        result = times.candidates("2026-04-05T01:45", "Australia/Lord_Howe")
        self.assertEqual([x["utc"] for x in result["candidates"]], ["2026-04-04T14:45:00+00:00", "2026-04-04T15:15:00+00:00"])

    def test_invalid_input_does_not_guess(self):
        for wall, zone in (("tomorrow", "UTC"), ("2026-09-19T12:00Z", "UTC"), ("2026-09-19T12:00", "Missing/Zone")):
            with self.subTest(wall=wall), self.assertRaises((ValueError, KeyError)):
                times.candidates(wall, zone)


class StatementTests(unittest.TestCase):
    def document(self):
        return {"accounts": [{"id": "a", "currency": "USD", "opening": "10.00", "closing": "10.30"}],
                "transactions": [{"id": "one", "account": "a", "currency": "USD", "amount": "0.10", "status": "posted"},
                                 {"id": "two", "account": "a", "currency": "USD", "amount": "0.20", "status": "posted"}]}

    def test_exact_decimal_balance(self):
        result = statement.reconcile(self.document())
        self.assertTrue(result["balanced"])
        self.assertEqual(result["accounts"][0]["calculated_closing"], "10.30")
        self.assertFalse(result["classification_verified"])

    def test_pending_is_excluded_and_duplicates_reported(self):
        data = self.document()
        data["transactions"].append(copy.deepcopy(data["transactions"][0]))
        data["transactions"].append({"id": "pending", "account": "a", "currency": "USD", "amount": "-100.00", "status": "pending"})
        result = statement.reconcile(data)
        self.assertTrue(result["balanced"])
        self.assertEqual(len(result["duplicate_rows"]), 1)
        self.assertEqual(result["accounts"][0]["pending_excluded"], 1)

    def test_conflicting_duplicate_and_currency_mismatch_fail(self):
        for patch in ({"amount": "99.00"}, {"currency": "EUR"}):
            data = self.document()
            row = copy.deepcopy(data["transactions"][0]);row.update(patch)
            data["transactions"].append(row)
            with self.subTest(patch=patch), self.assertRaises(ValueError):
                statement.reconcile(data)

    def test_accounts_with_same_transaction_id_remain_separate(self):
        data = self.document()
        data["accounts"].append({"id": "b", "currency": "EUR", "opening": "1.00", "closing": "0.90"})
        data["transactions"].append({"id": "one", "account": "b", "currency": "EUR", "amount": "-0.10", "status": "posted"})
        self.assertTrue(statement.reconcile(data)["balanced"])

    def test_unresolved_balance_and_ambiguous_amount_formats(self):
        data = self.document();data["accounts"][0]["closing"] = "11.00"
        self.assertEqual(statement.reconcile(data)["accounts"][0]["difference"], "0.70")
        for amount in (1.2, "1,200.00", "NaN", "Infinity", "1e3", "0.1234567890123"):
            with self.subTest(amount=amount), self.assertRaises(ValueError):
                statement.money(amount)

    def test_large_values_keep_small_decimal_contributions(self):
        data = self.document()
        data["accounts"][0].update(opening="999999999999999999999999999999.00", closing="999999999999999999999999999999.30")
        self.assertTrue(statement.reconcile(data)["balanced"])


class SubtitleTests(unittest.TestCase):
    def test_valid_global_times_and_media_bound(self):
        result = srt.check("1\n00:10:01,500 --> 00:10:03,000\nSecond chunk.\n", 700)
        self.assertEqual(result["end_ms"], 603000)
        self.assertFalse(result["speech_accuracy_verified"])

    def test_overlap_is_reported_not_erased(self):
        text = '1\n00:00:00,000 --> 00:00:10,000\nA\n\n2\n00:00:01,000 --> 00:00:02,000\nB\n\n3\n00:00:03,000 --> 00:00:04,000\nC'
        self.assertEqual(srt.check(text)["overlapping_cues"], [2, 3])

    def test_bom_and_crlf(self):
        self.assertEqual(srt.check('\ufeff1\r\n00:00:00,000 --> 00:00:01,000\r\nHello\r\n')["cue_count"], 1)

    def test_empty_invalid_indices_intervals_and_duration(self):
        for text in ('', '2\n00:00:00,000 --> 00:00:01,000\nA', '1\n00:00:02,000 --> 00:00:01,000\nA', '1\n00:61:00,000 --> 00:62:00,000\nA', '1\n00:00:00,000 --> 00:00:01,000\n'):
            with self.subTest(text=text), self.assertRaises(ValueError):
                srt.check(text)
        with self.assertRaises(ValueError):
            srt.check('1\n00:00:00,000 --> 00:00:01,000\nA', .5)
        with self.assertRaises(ValueError):
            srt.check('1\n00:00:00,000 --> 00:00:01,000\nA', float('nan'))


class CliTests(unittest.TestCase):
    def test_clis_emit_json_preserve_inputs_and_expose_non_success(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {"feed.xml": rss(item("a")), "ledger.json": json.dumps(StatementTests().document()).encode(),
                     "captions.srt": b'1\n00:00:00,000 --> 00:00:01,000\nA'}
            for name, data in files.items(): (root / name).write_bytes(data)
            calls = [(feed.__file__, [str(root / 'feed.xml'), '--feed-url', 'https://example.org/feed'], 0),
                     (statement.__file__, [str(root / 'ledger.json')], 0),
                     (srt.__file__, [str(root / 'captions.srt'), '--duration', '0.5'], 1),
                     (times.__file__, ['2026-03-08T02:30', 'America/New_York'], 2)]
            for script, args, expected in calls:
                with self.subTest(script=script):
                    result = subprocess.run([sys.executable, script, *args], capture_output=True, text=True)
                    self.assertEqual(result.returncode, expected, result.stderr)
                    self.assertIsInstance(json.loads(result.stdout), dict)
            for name, data in files.items(): self.assertEqual((root / name).read_bytes(), data)


if __name__ == '__main__':
    unittest.main(verbosity=2)
