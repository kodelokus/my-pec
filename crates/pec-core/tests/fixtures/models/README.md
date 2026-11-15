# Model-list fixtures

Responses of `GET /v1/models` (onboarding interfaces), replayed to the real client by `claude_contract.rs`
and to the app by `e2e/replay-server.ts`. Never refreshed by CI.

`ok.json`, `ok-paged-1.json`, `ok-paged-2.json` and `ok-paged-3.json` were recorded from the real API on 2026-11-13
The key travels only in the `x-api-key` header, so the bodies hold no secret. `ok.json` is the whole list in
one page. The paged set is the same list fetched with `limit=2`, then `after_id` set to the previous page's
`last_id`, the last page without a limit so it ends with `has_more: false`. This confirmed the pagination fields
`has_more`, `last_id` and the request parameter `after_id`.

The error responses and `malformed.txt` are still written by hand from the documented shapes.
