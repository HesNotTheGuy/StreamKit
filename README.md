# StreamKit

> Lightweight, second-monitor utility suite for Twitch streamers.

Fifteen tools in one Tauri desktop app — W/L tracker, timers, goal bars, soundboard, song requests, alerts, chat wall, raid log, and more. Every tool with a public-facing overlay can publish a transparent OBS browser source.

No accounts. No telemetry. No cloud. Everything runs locally; you bring your own Twitch and Spotify API keys.

## Why BYOK?

Most stream tools charge you (or sell your data) because *they* are paying the API bill. StreamKit dodges that: you register your own Twitch + Spotify dev apps in 60 seconds (the Setup tab walks you through it), paste the client IDs once, and the app talks to those services using *your* quota. Nobody else can see your tokens, no monthly fee, no rate-limit pool to share.

## Tools

| Tool | What it does |
|---|---|
| **W/L Tracker** | Wins/losses/draws counter with streaks, win-rate, custom-font overlay |
| **Timers** | Countdown / count-up / interval timers with OBS overlay |
| **Goal Tracker** | Multi-goal progress bars + overlay; goals can auto-fill from live Followers / Subs / Bits / Raids |
| **Soundboard** | Audio clips with keyboard shortcuts, per-slot volume, drag-in `.mp3`/`.wav` |
| **Spotify** | Now Playing overlay + Twitch chat `!sr` song requests with auto-queue |
| **Alerts** | Twitch EventSub alerts (sub / resub / gift / raid / cheer / follow) with custom text, image, audio |
| **Wheel** | Spin a wheel of anything — giveaways, "what game next", viewer picks — with a synced OBS overlay |
| **Chat Wall** | Pin viewer messages; cycle or scroll them on an overlay |
| **Stream Info** | Edit your stream title, category, and tags from the dashboard |
| **Raid Log** | Auto-logs incoming raids via Twitch IRC |
| **Queue** | Manage viewer queue / lobby |
| **Checklist** | Pre / mid / post-stream task lists with optional reminder intervals |
| **Minigames** | Quick polls, team battles, trivia — all with overlays |
| **Clip Notes** | Stamp clip-worthy moments while you stream |
| **Scratchpad** | Persistent autosaving notepad with heading jump |
| **Setup** | Onboarding + Twitch/Spotify configuration guide |
| **Remote Control** | Trigger StreamKit from a Stream Deck / Touch Portal / phone via local control URLs |

Every overlay tool has a "📋 Copy OBS Browser Source URL" button in its style panel — paste it into OBS as a Browser Source and you're done.

## Install

Grab the latest installer from [Releases](https://github.com/HesNotTheGuy/StreamKit/releases) and run it. Windows only for now (Tauri can cross-compile to macOS/Linux later if there's demand).

StreamKit uses port **3001** for OBS overlay URLs. If something else is already on that port, the app will refuse to start with a clear error — free the port and re-launch.

## First-run

1. Open StreamKit → it opens on your monitor of choice (window position is remembered)
2. Visit the **Setup** tab — it walks you through registering your free Twitch and Spotify dev apps
3. Pick a tool from the left sidebar (keys `1`–`9` switch between the first nine tools)
4. For overlay tools, copy the OBS URL from each tool's "🎨 Overlay Style" panel

Closing the window hides StreamKit to the system tray. Right-click the tray icon → Quit to fully exit.

## Connecting Twitch & Spotify

Both integrations are **bring-your-own-key**: you create your own free developer apps and paste the Client IDs into StreamKit. No client *secrets* are ever needed — Twitch uses implicit-grant OAuth and Spotify uses PKCE — and your access tokens live only in your browser's `localStorage`. They never touch a server, and nobody else can use your quota. The in-app **Setup** tab mirrors these steps with copy buttons.

> StreamKit serves on **port 3001**, so the redirect URLs below assume that. If you run on a different port, change the redirect URL in your dev app to match.

### Twitch

1. Open the [Twitch Developer Console](https://dev.twitch.tv/console/apps) and click **Register Your Application**.
2. Name it anything (e.g. `StreamKit`) and set **Category** to *Website Integration*.
3. Under **OAuth Redirect URLs**, add exactly:
   ```
   http://localhost:3001/src/tools/twitch/callback.html
   ```
4. Click **Create**, then copy the **Client ID** (there is no secret to copy).
5. In StreamKit, click **Connect Twitch** at the bottom of the sidebar, paste the Client ID, and authorize. Scopes requested: `user:read:email`, `clips:edit`, `channel:manage:broadcast`.

### Spotify

1. Open the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard) and click **Create app**.
2. Name it anything (e.g. `StreamKit`); under **APIs used**, select *Web API*.
3. In the app's **Settings → Redirect URIs**, add exactly, then **Save**:
   ```
   http://localhost:3001/src/tools/spotify/callback.html
   ```
4. Copy the **Client ID** from the Settings page.
5. Open the **Spotify** tool in StreamKit, paste the Client ID, and click **Connect to Spotify**. Scopes requested: `user-read-currently-playing`, `user-read-playback-state`, `user-modify-playback-state`.

> Song-request *playback* requires a Spotify **Premium** account. Reading Twitch chat for `!sr` commands is anonymous and needs no Twitch app at all.

## Remote control (Stream Deck, Touch Portal, phone…)

The installed desktop app runs a tiny local control server, so any controller that can send an HTTP request to your own machine can trigger StreamKit — no StreamKit-specific plugin required. Open the **Remote Control** tab to see your ready-to-paste URLs.

- **Stream Deck:** add a *Website* action (or the free *Web Requests* / *API Ninja* plugin for a silent request), set it to `GET` one of the URLs.
- **Touch Portal / Loupedeck / phone:** use a "send HTTP GET" action, or just bookmark the URL.
- Endpoints look like `http://localhost:3001/api/action/<action>?token=<token>` — built-in actions include `toggle_window`, `show_window`, `hide_window`, `wheel_spin`, and `tool:<id>` (jump to any tool).

The server binds to `localhost` only and every URL carries a private token (persisted across restarts), so other machines and stray web pages can't trigger your actions. The control API is part of the desktop build — it isn't available in browser-only dev mode.

## Develop

```bash
# Install dev dependencies (just Tauri CLI)
npm install

# Option A: browser-only dev (no Tauri, no Rust build)
python serve.py
# → http://localhost:3001/src/main.html

# Option B: full Tauri dev (desktop window + tray)
npm run dev

# Build NSIS + MSI installers for distribution
npm run build
```

The frontend is vanilla HTML/CSS/JS — no bundler, no framework, no build step for the frontend itself. Just open files and edit.

## Stack

- **Tauri 2** + Rust ([`src-tauri/`](src-tauri/)) — Axum file server, system tray, window-state persistence
- Vanilla HTML/CSS/JS in [`src/`](src/) — one self-contained HTML file per tool
- Shared CSS in [`assets/styles/dashboard.css`](assets/styles/dashboard.css), shared JS helpers in [`assets/js/util.js`](assets/js/util.js)
- All state lives in `localStorage`, keys prefixed `streamkit_`

## License

[MIT](LICENSE) — use it, fork it, sell your fork, embed it in your own thing; just keep the copyright notice.

**Bundled fonts:** [Bebas Neue](assets/fonts/BebasNeue-Regular.woff2) is distributed under the [SIL Open Font License 1.1](assets/fonts/OFL.txt), which is separate from the project's MIT code license.

## Support

If StreamKit makes your streams better, [sponsoring on GitHub](https://github.com/sponsors/HesNotTheGuy) keeps it free and BYOK for everyone.
