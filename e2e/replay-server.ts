// Serves the recorded responses to the app under test, so no run reaches the real API.
// The key decides the model list's answer: a key containing "rejected" or "nocredit" gets that failure.
// Chat replies replay `reply.sse`, numbered so a regenerated reply differs. The first request whose newest message
// contains "slow" streams one delta and then holds the connection, so it can be stopped. A newest message containing
// "remember" gets `tool-use.sse` (a `memory_remember` call), and the tool results get `after-tools.sse`.
// A request with a structured-output format is a compaction summary and gets `summary.sse`; a newest message
// containing "make it long" gets `reply-large.sse`, which reports a context past the compaction threshold.
// Dreaming: the REM themes get `themes.sse`, the deep decisions `decisions.sse`, the diary `diary.sse`.
// The world journey: the day's topics get `topics.sse`, dated the newest day asked about.
// The avatar: a newest message containing "good news" gets `reply-mood.sse`, which opens with a mood marker.
// Weather: the place lookup gets `search-jakarta.json` and the weather `current-rain.json`.
// Extensions: a newest message containing "remind me" gets `tool-use-alarm.sse`, an `alarm_set` call.
// POST /__revoke makes every later chat request fail with 401, as a key revoked after setup would, until
// POST /__reset, which each spec's session sends first.

import { readFileSync } from "node:fs";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { join } from "node:path";

const fixtures = join(import.meta.dirname, "../crates/pec-core/tests/fixtures");
const fixture = (path: string) => readFileSync(join(fixtures, path));
// Weather: the recorded Open-Meteo answers, for the requests an e2e build sends here as `/<host><path>`.
const weather = (name: string) =>
  readFileSync(join(import.meta.dirname, "../crates/pec-ext/src/weather/fixtures", `${name}.json`));

const answers: [RegExp, number, string][] = [
  [/rejected/, 401, "models/401.json"],
  [/nocredit/, 402, "models/402.json"],
  [/.*/, 200, "models/ok.json"],
];

let replies = 0;
let revoked = false;
let slowServed = false;

/** The text of the newest message in a chat request. */
function newest(body: string): string {
  const messages: { content: { text?: string }[] }[] = JSON.parse(body).messages ?? [];
  return (messages.at(-1)?.content ?? []).map((block) => block.text ?? "").join(" ");
}

/** Whether the newest message in a chat request answers tool calls. */
function answersTools(body: string): boolean {
  const messages: { content: { type?: string }[] }[] = JSON.parse(body).messages ?? [];
  return (messages.at(-1)?.content ?? []).some((block) => block.type === "tool_result");
}

function chat(body: string, res: ServerResponse) {
  if (revoked) {
    res.writeHead(401, { "content-type": "application/json" }).end(fixture("models/401.json"));
    return;
  }
  res.writeHead(200, { "content-type": "text/event-stream" });
  const request = JSON.parse(body);
  const schema = request.output_config?.format?.schema?.properties ?? {};
  if (schema.themes) {
    res.end(fixture("messages/themes.sse"));
    return;
  }
  if (schema.operations) {
    res.end(fixture("messages/decisions.sse"));
    return;
  }
  if (schema.days) {
    // The newest day asked about takes the fixture's date, so the answer covers it.
    const day = newest(body).match(/## (\d{4}-\d{2}-\d{2})/)?.[1] ?? "2026-11-10";
    res.end(fixture("messages/topics.sse").toString().replaceAll("2026-11-10", day));
    return;
  }
  if (request.output_config?.format) {
    res.end(fixture("messages/summary.sse"));
    return;
  }
  if (JSON.stringify(request.system ?? []).includes("dream diary")) {
    res.end(fixture("messages/diary.sse"));
    return;
  }
  if (newest(body).includes("good news")) {
    res.end(fixture("messages/reply-mood.sse"));
    return;
  }
  if (newest(body).includes("make it long")) {
    res.end(fixture("messages/reply-large.sse"));
    return;
  }
  if (answersTools(body)) {
    res.end(fixture("messages/after-tools.sse"));
    return;
  }
  if (newest(body).includes("remind me")) {
    res.end(fixture("messages/tool-use-alarm.sse"));
    return;
  }
  if (newest(body).includes("remember")) {
    res.end(fixture("messages/tool-use.sse"));
    return;
  }
  if (!slowServed && newest(body).includes("slow")) {
    slowServed = true;
    const events = fixture("messages/stream-error.sse").toString().split("\n\n");
    // Everything before the error event: the start, one block and one delta. Then silence.
    res.write(`${events.slice(0, 3).join("\n\n")}\n\n`);
    return;
  }
  replies += 1;
  res.end(fixture("messages/reply.sse").toString().replace('", Sam!"', `", Sam! (${replies})"`));
}

function handle(req: IncomingMessage, res: ServerResponse, body: string) {
  const key = String(req.headers["x-api-key"] ?? "");
  const versioned = req.headers["anthropic-version"] === "2026-10-21";
  if (req.method === "POST" && req.url === "/__revoke") {
    revoked = true;
    res.writeHead(204).end();
  } else if (req.method === "POST" && req.url === "/__reset") {
    revoked = false;
    slowServed = false;
    res.writeHead(204).end();
  } else if (req.method === "GET" && req.url?.startsWith("/geocoding-api.open-meteo.com/v1/search?")) {
    res.writeHead(200, { "content-type": "application/json" }).end(weather("search-jakarta"));
  } else if (req.method === "GET" && req.url?.startsWith("/api.open-meteo.com/v1/forecast?")) {
    res.writeHead(200, { "content-type": "application/json" }).end(weather("current-rain"));
  } else if (!versioned) {
    res.writeHead(400).end();
  } else if (req.method === "POST" && req.url === "/v1/messages") {
    chat(body, res);
  } else if (req.url?.startsWith("/v1/models")) {
    const [, status, file] = answers.find(([pattern]) => pattern.test(key)) ?? answers[2];
    res.writeHead(status, { "content-type": "application/json" }).end(fixture(file));
  } else {
    res.writeHead(404).end();
  }
}

export function startReplayServer(port: number): Promise<Server> {
  const server = createServer((req, res) => {
    let body = "";
    req.on("data", (chunk) => {
      body += chunk;
    });
    req.on("end", () => handle(req, res, body));
  });
  return new Promise((resolve) => server.listen(port, "127.0.0.1", () => resolve(server)));
}
