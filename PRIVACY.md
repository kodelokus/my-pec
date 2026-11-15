# Privacy

My Pec is local-first. It has no account, no server of its own and no telemetry.

## What My Pec stores, and where

- **Your API key** is kept in Windows Credential Manager. The interface hands it to the app's background process once and keeps no copy.
- **Everything else** is in `%APPDATA%\My Pec`: the companion's personality, what you told it about yourself, your conversation and its memory. These are plain files you can open, edit and delete.

Uninstalling My Pec does not delete that folder, so your companion survives a reinstall. Delete the folder, and the `claude-api-key.My Pec` entry in Credential Manager, to remove everything.

## What My Pec sends

When you talk to your companion, My Pec sends your message to the Claude API at `api.anthropic.com` over HTTPS, using your own key. To give a good reply, the request also includes the companion's personality, what you told it about yourself, recent conversation and relevant memories. Anthropic handles that data under its own terms and [privacy policy](https://www.anthropic.com/legal/privacy).

Nightly dreaming and the compaction of long conversations also send your notes and conversation to the same API, with the same key, to sort them into memory.

My Pec also contacts `api.anthropic.com` to check your key and to list the models your key can use.

If you agree to turn on search by meaning, My Pec downloads a small search model (multilingual-e5-small) once from Hugging Face (`huggingface.co`). It runs on your computer, so your memories never leave it. Nothing is downloaded unless you agree.

If you turn on Weather (it is off until you do), My Pec asks Open-Meteo for the weather where you are. When you search for your city, it sends the name you typed to `geocoding-api.open-meteo.com` to find it. Then, about every 30 minutes while My Pec runs, it sends that place's coordinates, rounded to about a kilometre, to `api.open-meteo.com` for its current weather. Nothing else is sent, and no account or key is used. Turn Weather off and it stops at once; the city you chose stays in `weather.md`.

It sends nothing anywhere else.

## Changes

A later version will add an update check. This page will say so before it ships.
