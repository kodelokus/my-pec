//! Server-sent events from the Messages API and the assembly of content blocks (chat R2, R3).

use serde_json::{Map, Value};

/// Splits a byte stream into `data` payloads, carrying a partial event across chunks.
#[derive(Default)]
pub(super) struct Events {
    buffer: Vec<u8>,
}

impl Events {
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Value> {
        self.buffer
            .extend(chunk.iter().copied().filter(|b| *b != b'\r'));
        let mut out = Vec::new();
        while let Some(end) = self.buffer.windows(2).position(|w| w == b"\n\n") {
            let event: Vec<u8> = self.buffer.drain(..end + 2).collect();
            let data: String = String::from_utf8_lossy(&event)
                .lines()
                .filter_map(|line| line.strip_prefix("data:"))
                .map(str::trim_start)
                .collect::<Vec<_>>()
                .join("\n");
            if let Ok(value) = serde_json::from_str(&data) {
                out.push(value);
            }
        }
        out
    }
}

/// What one event means to the caller.
#[derive(Debug, PartialEq)]
pub(super) enum Step {
    None,
    Text(String),
    /// An `error` event, with the API's error type.
    Error(String),
    Done,
}

/// The reply being assembled from events. A block marked `_unmerged` is never replayed.
#[derive(Default)]
pub(super) struct Assembly {
    pub blocks: Vec<Value>,
    json: Vec<(usize, String)>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Option<Value>,
}

impl Assembly {
    pub fn apply(&mut self, event: &Value) -> Step {
        let index = event["index"].as_u64().map(|i| i as usize);
        match event["type"].as_str().unwrap_or_default() {
            "message_start" => {
                let message = &event["message"];
                self.model = message["model"].as_str().unwrap_or_default().into();
                self.merge_usage(&message["usage"]);
            }
            "content_block_start" => {
                if let Some(i) = index {
                    if self.blocks.len() <= i {
                        self.blocks.resize(i + 1, Value::Null);
                    }
                    self.blocks[i] = event["content_block"].clone();
                }
            }
            "content_block_delta" => {
                if let Some(i) = index.filter(|i| *i < self.blocks.len()) {
                    return self.merge(i, &event["delta"]);
                }
            }
            "content_block_stop" => {
                if let Some(i) = index {
                    self.finish(i);
                }
            }
            "message_delta" => {
                if let Some(reason) = event["delta"]["stop_reason"].as_str() {
                    self.stop_reason = Some(reason.into());
                }
                self.merge_usage(&event["usage"]);
            }
            "message_stop" => return Step::Done,
            "error" => {
                return Step::Error(event["error"]["type"].as_str().unwrap_or_default().into());
            }
            _ => {}
        }
        Step::None
    }

    fn merge(&mut self, i: usize, delta: &Value) -> Step {
        let Some(block) = self.blocks[i].as_object_mut() else {
            return Step::None;
        };
        let kind = delta["type"].as_str().unwrap_or_default();
        match kind {
            "input_json_delta" => {
                let part = delta["partial_json"].as_str().unwrap_or_default();
                match self.json.iter_mut().find(|(j, _)| *j == i) {
                    Some((_, text)) => text.push_str(part),
                    None => self.json.push((i, part.into())),
                }
                return Step::None;
            }
            "citations_delta" => {
                push(block, "citations", delta["citation"].clone());
                return Step::None;
            }
            _ => {}
        }
        let fields: Vec<(&String, &Value)> = delta
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(name, _)| *name != "type")
            .collect();
        if fields.is_empty() || !fields.iter().all(|(_, v)| v.is_string()) {
            push(block, "_unmerged", delta.clone());
            return Step::None;
        }
        for (name, value) in fields {
            let piece = value.as_str().unwrap_or_default();
            match block.get_mut(name.as_str()) {
                Some(Value::String(s)) => s.push_str(piece),
                _ => {
                    block.insert(name.clone(), Value::String(piece.into()));
                }
            }
        }
        if kind == "text_delta" {
            Step::Text(delta["text"].as_str().unwrap_or_default().into())
        } else {
            Step::None
        }
    }

    fn finish(&mut self, i: usize) {
        let Some(pos) = self.json.iter().position(|(j, _)| *j == i) else {
            return;
        };
        let (_, text) = self.json.swap_remove(pos);
        let Some(block) = self.blocks.get_mut(i).and_then(Value::as_object_mut) else {
            return;
        };
        let input = if text.is_empty() {
            Ok(Value::Object(Map::new()))
        } else {
            serde_json::from_str(&text)
        };
        match input {
            Ok(input) => {
                block.insert("input".into(), input);
            }
            Err(_) => push(block, "_unmerged", Value::String(text)),
        }
    }

    fn merge_usage(&mut self, usage: &Value) {
        let Some(fields) = usage.as_object() else {
            return;
        };
        let target = self.usage.get_or_insert_with(|| Value::Object(Map::new()));
        if let Some(target) = target.as_object_mut() {
            for (name, value) in fields {
                if !value.is_null() {
                    target.insert(name.clone(), value.clone());
                }
            }
        }
    }
}

fn push(block: &mut Map<String, Value>, field: &str, value: Value) {
    match block.get_mut(field) {
        Some(Value::Array(items)) => items.push(value),
        _ => {
            block.insert(field.into(), Value::Array(vec![value]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn events(chunks: &[&str]) -> Vec<Value> {
        let mut parser = Events::default();
        chunks
            .iter()
            .flat_map(|c| parser.push(c.as_bytes()))
            .collect()
    }

    #[test]
    fn events_are_read_across_chunk_boundaries() {
        let a = "event: ping\ndata: {\"type\":\"ping\"}\n\nevent: content_block_delta\r\ndata: {\"type\":";
        let b = "\"message_stop\"}\r\n\r\n";
        assert_eq!(
            events(&[a, b]),
            [json!({"type": "ping"}), json!({"type": "message_stop"})]
        );
        assert_eq!(
            events(&["data: {\"type\":\"ping\"}\n"]),
            Vec::<Value>::new()
        );
    }

    fn assemble(events: &[Value]) -> (Assembly, Vec<Step>) {
        let mut assembly = Assembly::default();
        let steps = events.iter().map(|e| assembly.apply(e)).collect();
        (assembly, steps)
    }

    fn start(index: u64, block: Value) -> Value {
        json!({"type": "content_block_start", "index": index, "content_block": block})
    }

    fn delta(index: u64, delta: Value) -> Value {
        json!({"type": "content_block_delta", "index": index, "delta": delta})
    }

    fn stop(index: u64) -> Value {
        json!({"type": "content_block_stop", "index": index})
    }

    #[test]
    fn each_delta_kind_is_merged_into_its_block() {
        let (assembly, steps) = assemble(&[
            start(
                0,
                json!({"type": "thinking", "thinking": "", "signature": ""}),
            ),
            delta(0, json!({"type": "thinking_delta", "thinking": "Hm"})),
            delta(0, json!({"type": "signature_delta", "signature": "ab"})),
            delta(0, json!({"type": "signature_delta", "signature": "cd"})),
            stop(0),
            start(1, json!({"type": "text", "text": ""})),
            delta(1, json!({"type": "text_delta", "text": "Hi"})),
            delta(
                1,
                json!({"type": "citations_delta", "citation": {"cited_text": "x"}}),
            ),
            stop(1),
            start(
                2,
                json!({"type": "tool_use", "id": "t1", "name": "n", "input": {}}),
            ),
            delta(
                2,
                json!({"type": "input_json_delta", "partial_json": "{\"q\":"}),
            ),
            delta(2, json!({"type": "input_json_delta", "partial_json": "1}"})),
            stop(2),
            start(3, json!({"type": "novel", "value": ""})),
            delta(3, json!({"type": "novel_delta", "count": 3})),
            stop(3),
            json!({"type": "error", "error": {"type": "overloaded_error"}}),
        ]);
        assert_eq!(
            assembly.blocks,
            [
                json!({"type": "thinking", "thinking": "Hm", "signature": "abcd"}),
                json!({"type": "text", "text": "Hi", "citations": [{"cited_text": "x"}]}),
                json!({"type": "tool_use", "id": "t1", "name": "n", "input": {"q": 1}}),
                json!({"type": "novel", "value": "", "_unmerged": [{"type": "novel_delta", "count": 3}]}),
            ]
        );
        assert_eq!(steps[6], Step::Text("Hi".into()));
        assert_eq!(steps[16], Step::Error("overloaded_error".into()));
    }

    #[test]
    fn usage_and_stop_reason_come_from_start_and_delta() {
        let (assembly, steps) = assemble(&[
            json!({"type": "message_start", "message": {"model": "m", "usage": {"input_tokens": 9, "output_tokens": 1}}}),
            json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 4}}),
            json!({"type": "message_stop"}),
        ]);
        assert_eq!(assembly.model, "m");
        assert_eq!(assembly.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(
            assembly.usage,
            Some(json!({"input_tokens": 9, "output_tokens": 4}))
        );
        assert_eq!(steps[2], Step::Done);
    }
}
