#!/usr/bin/env bun
/**
 * Simple end-to-end chat smoke test against the running backend.
 * - Fetches a dev token (or uses SWITCHBOARD_SMOKE_TOKEN)
 * - Sends a prompt to /api/chat with a small OSS model
 * - Prints the reply and exits non-zero on failure
 *
 * Run with: `bun run scripts/chat-smoke.ts`
 *
 * Environment:
 *   SWITCHBOARD_SMOKE_API_BASE  (default: http://localhost:7070)
 *   SWITCHBOARD_SMOKE_MODEL     (default: meta-llama/Meta-Llama-3.1-8B-Instruct)
 *   SWITCHBOARD_SMOKE_PROMPT    (default: "Hello from smoke test")
 *   SWITCHBOARD_SMOKE_TOKEN     (optional: bearer token to skip dev token fetch)
 */

const API_BASE = process.env.SWITCHBOARD_SMOKE_API_BASE ?? "http://localhost:7070";
const MODEL_ID =
  process.env.SWITCHBOARD_SMOKE_MODEL ?? "meta-llama/Meta-Llama-3.1-8B-Instruct";
const PROMPT = process.env.SWITCHBOARD_SMOKE_PROMPT ?? "Hello from smoke test";

const log = (...args: unknown[]) => console.log("==>", ...args);

const fatal = (message: string, error?: unknown) => {
  console.error("ERROR:", message);
  if (error) {
    console.error(error);
  }
  process.exit(1);
};

const getToken = async (): Promise<string> => {
  if (process.env.SWITCHBOARD_SMOKE_TOKEN) {
    return process.env.SWITCHBOARD_SMOKE_TOKEN;
  }

  log("Requesting dev token…");
  const res = await fetch(`${API_BASE}/api/auth/dev/token`);
  if (!res.ok) {
    fatal(`Dev token request failed: ${res.status} ${res.statusText}`);
  }
  const body = await res.json().catch(() => null);
  if (!body?.token) {
    fatal("Dev token response missing token field", body);
  }
  return body.token as string;
};

const main = async () => {
  log(`Running chat smoke against ${API_BASE} with model "${MODEL_ID}"`);

  const token = await getToken();
  log(`Got token ${token.slice(0, 8)}…`);

  const form = new FormData();
  form.append("prompt", PROMPT);
  form.append("model", MODEL_ID);

  log("Sending prompt…");
  const res = await fetch(`${API_BASE}/api/chat`, {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: form,
  });

  const text = await res.text();
  if (!res.ok) {
    fatal(`Chat request failed: ${res.status} ${res.statusText}`, text);
  }

  let data: any;
  try {
    data = JSON.parse(text);
  } catch (err) {
    fatal("Chat response was not valid JSON", { text, err });
  }

  const content = data?.content ?? data?.message ?? "<no content>";
  log("Assistant reply:");
  console.log(content);
  process.exit(0);
};

main().catch((err) => fatal("Unexpected error", err));
