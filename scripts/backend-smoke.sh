#!/usr/bin/env bash
# Simple backend smoke test: health check, dev token, create folder + chat.

set -euo pipefail

API_BASE=${API_BASE:-http://localhost:3030}

log() {
  printf '==> %s\n' "$*"
}

request() {
  local method=$1
  local path=$2
  local data=${3-}
  local auth_header=${4-}

  local url="${API_BASE}${path}"
  local response http_code

  if [[ -n "$data" ]]; then
    response=$(curl -s -w "\n%{http_code}" -H "Content-Type: application/json" ${auth_header:+-H "$auth_header"} -X "$method" "$url" -d "$data")
  else
    response=$(curl -s -w "\n%{http_code}" ${auth_header:+-H "$auth_header"} -X "$method" "$url")
  fi

  http_code=$(tail -n1 <<<"$response")
  body=$(sed '$d' <<<"$response")

  echo "$body"
  echo "$http_code"
}

log "Checking health at ${API_BASE}/api/health"
read -r body status < <(request GET "/api/health")
if [[ "$status" != "200" ]]; then
  echo "Health check failed ($status): $body" >&2
  exit 1
fi
echo "Health OK: $body"

log "Requesting dev token"
dev_body=$(curl -s -H "Content-Type: application/json" "${API_BASE}/api/auth/dev/token")
token=$(python -c 'import json,sys; data=json.load(sys.stdin); print(data.get("token",""))' <<<"$dev_body")
if [[ -z "$token" ]]; then
  echo "Failed to parse dev token from response: $dev_body" >&2
  exit 1
fi
auth_header="Authorization: Bearer ${token}"
echo "Got token: ${token:0:8}…"

log "Creating folder"
read -r folder_body folder_status < <(request POST "/api/folders" '{"name":"Smoke","color":"#00AAFF"}' "$auth_header")
if [[ "$folder_status" != "200" && "$folder_status" != "201" ]]; then
  echo "Create folder failed ($folder_status): $folder_body" >&2
  exit 1
fi
folder_id=$(python -c 'import json,sys; data=json.load(sys.stdin); print(data["folder"]["public_id"])' <<<"$folder_body")
echo "Folder created: $folder_id"

log "Creating chat"
chat_payload=$(cat <<EOF
{
  "title": "Smoke Chat",
  "messages": [{"role":"user","content":"Ping"}],
  "folder_id": "$folder_id",
  "chat_type": "direct"
}
EOF
)
read -r chat_body chat_status < <(request POST "/api/chats" "$chat_payload" "$auth_header")
if [[ "$chat_status" != "200" && "$chat_status" != "201" ]]; then
  echo "Create chat failed ($chat_status): $chat_body" >&2
  exit 1
fi
chat_id=$(python -c 'import json,sys; data=json.load(sys.stdin); print(data["chat"]["public_id"])' <<<"$chat_body")
echo "Chat created: $chat_id"

log "Listing models"
read -r models_body models_status < <(request GET "/api/models" "" "$auth_header")
if [[ "$models_status" != "200" ]]; then
  echo "List models failed ($models_status): $models_body" >&2
  exit 1
fi
echo "Models OK (truncated): $(echo "$models_body" | head -n 2)"

log "Smoke test passed."
