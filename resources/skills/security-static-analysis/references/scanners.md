# Scanner handling

Prefer repository-owned rules to unpinned remote configuration. For Semgrep, use local configuration and disable metrics; do not use autofix during an analysis-only assignment. For CodeQL, record which build and query suite produced the database/results. Missing or failed extraction may leave whole paths unexamined.

Avoid analyzing generated reports or downloaded rule fixtures as application source. Do not deduplicate by title alone: the same rule at different locations may represent independent causes. Preserve original rule IDs and locations for follow-up.

SARIF invocation success is only what the producing tool reported. It does not establish complete repository coverage. Compare claimed files/languages with actual scanner scope and errors. Zero results without successful execution evidence must remain inconclusive.
