#!/usr/bin/env python3
"""Compare local RSS 2.0 or Atom snapshots without fetching or changing read state."""

import argparse
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
import hashlib
import json
from pathlib import Path
import sys
from urllib.parse import urljoin, urlsplit
import xml.etree.ElementTree as ET

ATOM = "{http://www.w3.org/2005/Atom}"
BASE = "{http://www.w3.org/XML/1998/namespace}base"
CONTENT = "{http://purl.org/rss/1.0/modules/content/}encoded"


def http_url(value):
    parts = urlsplit(value)
    if parts.scheme not in ("http", "https") or not parts.hostname or parts.username or parts.password:
        raise ValueError("Expected an HTTP(S) URL without embedded credentials")
    return value


def value(element, tag):
    child = element.find(tag)
    return "" if child is None else "".join(child.itertext()).strip()


def date_value(raw):
    if not raw:
        return {"raw": raw, "utc": None}
    try:
        try:
            parsed = datetime.fromisoformat(raw.replace("Z", "+00:00"))
        except ValueError:
            parsed = parsedate_to_datetime(raw)
        if parsed.tzinfo is None:
            return {"raw": raw, "utc": None}
        return {"raw": raw, "utc": parsed.astimezone(timezone.utc).isoformat()}
    except (ValueError, TypeError, OverflowError):
        return {"raw": raw, "utc": None}


def parse(data, feed_url):
    http_url(feed_url)
    if len(data) > 8 * 1024 * 1024:
        raise ValueError("Feed exceeds 8 MiB")
    text = data.decode("utf-8-sig")
    if "\x00" in text or "<!DOCTYPE" in text.upper() or "<!ENTITY" in text.upper():
        raise ValueError("DTD, entity declarations and non-UTF-8 XML are unsupported")
    root = ET.fromstring(text)
    base = urljoin(feed_url, root.get(BASE, ""))
    if root.tag == ATOM + "feed":
        elements, kind = root.findall(ATOM + "entry"), "atom"
    elif root.tag == "rss" and root.get("version") == "2.0" and root.find("channel") is not None:
        channel = root.find("channel")
        base = urljoin(base, channel.get(BASE, ""))
        elements, kind = channel.findall("item"), "rss2"
    else:
        raise ValueError("Only RSS 2.0 and Atom feeds are supported")
    if len(elements) > 10000:
        raise ValueError("Feed exceeds 10000 items")
    items, duplicate_count = {}, 0
    for item in elements:
        item_base = urljoin(base, item.get(BASE, ""))
        if kind == "atom":
            links = [link for link in item.findall(ATOM + "link") if link.get("rel", "alternate") == "alternate" and link.get("href")]
            preferred = next((link for link in links if link.get("type") in ("text/html", "application/xhtml+xml")), links[0] if links else None)
            link = http_url(urljoin(urljoin(item_base, preferred.get(BASE, "")), preferred.get("href"))) if preferred is not None else ""
            key = value(item, ATOM + "id") or link
            title = value(item, ATOM + "title")
            content = value(item, ATOM + "content") or value(item, ATOM + "summary")
            published, updated = value(item, ATOM + "published"), value(item, ATOM + "updated")
        else:
            raw_link = value(item, "link")
            link = http_url(urljoin(item_base, raw_link)) if raw_link else ""
            key, title = value(item, "guid") or link, value(item, "title")
            content = value(item, CONTENT) or value(item, "description")
            published, updated = value(item, "pubDate"), ""
        if not key:
            raise ValueError("Item lacks a stable identifier and link")
        row = {"id": key, "title": title, "link": link, "content": content,
               "published": date_value(published), "updated": date_value(updated)}
        digest = hashlib.sha256(json.dumps(row, sort_keys=True, ensure_ascii=False).encode()).hexdigest()
        row["fingerprint"] = digest
        if key in items:
            if items[key] != row:
                raise ValueError("Conflicting duplicate item identifier")
            duplicate_count += 1
        else:
            items[key] = row
    return {"feed_url": feed_url, "format": kind, "items": items, "duplicate_rows": duplicate_count}


def compare(current, previous=None):
    if previous and previous["feed_url"] != current["feed_url"]:
        raise ValueError("Cannot compare different feed identities")
    before = {} if previous is None else previous["items"]
    now = current["items"]
    return {"feed_url": current["feed_url"], "initial_baseline": previous is None,
            "new": [key for key in now if key not in before],
            "updated": [key for key in now if key in before and now[key]["fingerprint"] != before[key]["fingerprint"]],
            "unchanged": [key for key in now if key in before and now[key]["fingerprint"] == before[key]["fingerprint"]],
            "absent_from_current": [key for key in before if key not in now],
            "deletions_verified": False, "read_state_changed": False,
            "duplicate_rows": current["duplicate_rows"], "items": list(now.values())}


def read(path):
    with path.open("rb") as stream:
        return stream.read(8 * 1024 * 1024 + 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("current", type=Path)
    parser.add_argument("--previous", type=Path)
    parser.add_argument("--feed-url", required=True)
    args = parser.parse_args()
    try:
        current = parse(read(args.current), args.feed_url)
        previous = parse(read(args.previous), args.feed_url) if args.previous else None
        result = compare(current, previous)
    except (ValueError, OSError, ET.ParseError) as error:
        print(json.dumps({"error": str(error)}))
        return 1
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
