#!/usr/bin/env bash
# Registers two throwaway accounts on the local Conduit instance (started via
# docker-compose.yml in this directory) and creates a plain, unencrypted room
# with both accounts joined. Safe to re-run: falls back to logging in if an
# account already exists.
set -euo pipefail

HOMESERVER="http://localhost:6167"
SERVER_NAME="localhost:6167"
ACCT1_USER="acct1"
ACCT1_PASS="testpass1"
ACCT2_USER="acct2"
ACCT2_PASS="testpass2"
ROOM_NAME="rust-chat test room"

register_or_login() {
  local username="$1" password="$2"
  local session
  session=$(curl -s -X POST "$HOMESERVER/_matrix/client/v3/register" \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$username\", \"password\": \"$password\"}" \
    | python3 -c "import json,sys; print(json.load(sys.stdin).get('session',''))")

  if [ -z "$session" ]; then
    echo "  ($username already exists, logging in instead)" >&2
    curl -s -X POST "$HOMESERVER/_matrix/client/v3/login" \
      -H "Content-Type: application/json" \
      -d "{\"type\": \"m.login.password\", \"identifier\": {\"type\": \"m.id.user\", \"user\": \"$username\"}, \"password\": \"$password\"}" \
      | python3 -c "import json,sys; print(json.load(sys.stdin)['access_token'])"
    return
  fi

  curl -s -X POST "$HOMESERVER/_matrix/client/v3/register" \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$username\", \"password\": \"$password\", \"auth\": {\"type\": \"m.login.dummy\", \"session\": \"$session\"}}" \
    | python3 -c "import json,sys; print(json.load(sys.stdin)['access_token'])"
}

echo "waiting for conduit to be reachable at $HOMESERVER..." >&2
for _ in $(seq 1 30); do
  if curl -sf "$HOMESERVER/_matrix/client/versions" > /dev/null 2>&1; then
    break
  fi
  sleep 1
done
if ! curl -sf "$HOMESERVER/_matrix/client/versions" > /dev/null 2>&1; then
  echo "conduit never became reachable - is 'docker compose up -d' running in this directory?" >&2
  exit 1
fi

echo "registering $ACCT1_USER..." >&2
ACCT1_TOKEN=$(register_or_login "$ACCT1_USER" "$ACCT1_PASS")

echo "registering $ACCT2_USER..." >&2
ACCT2_TOKEN=$(register_or_login "$ACCT2_USER" "$ACCT2_PASS")

echo "creating test room..." >&2
ROOM_ID=$(curl -s -X POST "$HOMESERVER/_matrix/client/v3/createRoom" \
  -H "Authorization: Bearer $ACCT1_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\": \"$ROOM_NAME\", \"visibility\": \"private\", \"invite\": [\"@$ACCT2_USER:$SERVER_NAME\"]}" \
  | python3 -c "import json,sys; print(json.load(sys.stdin)['room_id'])")

ROOM_ID_ENC=$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=''))" "$ROOM_ID")

echo "joining $ACCT2_USER to the room..." >&2
curl -s -X POST "$HOMESERVER/_matrix/client/v3/join/$ROOM_ID_ENC" \
  -H "Authorization: Bearer $ACCT2_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{}' > /dev/null

cat <<SUMMARY

Ready. Test fixtures:

  Account 1 (use this one from rust-chat):
    user-id:  $ACCT1_USER
    password: $ACCT1_PASS

  Account 2 (use this one to act as "the other person", e.g. via curl
  or a real client like Element pointed at $HOMESERVER):
    user-id:  $ACCT2_USER
    password: $ACCT2_PASS

  Test room: $ROOM_ID

Run rust-chat with:
  cargo run -- matrix --homeserver $SERVER_NAME --user-id $ACCT1_USER --password $ACCT1_PASS --insecure

Then inside rust-chat:
  /join $ROOM_ID
SUMMARY
