# Message fixtures

Responses of `POST /v1/messages` (onboarding and chat interfaces), replayed to the real client by `claude_contract.rs`
and to the app by `e2e/replay-server.ts`. Never refreshed by CI.

These were written by hand from the documented response shapes. They are to be replaced with responses recorded from
the real API by the owner, with the key removed, confirming the prompt-too-long message and the `fallback` block shape.

`tool-use.sse`, `tool-use-parallel.sse` and `after-tools.sse` (memory interfaces) were also written by hand. They are to
be replaced with recorded responses, confirming that `strict` and `eager_input_streaming` are accepted together and that
the tool schemas pass strict mode.

`summary.sse` (a structured-output summary reply) and `reply-large.sse` (a reply whose usage puts the context past the
compaction threshold) belong to the compaction feature and were written by hand too. `summary.sse` is to be replaced
with a recorded response, confirming that `output_config.format` and `effort` are accepted together.

`themes.sse`, `decisions.sse` and `diary.sse` (dreaming: the REM themes, the deep decisions and the diary entry)
were written by hand as well. They are to be replaced with recorded responses, confirming that
`output_config.effort: "medium"` and `format` are accepted together.

`topics.sse` (journey: the day's topics for the world) was written by hand too. The replay server puts the
requested day in place of `2026-11-10`. It is to be replaced with a recorded response.

`reply-mood.sse` (avatar: a reply that opens with a mood marker split across two deltas) was written by hand from
`reply.sse`. The marker is still to be checked against real replies (avatar SC-006).

`tool-use-alarm.sse` (extensions: an `alarm_set` call for a daily alarm) is `tool-use.sse` with the tool and input
changed, written by hand for the end-to-end spec `us15-extensions`.
