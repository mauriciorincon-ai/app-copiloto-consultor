# Angel Ghost — User manual

> **Mandatory, living document.** Every feature that reaches `main` is documented here in the same
> sprint. Written for whoever is going to use the app, in plain English — no technical jargon or references
> to the code.

## What this app is

Angel Ghost is a small window on your Mac that **the client does not see, even when you share your screen**.
It listens to both sides of the video call, searches your own documents, and when someone
asks something you have already answered in a proposal, a case or a framework of yours, it puts it in front of you:
a headline, a line and where it comes from.

It records nothing. When you close it, no trace of the audio is left, and of what was said only what you decide to
save —your notes and, from the client, at most a one-line fact— and whatever you do not decide on waits a few
hours in the tray and deletes itself; the app itself has a
screen so you can check it instead of taking its word for it.

**And at the table, too:** in _In person_ the app listens to the room through your Mac's microphone —a face-to-face meeting,
or the video call running on another machine with your Mac beside it— and gives you the card in your ear
(_In person_, below). If that video call runs on a Windows PC, the app can also be installed there
(_Angel Ghost on Windows_, below).

**And before the meeting, you can rehearse it:** the app asks you the questions that client will probably
ask, drawn from your own proposal, listens to you through the microphone and tells you which evidence of yours you used and
which you had left unused (_The rehearsal_, below).

**Who it is for:** consultants and advisors who already have their material written and need it at the exact
moment of the conversation, without having to go searching in front of the client.

## First steps

**You need macOS 26 or later.** The app uses the speech recognition and the system model that
only macOS 26 ships with; on a Mac with an earlier version it does not start.

1. **Launch the app.** Three things appear: the notebook (the large window), a landscape band
   below the menu bar, next to the camera —that is its default place; you can put it at the bottom—, and the
   backdrop that covers it when you share your screen.
2. **Grant the permissions.** Go to _Permissions_. The app takes you to the exact spot in System Settings for each one.
   **You can use it without granting any**: whatever does not work, it will tell you.
   **System audio and Screen are two different permissions**, even though System Settings shows them in the same
   pane (“Screen & System Audio Recording”): each row in _Permissions_ names its own.
3. **Point to your documents folder.** Go to _Corpus_ → “Point to a folder”. The app reads your PDF,
   Word and Markdown files **where they are** — it does not copy them or upload them anywhere.
4. **Put on your headphones** and join your meeting. In _Session_, press “Start session”.

## Features

### The protected band · since Sprint 001

- **What it does:** a strip below the menu bar (or on the bottom edge, if you prefer it at the bottom)
  where everything the app has to tell you appears. **When you share your screen, the client sees your desktop background in that
  strip**, not the band.
- **How to use it:** it appears on its own at launch. Drag the **handle** in the center, away from its edge,
  to enlarge it and see more; dragging back leaves it compact. With the band at the top, the handle is at the bottom and
  is dragged down; with the band at the bottom, up.
- **Known limitations:** invisibility is **verified in Google Meet on macOS
  26.6.2, with the band at the bottom**. With the band at the top, and in Zoom and Teams, the app has not checked it,
  and tells you (“not verified”) instead of promising it. If that worries you: share a window instead of
  the full screen, or use a second monitor.

### Docking: the meeting makes room · since Sprint 001

- **What it does:** it makes room for the band in the video call window so that it covers nothing,
  as if they were two apps stuck together. **With the band at the bottom**, it trims the window from below. **With the band
  at the top** (since sprint 4), **it moves the window down and shrinks it** by the band's height, without moving its bottom
  edge. When the band closes, the window goes back to its place and its size.
- **How to use it:** it needs the **Accessibility** permission. If you did not grant it, the band floats
  on top and warns you with “not docked”.
- **At the top, only the meeting window is touched** that the app detected —Meet in your browser, Zoom or
  Teams—, never the one you have in front of you. Without a session, the band floats below the menu bar and says “not
  docked”; it docks when the session starts.
- **At the bottom, docking is the usual one: the window you have in front of you.** The app docks it by itself in the
  30 seconds after you open it, and every time you put the band at the bottom (`⌃⌥B` or _Session_). “Start
  session” does not dock it: if you opened the meeting later, bring it to the front and press `⌃⌥B` twice (the
  band goes up and comes back down, and on coming back it docks it).
- **After `⌥⎋` nothing is docked:** with the band closed, `⌃⌥B` and the _Session_ row only change your
  choice; they touch no window.
- **Known limitations:** without it the band floats on top instead of docking. And with **Meet in the
  browser** it is needed for something more: without it the app cannot read the tab title, so it
  does not know there is a meeting —_Session_ says “Cannot tell whether there is a meeting”—, does not read its screen and
  the amber radar does not look. With Zoom and Teams it is not needed for that.
  - **At the top:** if the meeting window would end up less than 240 px tall, or is on another
    screen, it is not touched and the band floats. If macOS will not let it be moved (full screen, another
    desktop), the app undoes what it did and the band floats.
  - At the top it is **tested live in Chrome, with the kit's Meet test page**: it docks in about
    230 ms and is returned in about 120. With Zoom, Teams, an external screen and full screen, not yet.
  - If you move the window yourself while it is docked, it is not returned on close: the app only returns what
    is still as it left it.

### Top or bottom: the band next to the camera · New · Sprint 004

- **What it does:** by default, the band goes **at the top**, below the menu bar and the notch, next to the
  camera. Reading the card there looks like looking at whoever is speaking; at the bottom, it looks like looking down. Its
  backdrop goes with it, so when you share your screen the client sees your desktop background in that strip.
- **How to use it:** in _Session_, the row **“The band: top · bottom”**, or **`⌃⌥B`** from anywhere.
  When it changes, the band lets go of the window it had docked and repositions itself. At the bottom it docks the
  window you have in front of you; at the top, the meeting's, only with the session started (without a session, it floats with
  “not docked”). The choice is remembered.
- **The first time**, _Session_ notifies you: “The band now sits at the top, next to the camera”, with
  **“Got it”**. Until you press it, the notice takes the place of the meeting card; the
  meeting stays in the rail.
- **Known limitations:** at the top the band uses the same protection as at the bottom, but it is **not
  verified**: nobody has looked at it while sharing the screen with the band at the top, and it is in the cycle's tests. What is
  verified is the band at the bottom, and _Session_ says so: “Your panel does not show in the
  screen you share. Verified on your Mac (macOS 26.6.2) on 2026-09-20 with the band at the bottom; with the
  band at the top, and in Zoom and Teams, it is unverified.”

### Listening to both tracks · since Sprint 001

- **What it does:** it hears your microphone (that is you) and your Mac's audio (that is the client) separately, and
  converts both to text **inside your machine**. No audio leaves your Mac to be turned
  into text.
- **How to use it:** _Session_ → “Start session”. Nothing turns on until you say so. `⌃⌥T`
  shows or hides the transcript. It appears **next to the card** that is in the band (it is its right
  column: without a card there is nowhere to paint it) and, if the band is compact, it enlarges it like
  the handle; hiding it returns it to its height. It is born hidden on purpose, because reading what was just
  said is the fastest way to stop listening.
- **Known limitations:**
  - **Use headphones.** With the Mac's speakers, your microphone also hears the client and the same
    turn arrives on both tracks. The app detects it and marks it as echo, but the conversation reads
    worse. It warns you before you start, while you can still put them on.
  - Each track listens to **one** language, and **it is chosen in _Language_**: each track's code (es-ES,
    en-US…) is a selector. Both start in **Spanish**; if your client speaks English, change it before
    “Start session”. **The choice is remembered** · since Sprint 003: the next time you open
    the app, each track stays in the language you left (`lo_que_se_elige_sobrevive_al_reinicio` proves it,
    in `prefs.rs`). Marking several languages on the same track is not in the app.
  - macOS only lets you have **five voice languages ready at once**. It is a system limit.
  - If you do not speak, the audio counters stay still. **That is correct**: when nobody
    speaks, macOS does not deliver a single sample.
- **If a track does not open, _Session_ tells you why · new in Sprint 002.** With the session running,
  the row of the track that did not open changes symbol, word (“Did not open”) and color, and below it
  gives the reason and what to do: the microphone or system audio permission is missing, another app has
  the device, the device delivers a format the app cannot read, or macOS would not let it open.
  And below appears the row **“Partly”**, which says which one remains (since sprint 3 it only appears when
  a track drops). Until sprint 002
  this screen said “Works” on both tracks no matter what, and you would have found out at
  the end of the meeting, seeing that half the transcript was missing.
- **Headphones by name · new in Sprint 002.** If the sound goes out through a USB or Bluetooth device
  (AirPods, a USB speaker), _Session_ shows its name. The app **cannot know** whether it
  is a headset or a speaker: if it is a speaker, do not use the audio-only mode. Over HDMI, DisplayPort or
  AirPlay it treats it as a speaker: the row shows in amber and says so.

### In person: the app listens to the room · New · Sprint 005

- **What it does:** for the meeting at the table, or for the video call running on another machine —a PC with
  Windows— with your Mac beside it (if you prefer not to install anything on that PC; since sprint 007 the app also
  runs on it). The app listens to **the room through your Mac's microphone, and nothing else**: it does not open
  the system audio or read the screen. With what it hears it searches your documents and gives you the usual card
  —headline, line and where it comes from—, by default **in your ear**.
- **How to use it:**
  1. In _Session_, **“In person”**, next to “Notes only”. Nothing starts yet: you see what the
     session will be like. “call on this Mac” takes you back to the usual one.
  2. Choose the **room language**: one for the whole room; by default, the client's.
  3. Choose **how you see the card**: **in your ear**, which comes selected, or **the band**. With the Mac's speakers,
     or with a monitor or a television over HDMI, DisplayPort or AirPlay, the band comes selected: the voice would not
     speak with the client listening to it.
  4. **“Start in person”.** To finish, “End session”: the voice turns off with the session and `⎋`
     returns to your other apps. `⌥⎋` cuts everything, as always.
  5. With the session running, `⌃⌥V` switches between the voice and the band; so does the _Session_ selector.
- **With the video call on a Windows PC where you do not install the app:** put the PC **on speaker** and the Mac beside it. Nothing is
  installed on the PC and the app does not touch that machine. **If you wear headphones on the PC, the app does not hear the
  client**: it only hears you.
- **How it decides when to give you a card.** Your voice and the client's come in through the same microphone, and the app
  **does not tell voices apart**: it does not identify anyone by their voice. So that it does not fire on what you say, the
  room follows three rules about text and time, chosen by measuring with two synthetic voices:
  - after a card, it waits **15 s** before giving another;
  - a term from your corpus said in passing is not enough: it takes **a question or a figure**;
  - the room's silence does not ask for a card.

  And no room turn lasts more than 15 s: two overlapping voices are not merged into a block that
  arrives late. In the test, of 14 client turns that asked for a card, 5 came out —as many as without rules,
  though not the same ones— and the extra ones, all caused by your voice, dropped from 16 to 5.
- **The voice waits for the room to go quiet.** If a card arrives while someone is speaking, the band says “The
  room is talking · I’ll say it when it’s quiet”, and the voice reads it as soon as there is silence. If in 4 s nobody goes quiet,
  the card stays in the band and `⌃⌥V` shows it to you.
- **What it does not do (known limitations):**
  - **It does not separate voices.** It does not know what you said and what the client said, and does not guess: in the transcript each
    turn says “room”.
  - **Some client questions come without a card:** those that arrive a few seconds after one of
    yours —the room takes them as your conversation—, terms the client names without asking, and the
    quick follow-up question. That is what `⌃⌥A` is for: it searches with the last thing the room heard, without rules.
  - **It does not keep your turns**, even if you have it turned on in _Notes_: the app does not know which are yours. Your
    preference does not change and applies again in the next meeting. Your notes, your pinned cards and your
    rehearsals are kept.
  - **It does not suggest what to keep** and nothing goes to the tray: it does not know whose each sentence is.
  - **It does not read the screen or move windows.** The band stays protected from capture: if you share your
    screen on a TV, the room does not see it.
  - **The whole room goes in one language**: if you speak in another, your part is transcribed badly.
  - **If the client's NDA forbids recording**, it does not start in person: only in notes-only mode.
  - **The suggestion**, if you have it turned on, is written on the sentence that triggered the card, not on
    the last one said.
  - **With Bluetooth or USB headphones** the app cannot tell a headset from a speaker: if it is a
    speaker, choose the band.
  - **If your headphones have a microphone**, macOS may listen through them and the app would hear mostly your voice,
    with the client far away or not coming through. _Session_ names the microphone it will use before starting and, if it is
    external, warns you: choose the Mac's microphone in System Settings → Sound → Input.
  - **With the card in your ear, `⎋` belongs to the app** for the duration of the session: it reaches no other.

### Your corpus, indexed · since Sprint 001

- **What it does:** it reads your documents and sorts them into the five units of consulting work:
  proposal, framework, case, client card and your profile. Every card you will see in a
  meeting comes from there.
- **How to use it:** *Corpus* → “Point to a folder”. It reads PDF, Word (.docx) and Markdown, going down
  through the subfolders.
- **Known limitations:**
  - **It does not read scanned files.** A PDF that is a photo of a paper is left out, counted and with its
    reason written down. It is never sent to an outside service to be read.
  - **A PDF has no headings**, it has lines: the app guesses where each section starts from the
    shape of the text, and tells you how many documents it made that guess for, so you can judge it.
  - Up to **2,000 documents** per folder.
  - **The app remembers the folder** (only its path, in your preferences) and **reads it again on
    startup**, in the background: *Corpus*, “This client” and the local door have it without you
    pointing to it again. If the folder lives in Documents, Desktop or Downloads, macOS may ask you
    the first time whether Angel Ghost can read it. If it is no longer there (a disconnected disk), the app says so and
    you can point to it again.
  - Dragging documents onto the window and re-reading only what changed are not in the app: today it
    walks the whole folder again.
  - The index lives in the app’s data folder, **and only your Mac account can read it**.

### The card at the right moment · since Sprint 001

- **What it does:** when the client asks something, states a figure, names something that is in your
  documents **or goes quiet after speaking**, the app searches your corpus and puts in the band
  a **headline of eight words, one line and the exact source** — document and section.
- **How to use it:** on its own. And `⌃⌥A` (“help me with this”) asks for it by hand when it misses.
- **It says why it came and how long it took · new in Sprint 002.** Below the source, a short line:
  “question · 1.2 s”, “figure · 0.9 s”, “your term”, “silence”, “you asked” or **“on screen”**
  —this one with the screen symbol, because it is the only card that arrives without anyone saying anything—.
  If the section came from a PDF, where the app **guesses** the headings from the shape of the text, it
  says so right there: “guessed section”.
- **Silence also asks for a card · new in Sprint 002.** If the client says something that carries no
  question or figure and goes quiet for **four seconds**, the app searches on its own for the last thing
  they said — it is the gap where it is your turn to speak. Two things it does well that are worth knowing:
  **while the client keeps talking it does not count as silence** (even if their previous sentence ended
  a while ago), and **it does not repeat the card that is already in the band**, so a question that already brought its
  card does not bring a second one when they go quiet. In sprint 001 this was written in the code and not
  wired up, and this manual declared it a limitation; now it works.
- **Known limitations:**
  - **Everything the card says is cut from your documents.** The card is not drafted: if it does not
    find anything, it says so. (The only thing drafted is the *suggestion*, if you turn it on, and it goes apart,
    marked as such and checked against its card.)
  - When it finds nothing, it shows **what it searched for** —so you can see at once if it misunderstood you—,
    the closest thing you do have, and a suggestion on how to conduct yourself. That suggestion speaks of
    **how to answer**, never of your business. If the question does not fit any known situation
    (price, timeline, credential, reference, contract) but your corpus has something close, **it
    names it for you**: “The closest thing you do have is ‘Scope’: offer it and ask what they
    need it for.” · new in Sprint 002.
  - It searches by the **words** of the question. If the client asks something with words
    completely different from those in your document, it will not find it.
  - Your own voice does not trigger cards: the app would answer you in the middle of your sentence.

### Nothing is recorded, and you can check it · since Sprint 001

- **What it does:** the audio, what was said and **the last frame read from the screen** live **only in
  memory** and die on close. The *Honesty* screen shows how much each one takes up right now and
  how many bytes have left your computer.
- **How to use it:** `⌥⎋` cuts everything on the spot — audio, transcript, the band and its backdrop — and
  puts the meeting window back in its place and at its size. There is also the button in *Honesty*,
  in case another app has taken the shortcut.
- **The cut reaches all 12 pieces · since Sprint 004.** *Honesty* says it under the red button:
  “The button cuts 12 of 12 pieces: none is left out.” In sprint 4 **the rehearsal** arrived: if you cut in
  the middle of one, the microphone closes and your unsaved answers go. In sprint 2 the screen reading and
  **the suggestion on its way** arrived: if the model is drafting when you cut, whatever comes back is
  thrown away and, with the external provider, a request that has not left yet does not leave. In sprint 3, **your
  turns as text** and **the suggestions you have not decided**: the cut takes them, just like the
  rest of the capture. **Your notes, your agreements, your pinned cards and the suggestions you saved
  stay** —*Honesty* says “Your notes are still there”—, and the tray of an already closed meeting keeps its
  window. The voice of the
  audio-only mode is the first one cut, because it is the only one your client could hear.
- **Known limitations:** the button cuts what lives in memory **now**; what has already come out of
  the speakers or what you have already seen in the band obviously cannot be undone.

### Your notes: the only thing from the meeting that remains · New · Sprint 003

- **What it does:** while you listen, *Notes* is your meeting notebook: **your note** (a field to
  write whatever you want), **your agreements** (you write them; the app does not decide what counted as an agreement) and
  **the cards you pinned**. When you finish, they are saved in **one encrypted file per meeting**, in the
  **app’s private folder**. It is the only thing from the meeting that reaches the next day, along with the
  suggestions you save: of the client, neither their voice, nor their turns, nor what was read from their
  screen is saved; at most, a one-line fact that you accepted.
- **How to use it:**
  1. During the session, `⌃⌥N` brings the notebook to *Notes* with the cursor at the end of your note. Write
     and carry on: it is kept in memory with every keystroke.
  2. An agreement: write it in its field and press ↵.
  3. `⌃⌥P` pins the card you are looking at in the band (its headline and its source, not the document). In
     the band, “Note it for later” does the same as `⌃⌥N`: it takes you to your note so you can write it
     in your own words —the client’s question is not copied—.
  4. When you stop the session, *Notes* shows you **what will be saved and what dies**, counted, and the name
     of the file that will be born. “Save encrypted and close” or “Close without saving”.
  5. With no meeting open, *Notes* is **the archive**: your saved meetings, where they live, with which
     key and when they are deleted. **“Show in Finder”** opens their folder with the meeting you are viewing
     selected. **“How long your notes live”** is a single choice for all of them: 7 days,
     30 days, **90 days** (default), 1 year or always. Each meeting is saved with the one in force when
     you save it.
  6. **“Export to text”** asks you first —because the exported file is no longer encrypted—, then
     asks for Touch ID or the Mac password, and then where. The `.md` carries your note, your agreements,
     **the suggestions you saved** (the client’s, the fact in one line, as on screen), your pinned
     cards and, if you keep them, your turns. The folder you choose does not change its permissions: the
     exported file is born readable only by your account. **“Delete now”** also asks first: it is
     deleted from the app and does not come back; if you use Time Machine, your backups keep it until they expire.
- **“Keep my turns”**: if you turn it on (it starts off), what **you** say, as text, also goes into
  the file of the following meetings. Never the audio, not even yours. And never a turn from
  your microphone marked as echo: with speakers, that turn is the client’s voice.
- **The notebook, protected:** from the moment you start listening until you save or discard the meeting,
  the notebook window is **protected from capture** like the band, with the same system flag.
  That way your note is not seen if you share the entire screen by mistake. **Not yet verified with the
  notebook:** the band is checked in Meet on macOS 26.6.2, with the band at the bottom; the notebook is
  checked at stop 5 of the ⭐⭐ (Act 2), and in Zoom and Teams, neither of the two.
- **Where they live:** in `~/Library/Application Support/com.aiapps.copiloto-consultor/notas/`, next to
  the tray. **Not in Documents**: from there the task that deletes what has expired while the app is closed cannot
  get in, and the iCloud trash would keep for 30 days what you delete. That folder is not synced with
  iCloud. **Your notes do go into your Time Machine backups**: they are yours and made to last, and
  if your Mac breaks you recover them from the backup. If saving fails —the Keychain does not answer, the
  disk is full—, the meeting stays open, your note whole, and “Try again” saves again.
- **The key:** it lives in the **login keychain** of your Mac (“Angel Ghost · notas”): it opens
  with your session and is not synced with iCloud, but **it travels with your Time Machine backups and with
  Migration Assistant**, protected by your password. Saving asks you for nothing; exporting a saved
  meeting asks for Touch ID or your password, once each time you open the app. **Your saved rehearsals
  use the same key and the same retention** (*The rehearsal*, below).
- **Known limitations:**
  - **If you delete the Keychain, your saved meetings cannot be opened.** The app has no recovery:
    it would be a second key. Only a Time Machine backup (or Migration Assistant) brings back your
    keychain, and with it your notes, which keep opening.
  - **Having the key stay only on this Mac arrives with Apple signing (H2):** until then, the
    keychain that the app can use does not know how to tie it to a single machine.
  - **Your notes do not appear in Documents:** you reach them from *Notes*, or with “Show in Finder”.
  - **The file name is in the clear** (`reunion-2026-09-27-1402.ghost`, or
    `paramo-azul-2026-09-27.ghost` if you chose a client): it says which day and, if you chose a client, with
    whom; not what was discussed.
  - **If the app crashes, whatever is unsaved is lost**: it lives in memory on purpose. If you leave the app with
    unsaved notes, they are saved on their own; and if you start another session with the previous one open, too.
  - **Your expired notes delete themselves, even if you do not open the app:** the same macOS task
    as the tray (below) deletes them, in the minute after they expire. Those you saved for “always”
    do not expire. While you have notes with a date —or rehearsals—, that task appears in Login Items.
  - If you choose the client in *Session* (“This client”, below), the file carries their name
    (`paramo-azul-2026-09-27.ghost`); otherwise it is called `reunion-<fecha>-<hora>`.

### The app suggests what to save, and the tray · New · Sprint 003

- **What it does:** while you speak, the app **suggests** what to save from the meeting, with **published rules and
  no model**: figures, deadlines and dates · your commitments (“I’ll send it to you”, “we agreed on”) · what clashes
  with a card you pinned · names that are not in your corpus · questions you were asked. Suggesting
  is not saving: nothing goes into your file without your yes.
- **What is saved from each side:** of what you say, your sentence. Of the client, **never their turn**: a
  one-line fact (“They said ‘four sources’; your pinned card says ‘three’”), with a fragment of
  eight words at most; of a question, only its keywords.
- **How to use it:**
  1. The latest suggestion appears in the band, at the top and on one line: “I suggest saving: …”. It does not
     cover the card, makes no sound and is not read aloud. **`⌃⌥↵` saves it.**
  2. In *Notes*, “I suggest saving this” shows them all, each with “Save” and “No”. The ones you
     save go into your file, apart from your agreements.
  3. When you stop, “on close” says how many remain undecided and lets you choose **how long they wait in the
     tray**: on close · 1 h · **3 h** (default) · end of day · 24 h. The choice is remembered.
  4. After “Save encrypted and close”, *Notes* shows **the tray** with its countdown: “Save” takes
     the suggestion to its meeting (with the meeting’s retention) and “No” deletes it. Also “Save
     all”, “Discard all”, changing the window right there and “Go to your meetings”.
  5. If the tray is from an earlier app session, its countdown is visible, but **what it says
     asks for Touch ID**, like exporting a meeting.
- **Where it lives:** encrypted with the same key as your notes and next to them, in the app’s private
  folder, **not in Documents**: the iCloud trash keeps for 30 days what you delete, and a list that
  promises to die at 3 h cannot have a copy that lives for a month. For the same reason, **the tray does not go
  into your Time Machine backups**: the app marks it as excluded from backups when it writes it. The local
  snapshots that macOS keeps for a few hours (under 24 h) do see it until they are recycled.
- **It deletes itself on expiry, even if you do not open the app.** macOS does it with a scheduled task
  at the minute of each expiry; between expiries nothing runs. *Honesty* shows the tray with its
  countdown, and if the task did not run while the app was closed, it says so in red.
- **Known limitations:**
  - **The first time, macOS warns that a background item was added.** Until the app is
    signed, in System Settings → General → Login Items & Extensions it appears as **“sh · Unidentified
    Developer”**: it is Angel Ghost’s deletion task, and it stays as long as there is a tray, notes or
    rehearsals with a date. If you turn it off, the tray, your notes and your expired rehearsals are only deleted with
    the app open, and *Honesty* says so.
  - If you delete the app, the task keeps deleting whatever is left at its time; afterwards it stays as an
    inert entry in Login Items, which you can remove.
  - At most 30 suggestions wait; on reaching 30, new ones do not come in and *Notes* says so.
  - With the local model on, the app does **not** draft the suggestions better: in this cycle they are only
    rules.

### The framework in hand: your client, their jurisdiction and their NDA · New · Sprint 003

- **What it does:** before you start, *Session* shows you **which rule applies to this client**, with its
  norms and its date; asks you **whether their NDA forbids recording or transcribing**, and if it does it
  proposes **notes-only mode**; and gives you a **model clause** for your engagement letter, in Spanish and
  in English. It **never blocks you** and it is **not legal advice**: it always says so.
- **How to use it:**
  1. In your client’s file (in your corpus), write a line with where the counterparty is:
     `Jurisdicción: Colombia` (or `Jurisdiction: Florida`). Re-index the folder.
  2. In *Session*, **“This client”** → choose the client. Its **flag** appears: the country or state,
     its risk (with symbol, text and color), the rule, what it implies, the norms and the date on which they were
     consulted (2026-09-17). If anything in that row could not be verified, it says so below:
     “Unverified: …”.
  3. **“Review”** next to the NDA: the app asks “Does this client’s NDA forbid recording or transcribing
     by any means?”. Your answer is saved for that client.
  4. If you answer **“Yes, it forbids it”**, *Session* proposes **“Start in notes-only mode”**, and lets you
     review the NDA again if you made a mistake.
  5. **“Engagement clause”** shows the clause in both languages, side by side, each with a “Copy”:
     copy the one in the language of your letter, not the app’s.
- **Notes-only mode:** you can also choose it yourself, with **“Notes only”** next to “Start session”.
  The meeting opens —your notebook, protected, and the band— and **nothing listens to it, transcribes it or
  reads it**: no microphone, no system audio, no screen, no amber radar. Your notes and agreements
  (`⌃⌥N`), pinning (`⌃⌥P`), the coral radar and `⌥⎋` carry on. **`⌃⌥A` searches your corpus with the last line of your
  note.** The band says it in amber: “Notes only · no transcription”. It ends like any
  session.
- **The meeting file** carries the name of the client you chose. The choice lives while the
  app is open: it is not saved to disk.
- **Known limitations:**
  - **It is not legal advice.** The catalog comes from a sourced investigation, dated 2026-09-17,
    with 17 points that could not be verified and that the app does not assert. Before using the app with clients
    in US all-party-consent states, in Germany, France, Chile or
    Argentina, or under an NDA that forbids transcribing, consult a lawyer.
  - The catalog has **27 jurisdictions**: Colombia, the US (federal and one-party-consent states), 14
    all-party or mixed-consent states, 3 with no verified statute, the European Union, Germany,
    France, Spain, Mexico, Chile, Peru and Argentina. If the file names another one, the app says so (“is not in
    catalog v1”) and does not guess.
  - If the line names several (“Colombia and California”), the flag is that of **the strictest one**.
  - The catalog is updated with a new version of the app, never over the network.
  - The template for the one-line notice to the client and the record that you informed them are left for H2.

### The rehearsal: practice before the meeting · New · Sprint 004

- **What it does:** before the meeting you choose a client and a proposal, and the app asks you **the questions
  that client is likely to ask**. They come from published rules applied to your own proposal (its sections, its
  figures, its deadlines and commitments, its risks and assumptions), from the client card for that client (who decides,
  prior agreements) and from a catalog of typical data, BI and AI objections, each with its source.
  It reads you the question, **listens to you through the microphone only** and, when each answer ends, tells you:
  - which evidence from your corpus you **used** and which you **had and did not use**;
  - how long you took;
  - your pace, in words per minute;
  - which fillers you repeated.

  **No score**: these are figures of what was measured. No model decides any of this.
- **How to use it:**
  1. In the rail, *Rehearsal* (under *Session*). Choose the **client** and, if it has several, the **proposal**;
     choose **5, 8 or 12 questions**. “Read the question aloud” comes turned on: turn it off if you
     prefer to read it yourself.
  2. **“Start the rehearsal”.** The voice reads the question; while it speaks, the microphone does not listen. Then,
     answer out loud.
  3. Your answer closes on its own after **2.5 s of silence**, or with **Enter** if you finish sooner. Thinking
     before the first word does not close it.
  4. Look at the evaluation. If you said something in other words and the app did not recognize it, press **“I did
     say it”**. **Enter** brings the next question.
  5. The keys belong to the window: **Enter** done · **R** repeat · **S** skip · **Esc** end.
     `⌥⎋` cuts everything, the rehearsal included.
  6. At the end, **the report**: how many you answered and how many you skipped, the evidence used, the average
     pace, the average time and the fillers. Three buttons: **“Save with your notes”**, **“Export
     as text”** and **“Close without saving”**.
- **What is saved and where:** only if you press “Save with your notes”: **your answers as text and the
  figures for each question**. The audio, never, not even for a moment on disk. It is encrypted with **your
  notes’ key** and deleted with **its retention** (90 days by default; “always” does not expire), even if you do not open the
  app: the same macOS task as your notes deletes it. It lives in the app’s private folder, in
  `ensayos/`, named with the client’s name and the date (`paramo-azul-2026-10-04.ghost`): it says who you
  rehearsed with and what day, not what you said. Saving asks you for nothing. The first time you save something with a
  date, macOS may warn that a background item was added (Background Items Added): it is the task that
  deletes what has expired. And if your notes’ key was created by another build of the app, macOS may ask
  whether it can use it. A report you do not save is not saved on its own when you leave the app.
- **Your progress with a client:** with a saved rehearsal, *Rehearsal* says “Rehearsals saved with this
  client” with its count and **“See your progress”**. Opening it asks for **Touch ID or your password** (“open
  your saved rehearsals”), once each time you open the app. It is the same unlock as your notes: if
  in this app session you already exported a meeting, it does not ask again. It is a table with one rehearsal per
  row —evidence, pace, fillers and time— and, from the second one, **“Since the first”**: how each
  figure changed, with its arrow. Only figures: your answers are not shown there. The last six are shown, and the
  earlier ones are counted.
- **Export and delete:** “Export as text” saves the rehearsal you just finished as a `.md`
  file you can read, wherever you choose, with the same unlock; nobody protects that file any more. A rehearsal already
  saved is not exported again. **“Delete this client’s rehearsals”**, in your progress, asks
  first, and does not open the files or ask for another unlock.
- **“Enrich the bank”** (in *AI*, off by default): if you turn it on, when you start a model suggests
  **up to 5 more questions**, grounded in your sections; they go at the end and are marked “suggested by the
  model”. It goes the same way as the suggestion —the system model, or the external provider if you
  turned it on, with the client’s data masked before leaving and counted in the network—; in “See what left”, its
  row says “enrich the bank”. A question that is not grounded in a section of yours is thrown away. With no
  model, the rehearsal works the same, with the rules.
- **While you rehearse:** *Honesty* shows **“Your rehearsal answers”** in memory, as text, until
  you save or close, and how much left your computer **in this rehearsal** (0 B, except “Enrich the bank” with
  the external provider); the rail says “Rehearsing”. The rehearsal **does not open** the system audio or
  the screen. The local door closes on its own, and *AI* says “It closed itself: there is a meeting or a rehearsal”.
- **The rehearsal and the session do not coexist.** You cannot rehearse with a session open, and starting a session
  discards the rehearsal. If you have a finished one that is unsaved, *Session* warns you first, on a line below
  “Start session”: “You have a finished rehearsal you have not saved” · “Starting the session discards it. Save it first in
  Rehearsal.”
- **With a video call open, not through the speakers.** If Zoom or Teams are open, or there is
  a Meet tab, and the sound comes out of the Mac’s speakers, over HDMI, DisplayPort or AirPlay, the rehearsal
  does not start: your microphone would hear the call —and take it for your answer— and the call would hear the voice that
  reads the question. *Rehearsal* says “A video call is open”
  (“With the speakers, your microphone would hear the call and the call would hear the rehearsal’s voice. Put on
  headphones or close it to rehearse.”). Without the Accessibility permission, the app cannot see if your browser
  is in a call: with the browser open and those speakers, it does not start either, and says “Cannot tell whether a
  video call is open” (“Without the Accessibility permission, the app cannot see whether your browser is in a call.
  Put on headphones or grant it in Permissions.”).
- **Known limitations:**
  - **The macOS transcription does not keep “uh” or “um”**, so they are not counted: only the fillers
    that it does write (“I mean”, “basically”, “like”, “you know”…). The screen says so.
  - **“You used” compares words, not ideas:** a card counts as cited if your answer shares with
    it at least two terms that were not already in the question. If you said it in other words, “I did
    say it”.
  - If the question is very general, there may be no cards to compare with and the figure comes out “—”: the app
    does not accuse you of anything you did not have.
  - **If you make long pauses in the middle of an answer**, the 2.5 s may cut you off: finish with Enter.
  - With no meeting, **the voice comes out wherever your Mac plays sound**; with a video call open, the rehearsal does not
    start through the Mac’s speakers. While it speaks, the microphone does not listen.
  - **With a video call open, the rehearsal does not start if the sound comes out of speakers that the app
    recognizes**: the Mac’s, a monitor over HDMI or DisplayPort, or AirPlay. With wired, Bluetooth or USB
    headphones —AirPods, for example— it does start: macOS cannot tell Bluetooth
    or USB headphones from a desk speaker, and the app trusts it, as with audio-only mode. **If the sound comes out of
    a Bluetooth or USB speaker, do not rehearse with a call open.** Without a video call, rehearse with whatever you
    like. It is checked at the start: if you open a video call mid-rehearsal, the rehearsal continues; end it
    with Esc before joining the call.
  - **The rehearsal has not been tried with a real voice yet.** The CI runs it with the kit’s audio
    and a full session in which a rehearsal is saved and decrypted; since it has no models to
    recognize voice, there the answer is saved without text. With your voice, your microphone,
    the Keychain and Touch ID it goes in the cycle’s tests.
  - With no proposal or client card for that client in your corpus, *Rehearsal* says “Your corpus has nothing for this
    client”: the app does not make up questions.
  - The rehearsal’s language is that of your proposal; if it cannot be known, that of the client card, and if
    that is not known either, that of your track. *Rehearsal* says it at the top (“Rehearsal language: Spanish.”). If your Mac does not
    transcribe that language, *Rehearsal* tells you and sends you to *Language*.

### The voice model for a language · since Sprint 001

- **What it does:** each language needs its own voice model, and macOS does not ship them all. *Language* says
  for each track whether its model is installed and, **if it is missing, offers to install it**.
- **How to use it:** *Language* → “Install”, the button next to the amber notice in the row. macOS
  downloads it and it takes a while: meanwhile, the button says “installing…” and cannot be pressed
  again. It is the only time transcription touches the network, and only because you asked for it (the other
  door to the network in the app is the external AI provider, off unless you turn it on).
- **Known limitations:**
  - If your Mac **does not recognize** that language, or has no speech engine, there is nothing to install and the app
    says so in those words instead of offering you a button that cannot work.
  - macOS allows **five languages ready at a time**. It is a system limit.

### Reads the meeting screen · New · Sprint 002

- **What it does:** it watches the meeting window —Zoom, Teams or the Meet tab— and **reads only what is
  new**: when another slide, another sheet, another document appears, it waits for it to hold still and
  reads it **once**. If it contains a figure or one of your terms, **it brings you the card without anyone
  asking**. And when the client asks something that two cards would answer equally well, what is
  on screen **breaks the tie**.
- **How to use it:** by itself, from the moment you press “Start session”. In *Session*, below the screen row:
  - the **“Read it on its own”** switch turns it off: then it watches nothing and forgets what it had read;
  - **`⌃⌥L`** reads it **once, now**, even if it is off. If there is no text —a camera, a video, a
    black screen— the band answers “I read the screen: there is no text to search.”

  **With a strict NDA:** turn off “Read it on its own” and use only `⌃⌥L` whenever you want.
- **What stays and what does not:** the image is **never saved**. In memory lives only **the last
  frame** and the words taken from it; *Honesty* says how much space it takes, and the cut (`⌥⎋`)
  erases it along with everything else. None of this leaves your Mac: macOS does the reading inside your machine.
- **Measured:** in the test kit (five synthetic screens), **four out of four** bring their card without
  being asked, and the agenda —which has neither a figure nor a term— brings none. Each reading takes **less than
  a tenth of a second**, and there is at most one per second.
- **Known limitations:**
  - It needs the **Screen** permission and, with Meet in the browser, also **Accessibility**
    (to find the tab). Without Screen, *Session* says so (“No permission”) and the app
    works the same, just without reading it.
  - It reads **the meeting window**, not your whole screen. It reads it **even if another window covers
    it**: macOS hands over its content as is. What leaves it unread is **minimizing or
    hiding it** (`⌘H`).
  - **The participants’ video does not count as “something new”**: it moves all the time and carries no
    text. If someone shares a video, the app waits for it to hold still.
  - It reads Spanish and English. A slide in another language is read worse.
  - While the client moves the mouse over the slide, the app waits: it only reads what holds
    still.

### The radar: who is recording and who is watching you · New · Sprint 002

- **What it does:** it warns you of two things, with two different colors and two different symbols, **looking only at your
  Mac and your screen**:
  - **Amber, “Be aware”** (the recording dot): the meeting screen shows the Meet, Zoom or Teams
    **recording or transcription notice**, or the name of a **note-taking bot** appears
    in the meeting window. The band says, for example, “Meeting recorded · note-taking bot present” and where it
    got it from: “read from your screen · 14:03”. That bot belongs to someone else: **Angel Ghost never joins the
    call**. The client has the right to record and to bring their bot; the app **warns, it does not block**.
  - **Coral, “Invasive”** (the filled X): a program **on your own Mac** that watches your screen,
    your camera, your keys or your processes —exam proctoring, employee monitoring, remote
    access—. The band says which one it is and **what it can see**: “A program on your Mac is watching you ·
    ‘X’ (remote access): …”.
- **How to use it:** there is nothing to do.
  - The **coral** one watches the programs on your Mac from the moment you open the app, every ten seconds. If it finds
    one **before you start**, the whole of *Session* is the notice —“Invasive software running on your Mac”—, with
    a table of what it found, **what each program can see** and the catalog version; you decide:
    **“Start anyway”** or **“Do not start”**.
  - With the session running, the coral one jumps into the band. **`⌃⌥R`** (“what it sees”) opens *Session* with the
    table; with the band expanded you also get **“See what it can see”** and **“Cut everything”** (the same
    cut as `⌥⎋`).
  - The **amber** one arrives with the screen reading: what that already reads from the meeting window is
    what the radar checks. The same notice is not repeated on every slide: it appears once, and again
    only if it changes (a new bot, for example).
  - If your Mac is **enrolled in an MDM** (a company’s device management), *Session* lists it
    as “Be aware” —in its list of what works, or inside the table if there is also an invasive
    program—: it is normal on a company machine and does not jump into the band.
- **What it does NOT do, and how it is checked:** it does not touch anyone’s machine, opens no connections,
  asks nothing of the internet and does not close or block any program. The list of programs is requested
  from your Mac’s kernel —the same one you see in Activity Monitor—, compared in memory and thrown
  away. The catalog travels **inside the app**, versioned and with the source of each row; it is updated
  with a new version of the app, not over the network. A test watches it and fails if the radar calls
  anything outside its list (`tests/unit/radar-solo-este-mac.test.ts`).
- **Measured:** in the test kit, a synthetic Mac with 82 normal programs —and deliberately
  similar names, such as Microsoft Teams or the “Screen Sharing” app you use to look at someone else’s
  screen— gives **zero** warnings; the same Mac with one program from each row of the catalog finds
  **all** of them.
- **Known limitations:**
  - The **amber one needs screen reading turned on** and the meeting window visible: if you
    turn off “Read it on its own”, the amber radar does not watch either. The coral one does not depend on that.
  - It recognizes programs **by the name of their executable**. A renamed program, or one that is not
    in the catalog, is not seen. **Browser extensions** (some exam-proctoring ones
    are) have no process of their own and are not seen.
  - An **open** remote access program does not mean someone is connected **right now**: the
    app warns that they could be, not that they are.
  - If there are two note-taking bots, the band names the first.
  - The bot is recognized by its name **anywhere in the meeting window** —the participant list, but also a slide or the chat that mentions it—: the band says
    that it “appears in the window”, not that it is connected.
  - The recording notice is recognized by its phrases in Spanish and English, just as Meet, Zoom and
    Teams write them today. If they change, the catalog has to be brought up to date.

### The suggestion: what to say, with its source · New · Sprint 002

- **What it does:** when a card arrives that answers what the client said, the app **proposes what to
  say in one sentence**, below the card: “Confirm that the cleanup includes up to three sources; a
  fourth is extra and is quoted separately”. It says where it was drafted (“suggestion · on your Mac”, or the name
  of the external provider) and with what **confidence** (high, medium or low). **The card does not go away**: its
  headline and its source stay in view, and with the band expanded the suggestion takes the slot on the
  right.
- **How to use it:** in **AI** (the last screen of the notebook), turn on **“Draft suggestions”**.
  It starts off: the app works entirely without it. At the top, **who writes**: the macOS system model, inside your Mac, if you have it; and if no one can, **why** (“Apple Intelligence is off in Settings”).
  - **The system model** needs **Apple Intelligence** turned on in *System Settings →
    Apple Intelligence & Siri*. It is free and nothing leaves your Mac.
  - **The external provider** (Claude or Groq) is optional and is **yours**: you paste your key, the app
    stores it in your **Keychain** —never in a file— and you turn its switch on. Without a key it cannot be
    turned on.
  - **What each provider does with what you send** is written in AI, under the cost, for the
    provider you choose.
    Neither of the two trains on it. **Claude keeps it up to 30 days**; for it to keep nothing
    you need a zero-retention agreement with Anthropic. **Groq may keep it up to 30 days**
    to watch for abuse, unless you turn on zero retention in *Data Controls*, in its console: if you
    use Groq, turn it on. What goes out is always anonymized on your Mac, but it goes out. Gemini was in
    the app and was removed: its API offers no zero retention at any tier, and without billing it trains on what
    it receives (the table, with its sources and the date it was read, is in ADR 011).
- **What it does not do, and how it is checked:**
  - **It never invents the source.** The model receives **the last sentence from your client** and the three cards of the moment, each
    with a number; it has to say which one the suggestion came from, and if it cites one it was not given, the
    suggestion **is thrown away** and you do not see it. It is watched by `sintesis::pruebas::una_sugerencia_sin_fuente_dada_se_descarta`,
    and the type that reaches the band can only be built by going through that check (it does not compile any
    other way).
  - **Nor does it let it say what the card does not say.** Citing correctly is not enough: every word with
    content in the suggestion has to be in the card it cites, the figures have to be the
    same (“four” is 4) and it cannot add or remove a “no”. If it fails, the suggestion is thrown away and you
    keep the card. It was born from measuring the real model: it cited correctly and said “the closing workshop is
    separate” when the proposal includes it. They are watched by `sintesis::fiel::pruebas` and
    `sintesis::pruebas::una_linea_que_la_ficha_citada_no_dice_se_descarta`. If the headline the model
    proposes comes neither from the card nor from your client’s sentence, you see the card’s headline
    (`…un_titular_inventado_se_cambia_por_el_de_la_ficha`).
  - **It does not make you wait.** The card comes out first, without waiting for the model; the suggestion arrives
    afterwards. If it takes more than **6 seconds**, it is not shown (`…pasado_el_techo_no_hay_sugerencia…`).
  - **Only short text, anonymized on your Mac, goes out to the external provider:** the last sentence from your client —a third party’s words— and the three
    cards, with your clients’ names, emails, phone numbers, long numbers and people’s
    names replaced by placeholders before leaving; the reply comes back with the names
    put back, on your Mac. Never audio, screen or whole documents: that path does not exist
    (`api::pruebas::lo_que_sale_al_api_no_lleva_los_nombres_plantados`). Every byte that goes out is
    counted in *Honesty* and in the band.
  - **You can see exactly what went out · since Sprint 003.** In *AI*, when something has gone out —in a
    meeting, or in a rehearsal with “Enrich the bank”—, **“See what left · N”** appears on the
    external provider’s panel. It opens the exact text of the last request —with what was masked on
    your Mac **struck through** and its placeholder next to it—, how many characters it was, and the table of
    **recent requests**, the last 20 (time, what it went out for —“draft suggestion” or “enrich
    the bank”—, characters, masked data and USD). That log lives in memory like everything else: it is
    erased when the session ends, with `⌥⎋` or when you leave the app. “← Who writes” takes you back.
  - **Nothing of the suggestion goes to the log**: only who, how long it took and how much went out
    (`tests/unit/logs-de-la-sintesis.test.ts`).
- **Cost:** the system model costs zero. The external one is charged to your account with the provider; AI
  shows **this meeting or rehearsal**, **this month** and the **monthly cap (USD 10)**. On reaching the cap, it goes back by itself
  to local; the meeting does not stop. The month’s figure is the only thing saved (a number, not the text
  of anything).
- **Measured** (test kit, `el_kit_de_sugerencias_mide_grounding_y_latencia`): of the 30
  questions, 17 reach a card. With **the system model** (Apple Intelligence, on a Mac with
  macOS 26, 2026-09-26): between **11 and 13 of 17** suggestions pass the check —it varies from one
  run to another—; the rest are thrown away and the card remains. It takes **~0.8 s median and ~1 s at the worst
  5%** (the budget is 4 and 6). With the CI test provider, 17 of 17. Before the
  fidelity check, 17 of 17 “cited correctly” and 3 of them said something false or foreign to their
  card: that is why the lower figure is the honest one.
- **Known limitations:**
  - The cards that **the screen alone** brings carry no suggestion: they answer no question.
  - A **loose** person’s name (“Andrea”) that is not in your corpus is not recognized when anonymizing;
    two capitalized words in a row (“Andrea Villalba”) are.
  - The **MLX** path (a model you download yourself) is not in this MVP: it stays on the H2 roadmap, and
    in AI its row says so with “In H2”. Without Apple Intelligence, the local suggestion is not available.
  - The suggestion is written **in the language of the card it cites**, not of the question: if your
    client asks in English and your proposal is in Spanish, the suggestion comes out in Spanish.
  - Sometimes the suggestion is faithful to its card but **does not answer the question** (it cites another of the three
    cards of the moment). It is not false; it is of little use. The confidence usually says so.
  - The AI switches **are remembered** · since Sprint 003: “Draft suggestions” and the
    external provider stay as you left them. With one exception, on purpose: if you deleted the provider’s key
    in “Keychain Access”, the API **does not** turn itself on when the app opens.
  - The price of each provider is written in the app with its date (2026-09-26); if the provider changes it, the
    cost you see drifts until the next version.

### The local door: let Claude Code operate the app · New · Sprint 003

- **What it does:** it lets **Claude Code**, on your own Mac and in your session, operate the app through a
  command, `ghost`: search your corpus, reindex it, run the evaluation kit with your questions,
  read and change some preferences and open your saved notes. **It starts closed**, you open it, and
  **in a meeting or during a rehearsal it closes itself** and denies everything: an agent never touches what
  is alive in memory. It does not go out to the network —it is a local channel of the Mac, which only your user can reach— and the counter stays at 0 B.
- **How to use it:**
  1. Build `ghost` once, next to the app: `pnpm ghost` in the project folder.
  2. In *AI*, **“Local door · closed”** (in the “Draft suggestions” row) → the door view opens → the
     **“closed”** switch opens it.
  3. The view shows the command, with **“Copy”**: the path of `ghost` and `--help`. Paste it to Claude
     Code; `ghost --help` explains to it what it can ask for.
  4. **The first time Claude Code uses the door, macOS asks you** whether `ghost` may use the
     information stored under “Angel Ghost · puerta” in your keychain, with your Mac’s password. That is
     the key: if you do not give it, it does not get in. It asks again **once each time you open the door**.
  5. **“What your agent did”** lists every order, **including the denied ones**, with its time and its reason, without
     the content: it says “ghost corpus buscar”, not what was searched.
  6. To close it, the same switch. It also closes when you quit the app.
- **What it can ask for** (`ghost --help`): `corpus buscar <texto>` · `corpus reindexar` ·
  `kit <preguntas.json>` (for example, `docs/kit-de-prueba/preguntas.json`) · `prefs leer` ·
  `prefs cambiar <clave> <valor>`, only with `idioma-consultor`, `idioma-cliente`, `retencion`,
  `ventana-de-la-bandeja` and `lectura-automatica` · `notas listar` · `notas abrir <archivo>`, which asks
  you for **Touch ID or your password on the Mac** once each time you open the door: even if you have
  exported in *Notes*, the door asks for its own.
- **What it can never do:** anything in a meeting (if the app is listening, if there is a meeting in notes-only mode, if it sees
  an open video call **or if it cannot know**) or during a rehearsal; turn on the external API;
  change “Draft suggestions”, the external provider, “Keep my turns” or what you answered about an NDA; open itself
  by itself; touch another machine. All of that is denied and stays in the log.
- **If it closed itself:** *AI* says so in amber, “It closed itself: there is a meeting or a rehearsal”. Open it again
  yourself when it ends; it does not open by itself.
- **Known limitations:**
  - `ghost` **is not installed in your PATH**: it lives next to the app you built (and inside the `.app` if you
    package it), and the door view gives you its path. Installing it in the PATH arrives with the signed app.
  - It searches, reindexes and measures over **the corpus the app has indexed**: on startup, the app re-reads
    in the background the folder it remembers; if you have just opened it, wait for *Corpus* to say how
    many documents there are. Adding folders is done by hand, in *Corpus*.
  - The kit measures **the search** (nDCG@5 and what it does not find); it does not compare models.
  - The log keeps the last 50 orders of this session of the app; three are shown and the rest
    scrolls. On exit, it is erased.

### Audio-only mode: the card, in your ear · New · Sprint 002

- **What it does:** it **reads the card aloud to you** while the band shrinks to a single line, so that
  you get back the full meeting screen. It is what you asked for in the design: *“let it talk to me
  in parallel in case I want to see the screen completely and it does not interrupt me”*.
- **How to use it:**
  1. `⌃⌥V` **turns the mode on**. The band drops from 88 to 44 px, the meeting window gets
     bigger, and the app reads you the card you have in front of you: the headline, the line and **where it comes from**.
  2. From then on, every time the client finishes speaking and a new card appears, it reads it
     to you by itself. There is nothing to press.
  3. `⎋` **silences** it on the spot, without waiting for the sentence to end.
  4. `⌃⌥V` again **turns the mode off** and the band goes back to its 88 px. Dragging the handle upward
     does the same: the height *is* the mode.
- **When it goes quiet by itself, and why:**
  - **If the sound comes out of your Mac’s speakers**, it does not speak and tells you in amber: “Plug in
    headphones · the client would hear you”. It is the mode’s lock.
  - **While someone is speaking** in the meeting —the client or you—. It never speaks over anyone. In
    *in person* mode it does not skip this: it waits for the room to go quiet, up to 4 s.
  - **While it is already saying another card.** It does not queue one behind another.
  - And `⌥⎋` silences it and turns the mode off, like everything else.
- **Known limitations:**
  - **Over HDMI, DisplayPort or AirPlay** —a monitor’s speaker, a television— **it does not speak**: those
    connections are almost always a speaker, and the app treats them like the Mac’s speakers.
  - **With Bluetooth or USB headphones —AirPods, for example— the app speaks, and it cannot be
    sure they are headphones.** macOS does not tell a headset apart from a desk speaker
    connected by the same cable: it only knows for certain when the sound comes out of the Mac’s internal speaker.
    The app draws the line there —just like the echo warning in *Session*— and tells you here instead of
    promising what it cannot know. If your output is an external speaker, **do not turn on the
    mode**.
  - **`⎋` stops reaching the meeting while the mode is on.** In Google Meet it is the
    key that exits full screen. It is taken only while the mode lasts and given back when you
    turn it off.
  - **The “I have nothing on…” cards are not read.** Their headline is the client’s words, and
    sending them out through the speaker would be reading you the transcript. The band does draw them.
  - **If your Mac has no voice for your language**, the mode does not turn on and says so in the log.
  - It reads in **your** language —that of your track—, because the card comes from your documents.

### Spanish and English, in everything · since Sprint 001

- **What it does:** the whole app, including the texts macOS shows you when asking for permissions, is in
  both languages. It follows the system’s.

### Your technical dictionary · New · Sprint 002

- **What it does:** the macOS speech engine does not know your jargon. “Power BI” comes out as “power by”, “DAX” comes
  out as “the ax” and “Lakehouse” comes out as “lake house” — and each of those is a card that does not arrive,
  because the app searches your documents for the wrong word. The dictionary **corrects the text
  after transcribing**, on your Mac, without a model and without anything leaving your machine.
- **How to use it:** by doing nothing. It comes with the data jargon already in (Power BI · DAX · Microsoft
  Fabric · Semantic Model · Lakehouse) and **your clients’ names come out on their own from your corpus** —
  if you have a card for “Páramo Azul”, the app already knows how to write it right.
- **Where to see it · new in phase 3 of Sprint 002:** *Language* shows how many terms it has, how many
  came from your corpus and how many from your file, and the full path of the file.
- **And if you want to add your own:** the file is
  `~/Library/Application Support/com.aiapps.copiloto-consultor/diccionario.yaml`, and you can edit it
  with any text editor. One line per term:

  ```
  Power BI: [power bi, powerbi, power by]
  Lakehouse: [lake house]
  Contabilidad Regulatoria:
  ```

  On the left, **how you want to see it written**; in brackets, the ways it is misheard. A
  term without brackets also works: it is corrected by similarity. Changes are applied **when the next
  session starts** — there is no need to close the app.
- **How much it improves, measured:** on the test audio with jargon, transcription goes from 46% of
  wrong words to 42% in Spanish, and from 35% to 26% in English. On the audios **without**
  jargon nothing changes, which is just as important: it does not touch what was already right.
- **Known limitations:**
  - **It fixes what was heard as similar, not what was lost.** “Power B” becomes “Power BI”,
    but if the engine heard “Lakehouse” as “en la que usé”, there is nothing left to resemble.
  - **Terms of six letters or fewer are only corrected if you write how they sound.** “DAX”
    is one letter away from “das”, “dos”, “tax” and “max”, and “fabric” one from “fábrica”: correcting by
    similarity there would break more than it fixes. And a client name of fewer than eight letters
    is only corrected if it was heard exactly.
  - **Your clients’ names are not saved in the file.** They come out of your corpus every time you
    start a session, so that file contains no one’s name.
  - **Each track listens to one language.** If in the middle of a Spanish sentence the client says three
    words in English, the dictionary fixes the jargon it recognizes, but **a whole sentence in the
    other language is not transcribed well** — it is measured and it is said on the *Language* screen. Marking
    several languages per track is not in the app.

## Angel Ghost on Windows · Sprints 006, 007 and 008

It is not a new feature but the same app on another system, which is why it sits outside “Features”.

- **What it does:** the app installs on a Windows PC. It has the protected band, the microphone, local
  transcription with whisper, your encrypted notes and your corpus with its cards. **Since sprint 007 it also
  handles the video call on that same PC** (Teams, Zoom or Meet in the browser): both tracks, the card in your ear, reading
  the meeting screen, rehearsal with voice and the shortcuts. **Since sprint 008, everything else:** docking,
  the suggestion with the API, the radar, opening your notes with Windows Hello and deletion with the app closed. Honesty
  has the block **“On Windows today”**, which says what it does and what it does not, each thing with its reason.
- **How it installs:** it arrives as a file (`.msi` or `.exe`) from the repository's continuous integration. **The
  installer is not signed yet:** Windows SmartScreen will show “Windows protected your PC”. Click **More
  info** and then **Run anyway**. If your
  account is not an administrator, ask whoever administers the PC to install it.
- **Which Windows you need:** **Windows 10 version 2004 (build 19041) or later**. On an earlier version Windows cannot
  hide the band from screen capture, and Angel Ghost does not open it without its protection: it says so with a dialog
  at startup and closes.
- **The microphone:** Windows asks for nothing from the app. You grant or revoke it yourself in **Settings → Privacy &
  security → Microphone**. In Permissions, the **Open Windows Settings** button takes you there.
- **The speech model:** in Language, **Install** downloads the large whisper model (1.6 GB). Before it is used, its
  SHA-256 checksum is verified; if it does not match, it is deleted. **It is never downloaded with a meeting, a session (also “notes-only” or in person) or a rehearsal open, and if one starts while it downloads, the download stops** and what it had is deleted. If the connection hangs, it lets go by itself after three hours. How long whisper took on your PC is remembered when you reopen the app.
- **Your PC's speed:** Honesty measures how long transcription takes on 10 seconds of sample audio. If it
  goes over 4 s, it says so and offers to download the small model (0.5 GB): it is less accurate, but faster on that PC.
- **Your notes:** they are saved encrypted, with the key in the Credential Manager of your user account. Exporting them,
  opening your rehearsals and opening another session's tray ask for **Windows Hello** (your PIN, fingerprint or face) once
  per app session.
- **What we do not verify:** that the band does not show up in Teams, Zoom or Meet. Honesty says it as “not verified”
  until you test it on your PC, and the band and _Session_ repeat it in those words: on Windows they do not say “Verified”
  nor name the Mac.

### The video call on Windows · Sprint 007

- **The two tracks.** With the meeting open, **Start session** opens your microphone (that is you) and the **system
  audio** (the client): what plays on your PC. Windows asks for no permission for this, so Permissions says so:
  _Needs no permission on Windows_. The app **does not tell voices apart**: the whole system track counts as the client's, even
  a notification or a song playing at that moment. **If you switch audio output with the session running**
  (Bluetooth headphones, for example), the app reopens the system audio on its own, within a couple of seconds; if
  the client transcript stops moving after the switch, report it.
- **Which meeting it recognizes.** Teams or Zoom **when you are on a call**: having the app open is not enough, because on
  Windows they stay open all day (sprint 008). The app knows because Teams or Zoom have the microphone open;
  Windows reports it per program and without asking for permission, and the app hears nothing because of it. Meet, in Chrome, Edge, Firefox or
  Brave, by the tab title. From the browsers it reads **only the titles of their windows**, nothing else on the PC.
  Each program in the catalog carries the official page it comes from; none is taken as verified. **Not verified:** whether
  Teams keeps the microphone open when you mute yourself inside the call; if not, the meeting may stop being detected
  while you are muted.
- **The app's own voice does not trigger cards for you.** What the app says through your speakers comes back through the system audio;
  if it matches a client turn, that turn is marked as an echo and searches for nothing.
- **The card in your ear** uses the Windows voice. It needs an installed voice for the language (Settings → Time & language →
  Speech). **With speakers, or with an HDMI or S/PDIF output, it does not speak**: the client would hear it. With headphones, it does.
  The app trusts what Windows declares about the output (speakers, headphones, HDMI); if an output is not classified,
  it says so instead of assuming. “In your ear” is only marked when the voice can speak; otherwise the card shows on the band. The
  **rehearsal** uses the same voice.
- **The screen.** It reads the meeting window, **only when it changes**, with Windows text recognition.
  It needs the text language pack (Settings → Time & language → Language & region); without it, it says so. The image
  lives in memory and is discarded once read.
- **If your PC cannot keep up.** Transcription serves the two tracks one after the other and **the client goes first**. If
  more than 4 turns pile up, **your** oldest ones are discarded first and, if none of yours is left (in the room the
  track has no owner), the oldest turn; Honesty counts how many, without saying whose they were.
  How long your PC takes to transcribe is stated by Honesty; the continuous integration figure is from another machine and does not count.
- **The shortcuts.** On Windows they are `Alt+Shift` and a key; none uses `Ctrl+Alt`, which on many keyboards is AltGr. If another
  app already has one of those combinations, Windows will not let it be registered: the app says so in Honesty and the action can still be done
  from the app. Word and Excel use some `Alt+Shift` combinations, and while one of them is in the foreground
  it may take it; what happens on your PC gets settled in the test.

| Key           | What it does                                                                                                                                                                                         |
| ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Alt+Shift+Q` | cuts everything: audio, transcript, band and backdrop                                                                                                                                                |
| `Alt+Shift+H` | “help me with this”                                                                                                                                                                                  |
| `Alt+Shift+C` | shows or hides the transcript next to the card                                                                                                                                                       |
| `Alt+Shift+W` | turns audio-only mode on or off                                                                                                                                                                      |
| `Alt+Shift+D` | reads the screen once, now                                                                                                                                                                           |
| `Alt+Shift+U` | what the radar sees on your PC (since sprint 008)                                                                                                                                                    |
| `Alt+Shift+K` | take a note                                                                                                                                                                                          |
| `Alt+Shift+F` | pin the card                                                                                                                                                                                         |
| `Alt+Shift+Z` | save the suggestion                                                                                                                                                                                  |
| `Alt+Shift+X` | the band, top or bottom                                                                                                                                                                              |
| `Esc`         | silences the voice. **It only exists while audio-only mode is on**: with the mode on, the app keeps `Esc` and it does not reach Teams, Zoom or your other apps until you turn it off (`Alt+Shift+W`) |

### Windows complete · Sprint 008

- **Docking.** With the band at the top, on **Start session** the meeting window moves down to the band and shrinks by the
  same amount, with its controls where they were; at the bottom, the window in front shrinks down to the band. On close it goes back
  to its place, and if the app crashes, it goes back the next time you open it. **Needs no permission on Windows**: Permissions says so.
  If you resize it by hand in the meantime, yours wins and it is not touched. Not verified: a maximized window
  docks without being restored; and Teams or Zoom may refuse to shrink: then it is undone and the band floats on top.
- **Windows Hello.** Opening your saved notes, your rehearsals or another session's tray asks for your PIN, your fingerprint or your
  face, once per app session. If your PC does not have Windows Hello set up, the app tells you where to set it up
  (Settings → Accounts → Sign-in options) and your notes stay encrypted.
- **What expires is deleted with the app closed.** The app leaves a task in the Windows **Task Scheduler**
  that runs at the minute of each expiry, even if you do not open the app; if the PC was off, it runs when
  it comes back. What runs is PowerShell over a list kept by the app: it only deletes expired `.ghost` files, writes nothing
  and does not launch the app. When nothing is left to expire, the task removes itself. If a policy from your company blocks PowerShell,
  Honesty says it did not run.
- **The radar.** It looks at the programs on **this PC** against its Windows catalog —exam proctoring, employee
  monitoring, remote access such as TeamViewer, AnyDesk or Quick Assist— and whether the PC is managed by your company
  (MDM). Each row carries the public page the program's name comes from and the date it was read. It looks at nothing
  outside your PC, blocks nothing and closes nothing: it warns. `Alt+Shift+U` shows what it sees.
- **The suggestion.** On Windows there is no system model: the suggestion is written by the **API** you choose in AI (Groq
  or Claude), with your key stored in the Credential Manager, and only anonymized text leaves your PC, just
  as on the Mac. Without the API, the card is still the answer. Each send is counted in Honesty.

| Key                          | On Windows today                                                                                                        |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `banda-protegida`            | yes, not verified on Teams, Zoom or Meet until you test it on your PC                                                   |
| `presencial`                 | yes, with local transcription (whisper)                                                                                 |
| `corpus-y-ficha`             | yes                                                                                                                     |
| `notas-cifradas`             | yes: encrypted, with the key in the Credential Manager; exporting and opening the tray ask for Windows Hello            |
| `videollamada`               | yes: your microphone and the system audio, two tracks; not verified on Teams, Zoom or Meet until you test it on your PC |
| `pantalla`                   | yes: the meeting window, read only when it changes                                                                      |
| `voz-al-oido`                | yes, with headphones; it does not speak through speakers                                                                |
| `acople`                     | yes: the meeting window moves down and shrinks, and returns on close; no permission to grant                            |
| `ensayo`                     | yes: the system voice reads the questions                                                                               |
| `sugerencia`                 | yes, with the API and your key: Windows has no system model                                                             |
| `radar`                      | yes: the programs on this PC that watch you, and whether it is managed (MDM)                                            |
| `atajos`                     | yes: Alt+Shift and a key; none with Ctrl+Alt, which on many keyboards is AltGr                                          |
| `borrado-con-la-app-cerrada` | yes: the Task Scheduler, at the minute of each expiry                                                                   |
| `desbloqueo`                 | yes: Windows Hello, once per app session                                                                                |
| `puerta-local`               | not on Windows for now: the door for Claude Code                                                                        |

## Keyboard shortcuts on the Mac

These are the Mac's. On Windows they are different —`Alt+Shift` and a letter— and they are in _Angel Ghost on Windows_, above.

| Key   | What it does                                                                                                                                                                                                                           |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `⌥⎋`  | cuts everything: audio, transcript, band and backdrop. Gives back the meeting window                                                                                                                                                   |
| `⌃⌥A` | “help me with this”: looks for a card about the last thing the client said; **in notes-only mode, with the last line of your note**                                                                                                    |
| `⌃⌥T` | shows or hides the transcript next to the card; with the compact band, it enlarges it                                                                                                                                                  |
| `⌃⌥V` | turns **audio-only mode** on or off: it reads the card to you and the band shrinks to one line                                                                                                                                         |
| `⌃⌥L` | **reads the screen once, now** — also with automatic reading off                                                                                                                                                                       |
| `⌃⌥R` | **what it sees**: opens _Session_ with the radar table — which program on your Mac is watching you and what it can see                                                                                                                 |
| `⌃⌥N` | **take a note**: the notebook to the front, in _Notes_, with the cursor at the end of your note                                                                                                                                        |
| `⌃⌥P` | **pin** the card you see in the band: it stays in your notes, with its headline and its source; the band says “pinned”                                                                                                                 |
| `⌃⌥↵` | **save the suggestion** you see in the band: it goes into your archive                                                                                                                                                                 |
| `⎋`   | silences the voice — **only while audio-only mode is on**                                                                                                                                                                              |
| `⌃⌥B` | **the band, top or bottom**: repositions it; at the bottom it docks the window in front and at the top the meeting window, only with the session started (with the band closed after `⌥⎋`, only your choice changes). It is remembered |

On the _Rehearsal_ screen, and only there, the keys belong to the window: **Enter** ready · **R** repeat ·
**S** skip · **Esc** end. They do not fire with the cursor in a selector.

> **Why `⌃⌥` (Control + Option) and not `⌘⇧`.** Up to sprint 2 the keys were `⌘⇧`, and according to
> Zoom's documentation for Mac, in Zoom those keys mute your microphone (`⌘⇧A`), turn off your
> camera (`⌘⇧V`) and pause screen sharing (`⌘⇧T`). While Angel Ghost was open,
> Zoom stopped receiving them. And `⌘⇧T` reopens the last closed tab in the browser. Zoom, Meet and
> Teams do not document any `⌃⌥` key. The Zoom part has not been tested live: it is not installed on
> the Mac where the app is built.
>
> **If you use VoiceOver**, its commands also start with `⌃⌥`. With VoiceOver on, these
> keys may clash with its own. The same goes for apps that arrange windows with `⌃⌥`, such as Rectangle.
>
> **And watch out for `⎋`:** while audio-only mode is on, that key belongs to the app and does not
> reach the meeting. It is returned as soon as you turn the mode off with `⌃⌥V`.

## Frequently asked questions

**Can the client see the band if I share my screen?**
On Google Meet on macOS 26.6.2, with the band at the bottom, no: they will see your desktop background in that strip.
With the band at the top —its factory spot— it uses the same protection, but it is not verified. On Zoom and
Teams it is not checked either, and the app tells you so instead of promising it.

**Does it upload my documents anywhere?**
No. It reads them where they are and the index stays on your Mac. The _Honesty_ counter shows the bytes
that left your machine: **0** unless you turn on the external provider in _AI_, and even then only
short, anonymized text leaves: with “Draft suggestions”, the client's last sentence and three short
cards; with “Enrich the bank”, the titles and first sentence of the sections of your proposal and of
the client's card, once per rehearsal —never a whole document—.

**Does it store what is said in the meeting?**
No. The audio lives thirty seconds in memory and is overwritten; the text, the last twelve turns.
On close, nothing of what was said remains except what you decide: your note, your agreements, your
pinned cards, the suggestions you save (the client's, a one-line fact, never their sentence) and
—only if you turn on “Keep my turns”— what **you** said, as text. Encrypted, in one file
per meeting. What you do not decide waits in the tray and deletes itself (3 h by default, 24 h at most).

**Does the rehearsal record my voice?**
No. Your voice passes through memory to be transcribed and is overwritten; the audio never touches the disk. If you save the
rehearsal, what remains is your answers as text and their figures, encrypted with your notes' key and with its
retention. If you do not save it, nothing remains on close. And with a video call open it does not start if the sound comes out
through the Mac's speakers, HDMI, DisplayPort or AirPlay. Over Bluetooth or USB the app cannot tell headphones
from a speaker: with a speaker like that, or if you open the call midway through the rehearsal, your microphone would
hear it and save it as your answer. Do not rehearse with a call playing through a speaker.

**Do I need internet?**
Only for the video call. The app transcribes, searches and —with the system model— drafts inside
your Mac. It touches the network in two cases, and you turn on both: when you ask it to install a language's
speech model —the **“Install”** button in _Language_; then macOS downloads it and nothing leaves
from here—, and if you turn on the **external provider** in _AI_, with your key.

**Why does it ask me for headphones?**
With speakers, your microphone also hears the client and the two tracks get mixed. The app
detects it and flags it, but it works better with headphones. In the rehearsal, with a video call open, it does not
start through the Mac's speakers, a monitor's (HDMI or DisplayPort) or AirPlay: your microphone would hear the
call and the call would hear the rehearsal voice. With wired, Bluetooth or USB headphones it does start; since the app
cannot tell Bluetooth or USB headphones from a speaker, with a speaker like that do not rehearse with a call
open.

## History

| Sprint | Features added to this manual                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 001    | the protected band · docking · the two tracks and local transcription · the indexed corpus · the evidence card and the suggestion on how to conduct yourself · a language's speech model · the cut and the Honesty screen · Spanish and English                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 002    | the silence trigger · **your technical dictionary** · **audio-only mode** · **screen reading** and `⌃⌥L` · **the whys** (the track that did not open, the audio output by name, the missing engine) · **why the card arrived and how long it took** · the keys become `⌃⌥` · **the radar** (amber and coral) and `⌃⌥R` · **the suggestion** and the **AI** screen                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 003    | **your notes** (the meeting notebook, the encrypted file per meeting, retention, export and delete) · `⌃⌥N` and `⌃⌥P` · **the protected notebook** while the meeting is open · **what went out to the API**, in _AI_ · **rule-based suggestions and the tray** with its countdown and its deletion with the app closed · `⌃⌥↵` · the “pinned” signal · the cut goes to 11 pieces · preferences are remembered · your notes move to the **app's private folder**, with “Show in Finder”, and delete themselves on expiry even if you do not open the app · **the frame in hand**: “This client”, its jurisdiction flag, the NDA check, the engagement-letter clause and **notes-only mode** · **the local door for Claude Code** (`ghost`), in _AI_                                                                 |
| 004    | **the rehearsal** (rule-based questions from your proposal, your card and the typical objections; the evaluation without a score: what you used, what you had and did not use, time, pace and fillers; the report; saving it with the key and retention of your notes; your progress per client; export and delete) · **“Enrich the bank”**, in _AI_ · **the band at the top**, next to the camera, with `⌃⌥B` and the first-time notice · docking at the top **moves down and shrinks only the meeting window**, when the session starts · the cut goes to 12 pieces · after its audit: with a video call open the rehearsal asks for headphones, _Session_ warns before discarding an unsaved rehearsal, the door also closes with a rehearsal and says so, and the protection at the top is declared unverified |
| 005    | **in person**: the room through the Mac's microphone, without system audio or screen; the room's language; the card in your ear by default, with the voice that waits for the room to go quiet; the rules measured with two synthetic voices so they do not fire on your voice; the Windows PC alongside; what it does not do (does not tell voices apart, does not save your turns, does not propose) · in English, a sentence that starts with “is”, “have” or “do” only counts as a question if they open it                                                                                                                                                                                                                                                                                                    |
| 006    | **Angel Ghost on Windows**: the protected band, the microphone, transcription with whisper (with its calibration on your PC and the small model as a fallback step), your encrypted notes in the Credential Manager, Windows microphone privacy and **“On Windows today”** in Honesty · the unsigned installer comes out of continuous integration                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 007    | **the video call on Windows**: system audio as the client's track, the meeting recognized by program, the card in your ear with the Windows voice (and rehearsal with voice), reading the meeting screen, the `Alt+Shift` shortcuts, the client first in the transcription · the band and _Session_ say “not verified” and do not name the Mac                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

> **Corrected after the sprint 001 audit** (2026-09-22): three sentences in this manual had
> stopped being true and were fixed with what the code actually did — the silence
> trigger, which did not exist then and was declared as a limitation (**it was wired in sprint 002**, and
> what this manual says above is what it does today); the installation of the speech model, which had no
> button and now has one; and the automatic card, which did not reach the band because of a defect in the
> bridge. The detail is in `sprints/SPRINT_001-auditoria.md`.
