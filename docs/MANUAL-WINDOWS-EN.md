# Angel Ghost on Windows · user manual

> **Your own experience, in the meeting, at the right moment — and only you see it.**
>
> This manual is for anyone using Angel Ghost on a Windows PC and nothing else: you do not need to have seen the app on any other system. Every screen and button below uses the exact English label you see. What this manual calls **unverified** was built and tested by the continuous integration, but nobody has run it on your PC in a real meeting yet: it goes to the test gate (`docs/GUIA-DE-PRUEBA-WINDOWS.html`).

## What it is

A small window on your PC that the client cannot see even if you share your screen. It listens to both sides of the video call (your microphone and what plays on the PC), reads the meeting screen only when it changes, and puts **evidence cards from your own corpus** in front of you: a headline, a line and the exact source. If you turn it on, a model drafts a short suggestion. Before the meeting you can rehearse with the questions that client will probably ask.

What sets it apart: **it does not persist what it should not, and you can check that.** Audio, the client’s transcript and what is read from the screen live in memory only and die on close. What stays is yours, encrypted. The app hides nothing from you and evades nothing: it asks Windows to keep its band out of screen captures, and says in plain words where that is not verified.

## What you need

- **Windows 10 version 2004 (build 19041) or later**, on an x64 or ARM64 PC (for example a Surface with Snapdragon). On an older Windows the band cannot be hidden from capture, and the app does not open it without that protection: it says so in a dialog at startup and closes.
- A **microphone** and **headphones** (for the spoken card and the rehearsal, the voice never plays through speakers).
- **Teams, Zoom or Meet** (in Chrome, Edge, Firefox or Brave) for video calls.
- About **1.6 GB free** for the speech model (0.5 GB on an ARM64 PC), plus 6 to 9 MB for the installer.
- Optional: **Windows Hello** set up (PIN, fingerprint or face) to open what is encrypted, and a **Groq or Claude key** only if you want the drafted suggestion.

## Installing

1. **Which file.** The repository’s continuous integration publishes two packages. On an x64 PC, `angel-ghost-windows`; on a Surface or any Windows ARM, `angel-ghost-windows-arm64` (it is native: the x64 one would run emulated). Each holds an `.exe` (for your user, no administrator) and an `.msi` (per machine, asks for administrator). **Install only one of the two**: they share an identifier.
2. **The Windows warning.** Because the installer is not signed, SmartScreen warns: on your English Windows it says “Windows protected your PC”. Click **More info** and then **Run anyway**. If your account is not an administrator and you choose the `.msi`, ask whoever manages the PC to install it.
3. **Where it ends up.** The `.exe` goes to `%LOCALAPPDATA%\Angel Ghost`. The `.msi` goes per machine, to `C:\Program Files\Angel Ghost` on a clean PC; **if the `.exe` was installed before, it reuses that folder** (seen on the reference Surface). Your data lives apart, in `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\`: the models, your preferences, your dictionary and the corpus index.
4. **Switching from `.exe` to `.msi` (or back).** Uninstall first: **Settings → Apps → Installed apps → Angel Ghost → Uninstall**. If the uninstaller offers **Delete the application data**, leave it unticked to keep your models and your dictionary.
5. **What has been verified.** The ARM64 `.exe` and `.msi` were installed and opened on a Surface (Snapdragon X Plus) in sprints 009 and 010. The x64 installer is built and tested by the continuous integration on every change; it has not yet been installed by hand on an x64 PC.

## The first time: the speech model

The app transcribes with **whisper**, inside your PC; no audio leaves. The model does not ship inside the installer: it is downloaded once.

- In **Language**, click **Install**. On an x64 PC it downloads the large model (`large-v3-turbo`, 1.6 GB); **on an ARM64 PC it downloads the small one** (`small`, 0.5 GB), because the large one does not fit the 4-second ceiling on those machines. Before it is used its SHA-256 checksum is verified: if it does not match, it is deleted.
- **Nothing downloads with a meeting, a session (including “notes only” or in-person) or a rehearsal open**, and if one starts while it downloads, the download stops. If the connection hangs, it lets go by itself after three hours.
- When it finishes, the app measures your PC. In **Honesty**, the transcription block shows **one line per model, track by track**: your 10 s turn (the figure that decides the model) and the client’s 6 s turn. Each line says whether it fits the 4 s ceiling (✓), goes over (⚠), or still needs measuring or downloading (○).
- **Measure again** repeats the measurement: with the large model it takes about half a minute, and meanwhile the screen says so (“Measuring…”).
- The rule: transcription uses **the largest model whose track of yours fits in 4 s on your PC**; a model that has not been measured counts as fitting. On the reference Surface: `small`, 2.0 s your track and 1.6 s the client’s; `large-v3-turbo`, 9.8 s and 9.7 s. Those are the figures from the live run on that Surface; figures change from one measurement to the next, and the general user manual quotes the range of several (the small one, about 1.6 s; the large one, 8.5 to 18 s).

## A meeting, end to end

1. **Before.** In **Corpus** you point to the folder with your proposals, frameworks and cases (PDF, Word or Markdown); it is indexed on your PC. In **Session** you choose “This client”: their jurisdiction, the engagement clause and the NDA check; if the NDA forbids transcribing, you switch to “notes only”. You choose whether the band sits on top (the default, next to the camera) or at the bottom.
2. **You join the call** (Teams or Zoom when you are in a call; Meet by the tab title). Click **Start session**: your microphone and the system audio open —two tracks, no bot in the meeting— and the meeting window moves down and shrinks just enough for the band not to cover it.
3. **During.** When the client finishes a question, the band brings its card in under 4 s. The card is passive: it makes no sound and does not steal focus. If the corpus has nothing, the band gives you a **manoeuvre** (postpone with a date, ask for the data, rephrase…) instead of a mute “no result”. With headphones, **audio-only mode** (`Alt+Shift+W`) reads you the card. `Alt+Shift+H` is “help me with this”.
4. **On close.** You save your notes and agreements encrypted, with the retention you choose. What is the client’s dies. Proposals you did not decide wait for you in a **tray with a countdown** (default 3 h, ceiling 24 h).

## The band

It is the window only you see. It asks Windows to keep out of captures, and **in Teams, Zoom and Meet that is unverified**: Honesty and Session say so in those words until you test it on your PC with a real video call. Meanwhile, **share a window, not the whole screen**.

- **Docking.** The meeting window moves down and shrinks to just below the band, at the height you gave it, when the session starts. It goes back when you close the band with its ×, cut everything with `Alt+Shift+Q` or quit the app (and, if the app crashes, the next time you open it); **End session does not send it back**. Windows asks for no permission for this. If you move it by hand meanwhile, the app does not touch it. Unverified: a maximized window, another monitor, scaling at 125 % or 150 %, and Teams or Zoom refusing to shrink (then it is undone and the band floats on top).
- **Sizes.** 88 px; it grows to 200 with the handle; 44 px in audio-only mode.
- **Height.** With the band at the top, the small bar on its top edge drags down to uncover what the band hid —your browser tabs and address bar, for example—. The height is saved and the band opens there when you start the app; the app limits it to your screen, so it never leaves it or leaves the meeting without room. In **Session**, the **Band height** row says how far down you put it and **Back to the top** sticks it to the edge again. With the band at the bottom there is no height to choose.
- **Closing.** The **×** in the top right corner closes the band **without closing the app**. With a session running it does what **End session** does: it ends it, gives the docked meeting back and closes the band (its name says so: “Close the band and end the session”). `Alt+Shift+Q`, which cuts everything, closes the band too. With the band closed, **Session** says “The band is closed” and **Show the band** opens it on its edge and at its height; **Start session** brings it back too.
- **Quitting.** **Closing the main window closes the whole app**, band included. If a session was open, it is cut, and what you wrote and had not saved is saved encrypted. The band alone closes with its ×.

## The two tracks and the card

- **Who is speaking** is decided by track, never by voice: your microphone is you; what plays on the PC is the client. The app **does not separate voices**: a notification or a song that plays counts as the client’s.
- **If you change the audio output** with the session running (Bluetooth headphones, for example), the app reopens the system audio by itself, within a couple of seconds.
- **The app’s own voice does not trigger cards:** what it says in your headphones comes back through the system audio and is marked as an echo.
- **If your PC cannot keep up.** Transcription serves the two tracks one after the other and the client goes first. If more than 4 turns pile up, your oldest ones are dropped first; Honesty counts how many, without saying whose they were.
- **The screen.** It reads the meeting window **only when it changes**, with Windows’ text recognition (it needs the text language pack: **Settings → Time & language → Language & region**). `Alt+Shift+D` reads it once, now. The image lives in memory and is discarded as it is read.

## Shortcuts

They are `Alt+Shift` and a key; none uses `Ctrl+Alt`, which on many keyboards is AltGr. If another app already has a combination, Windows will not let it be registered: Honesty says so, the band warns (“N keys are not responding”) and the action works anyway from the app.

| Key           | What it does                                                   |
| ------------- | -------------------------------------------------------------- |
| `Alt+Shift+Q` | cuts everything: audio, transcript, band and filler            |
| `Alt+Shift+H` | “help me with this”                                            |
| `Alt+Shift+C` | shows or hides the transcript next to the card                 |
| `Alt+Shift+W` | turns audio-only mode on or off                                |
| `Alt+Shift+D` | reads the screen once, now                                     |
| `Alt+Shift+U` | what the radar sees on your PC                                 |
| `Alt+Shift+K` | take a note                                                    |
| `Alt+Shift+F` | pin the card                                                   |
| `Alt+Shift+Z` | save the proposal                                              |
| `Alt+Shift+X` | the band, top or bottom                                        |
| `Esc`         | silences the voice. It only exists while audio-only mode is on |

## Your corpus and the cards

You drag or point to proposals, frameworks, cases, client files and your profile. The app understands five units (proposal, framework, case, client file and your profile) and the client’s question lands in the right one. The index lives on your PC, in the clear, inside the app’s private folder. A **card** is an eight-word headline, a line and the exact source (document and section). All of this is deterministic: it uses no model to search.

## Your notes, the tray and what expires

- **They are saved encrypted.** The key is in your Windows user’s **Credential Manager**, never in a file.
- **Opening what is encrypted asks for Windows Hello** (your PIN, fingerprint or face), once per app session: exporting a meeting, opening your rehearsals and opening a previous session’s tray. If your PC does not have it set up, the app says where to do it (**Settings → Accounts → Sign-in options**) and your notes stay encrypted.
- **The app proposes what to save**, with published rules and no model: figures, deadlines, your commitments (“I’ll send it”, “we agreed”), what clashes with a card you pinned. Proposing is not saving: nothing goes in without your yes, and the client’s turn is never saved, only a one-line fact.
- **What expires is deleted with the app closed.** The app leaves a task in **Task Scheduler** called `com.aiapps.copiloto-consultor.vencimiento`, in the root library, that runs at the minute of each expiry even if you do not open the app (if the PC was off, it runs when it comes back). What runs is PowerShell over a list from the app: it only deletes expired `.ghost` files, writes nothing and does not launch the app. When nothing expires any more, the task removes itself. If a company policy blocks PowerShell, Honesty says it did not run.
- **Backups.** If you back up your PC, those backups keep what was deleted until they expire: the app does not control where they go.

## In-person mode

At the table with the client, or next to the PC where a video call runs, the app listens to **the room through the PC’s microphone**: a single track, no system audio, no docking. In the room nobody has an owner: the transcript says “room”, never “client” or “you”. If the video call is on this same PC, use **“call on this PC”**, not in-person: that way it hears the client through the system audio. If your default audio input is a Bluetooth headset, the app would mostly hear your voice: Session warns you and says where to change it (**Settings → System → Sound → Input**).

## Rehearsal

Before the meeting, the app asks you the questions that client will probably ask, derived from your proposal by published rules and a catalogue of objections. **Windows’ voice reads them** (with headphones; it needs a voice for the language: **Settings → Time & language → Speech**); you answer aloud and, at the end, you see which evidence from your corpus you cited and which you missed, how long you took and which filler words you repeated. What is saved is yours (your text and figures, never your audio) and is encrypted.

## The suggestion (optional)

On Windows there is no system model: the suggestion is drafted by the **API** you choose in **AI**, Groq or Claude, **with your own key**, stored in Credential Manager. It starts off; when on, only minimized, anonymized text leaves, under the retention each provider publishes and the app shows where you choose. Every request is counted in Honesty. Without the API, the card is still the answer.

## The radar

It looks at the programs on **this PC** against a catalogue with a source and a date per row: exam proctoring, employee monitoring, remote access (TeamViewer, AnyDesk, Quick Assist) and whether your company manages the PC. It warns; it does not block or close anything, and it looks at nothing outside your machine. `Alt+Shift+U` shows what it sees.

## Honesty, Permissions and Language

- **Honesty** is the screen that does not lie: how many bytes left your machine in this meeting (zero in local mode), what lives in memory right now, what persists and for how long, the **“On Windows today”** block (what the app does here and what it does not, each with its reason) and your PC’s transcription.
- **Permissions.** Only the **microphone** has a permission on Windows, and you grant it in **Settings → Privacy & security → Microphone**; the **Open Windows Settings** button takes you there. System audio, the meeting screen and docking ask for none.
- **Language.** Two tracks, one language for each (Spanish and English), the speech model and your **technical dictionary**: terms that correct the transcript (Power BI, DAX, client names) in `diccionario.yaml`, which the app never completes or guesses.

## What Windows does not do for now

- **The door for Claude Code** (operating the app from a local command) **is not on Windows for now**; Honesty says so. The AI screen does not offer it.
- **Invisibility in Teams, Zoom and Meet is unverified**: the promise is graded by platform, and only what your live run verified counts.
- **Windows older than 2004** cannot hide the band: the app does not open.

## The 30 features of the vision, on Windows

Each row says what state it has on Windows and which “On Windows today” key covers it. **done** = built and checked end to end on a Windows PC; **done, unverified** = built and tested in the continuous integration, but what needs a real video call, your microphone or your Windows user in front has not been run by anyone on your PC yet; **out** = it does not exist on Windows, with its reason.

| N   | Feature                                                                  | Key                                                            | State            | What is missing or why                                                                                         |
| --- | ------------------------------------------------------------------------ | -------------------------------------------------------------- | ---------------- | -------------------------------------------------------------------------------------------------------------- |
| 1   | Private band protected from capture                                      | `banda-protegida` · `acople`                                   | done, unverified | The protection is requested from Windows; Teams, Zoom and Meet, the band height and × are verified at the gate |
| 2   | Permissions, detected meeting, status and panic button                   | `videollamada` · `atajos`                                      | done, unverified | The microphone is granted in Settings; `Alt+Shift+Q` cuts everything                                           |
| 3   | Network-egress counter and honesty panel                                 | no key                                                         | done             | Honesty counts the bytes (0 B in local mode) and says what lives in memory                                     |
| 4   | The app on the phone, in in-person mode                                  | no key                                                         | out              | It belongs to another platform: it is not a Windows feature                                                    |
| 5   | Windows                                                                  | no key                                                         | done, unverified | x64 and ARM64, with unsigned `.exe` and `.msi`; the ARM64 one was installed and opened on a Surface            |
| 6   | Two audio tracks, no bot                                                 | `videollamada`                                                 | done, unverified | Your microphone and the system audio; a real video call is missing                                             |
| 7   | Local transcription with end of turn                                     | `videollamada`                                                 | done, unverified | whisper on your PC, measured with the kit’s audio; real voice is missing                                       |
| 8   | Screen reading only when it changes                                      | `pantalla`                                                     | done, unverified | Windows text recognition; a real presentation is missing                                                       |
| 9   | Spanish and English, with technical vocabulary                           | no key                                                         | done, unverified | Interface and dictionary, checked in both languages; English transcription is missing with a real voice        |
| 10  | Several voices on the same channel                                       | no key                                                         | out              | Pending in the vision; also, the app does not separate voices by design                                        |
| 11  | No headphones (echo cancellation)                                        | no key                                                         | out              | Pending in the vision; with speakers the spoken card does not speak                                            |
| 12  | Native streaming transcription                                           | no key                                                         | out              | Pending in the vision; Windows’ engine transcribes by turns                                                    |
| 13  | Reading the client’s dashboard as a consultant                           | no key                                                         | out              | Pending in the vision                                                                                          |
| 14  | Your corpus indexed on your PC                                           | `corpus-y-ficha`                                               | done, unverified | PDF, Word and Markdown; your real corpus is missing                                                            |
| 15  | Consulting model                                                         | `corpus-y-ficha`                                               | done, unverified | The five units; your real corpus is missing                                                                    |
| 16  | Knows when to help you                                                   | `corpus-y-ficha` · `atajos`                                    | done, unverified | Question, term, figure or silence, and `Alt+Shift+H`                                                           |
| 17  | Evidence cards                                                           | `corpus-y-ficha`                                               | done, unverified | Headline, line and exact source                                                                                |
| 18  | Manoeuvre when the corpus has nothing                                    | `corpus-y-ficha`                                               | done, unverified | Catalogue of six manoeuvres, no model                                                                          |
| 19  | Audio-only mode (voice in your ear)                                      | `voz-al-oido`                                                  | done, unverified | Windows’ voice, with headphones; through speakers it does not speak                                            |
| 20  | Synthesized suggestion                                                   | `sugerencia`                                                   | done, unverified | Only with the API and your key: Windows has no system model                                                    |
| 21  | What is yours stays, what is the client’s dies: encrypted notes and tray | `notas-cifradas` · `desbloqueo` · `borrado-con-la-app-cerrada` | done, unverified | Credential Manager, Windows Hello and Task Scheduler; your PC is missing                                       |
| 22  | Detected agreements                                                      | `notas-cifradas`                                               | done, unverified | The app proposes and you decide; nothing goes in without your yes                                              |
| 23  | Guided practice                                                          | `ensayo`                                                       | done, unverified | Windows’ voice reads the questions; hearing it on your PC is missing                                           |
| 24  | In-person mode                                                           | `presencial`                                                   | done, unverified | The room through the PC’s microphone; a real room is missing                                                   |
| 25  | Verifiable ephemerality                                                  | no key                                                         | done, unverified | The sweep and the tests run in the continuous integration; a full session on your PC belongs to the gate       |
| 26  | Jurisdiction flag, engagement clause and NDA check                       | no key                                                         | done, unverified | Session screen; a test walks it, your real use is missing                                                      |
| 27  | Measured from day one                                                    | no key                                                         | done, unverified | Synthetic kit in the continuous integration; Honesty measures your PC’s figure                                 |
| 28  | Local door for Claude Code                                               | `puerta-local`                                                 | out              | Not on Windows for now (ADR 022)                                                                               |
| 29  | Transparent mode                                                         | no key                                                         | out              | It does not exist yet on any platform                                                                          |
| 30  | Capture radar                                                            | `radar`                                                        | done, unverified | Windows catalogue with a source per row; what your PC has installed is missing                                 |

## Frequently asked questions

- **The band stayed on screen and I do not know how to close it.** Press its **×**: it closes and the app keeps running; **Show the band**, in Session, brings it back. To quit altogether, close the main window: the app quits entirely. If you cannot see it, open it from the taskbar, or end **Angel Ghost** in Task Manager (**Task Manager → Processes → End task**).
- **The band covers my browser tabs.** Drag the small bar on its top edge down until you can see them; it stays saved. **Back to the top**, in Session, sticks it to the edge again.
- **“Install” downloads nothing.** A meeting, a session or a rehearsal is open, or the connection fails: the screen says so (“The model couldn't be downloaded”).
- **Honesty says “not measured”.** The model is downloaded but has not been measured this session: click **Measure again**.
- **A shortcut does not respond.** Another app on your PC already has that combination; Honesty names it. The action works anyway from the app.
- **The spoken card does not speak.** Through speakers, or through an HDMI or S/PDIF output, it does not speak: the client would hear it. With headphones it does; and it needs a voice for the language installed.
- **Can the data folder be moved?** No: it is the app’s private folder (`%LOCALAPPDATA%\com.aiapps.copiloto-consultor\`).

## Building it yourself

The `README.md` has “Developing on Windows ARM”. The script `scripts/entorno-windows.ps1` says what is missing without installing anything; tests run with `AG_SIN_HARDWARE=1 cargo test --locked -j 4` (the sentinel keeps them away from the microphone and permissions; `-j 4` keeps the linker from running out of memory) and the installer comes from `pnpm tauri build --bundles nsis`.
