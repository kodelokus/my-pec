// Every user-facing string in English, in one place, ready for translation. strings.id.ts has the same shape.

import type { RendererProblem } from "./avatar/renderer";
import type { Activity } from "./bindings/Activity";
import type { AppError } from "./bindings/AppError";
import type { AvatarProblem } from "./bindings/AvatarProblem";
import type { FileKind } from "./bindings/FileKind";
import type { KeyCheckOutcome } from "./bindings/KeyCheckOutcome";
import type { MarkKind } from "./bindings/MarkKind";
import type { MemoryMark } from "./bindings/MemoryMark";
import type { Mood } from "./bindings/Mood";
import type { ProviderError } from "./bindings/ProviderError";
import type { SemanticProblem } from "./bindings/SemanticProblem";
import type { SendRefusal } from "./bindings/SendRefusal";
import type { SetupStep } from "./bindings/SetupStep";
import type { WeatherCondition } from "./bindings/WeatherCondition";
import type { WeatherView } from "./bindings/WeatherView";

const activities: Record<Activity, string> = {
  idle: "idle",
  listening: "listening",
  thinking: "thinking",
  speaking: "speaking",
  sleeping: "asleep",
};

export const KEY_ADDRESS = "https://platform.claude.com/settings/keys";

export const steps: Record<SetupStep, string> = {
  Key: "API key",
  Soul: "Personality",
  Avatar: "Look",
  AboutYou: "About you",
};

/** Names of the world's items by permanent id (world FR-011). */
const worldItems: Record<string, string> = {
  "home.cottage": "Cottage",
  "home.burrow": "Burrow",
  "grass.tuft": "Tuft of grass",
  "grass.tall": "Tall grass",
  "flower.daisy": "Daisy",
  "flower.poppy": "Poppy",
  "flower.buttercup": "Buttercup",
  "flower.bluebell": "Bluebell",
  "clover.patch": "Clover",
  "pebble.round": "Round pebble",
  "pebble.flat": "Flat stone",
  "pebble.cluster": "Pebbles",
  "mushroom.red": "Red mushroom",
  "mushroom.brown": "Brown mushrooms",
  "moss.patch": "Moss",
  "waterweed.sprig": "Waterweed",
  "bush.round": "Bush",
  "bush.berry": "Berry bush",
  "bush.flowering": "Flowering bush",
  "flower.sunflower": "Sunflower",
  "sapling.oak": "Oak sapling",
  "sapling.pine": "Pine sapling",
  "fern.small": "Fern",
  "fern.large": "Large fern",
  "log.fallen": "Fallen log",
  "stump.old": "Old stump",
  "reed.cattail": "Cattails",
  "reed.cluster": "Reeds",
  "pond.small": "Pond",
  "rock.small": "Rock",
  "rock.mossy": "Mossy rock",
  "rock.flat": "Flat rock",
  "tree.oak": "Oak tree",
  "tree.pine": "Pine tree",
  "tree.birch": "Birch tree",
  "tree.fruit": "Fruit tree",
  "tree.willow": "Willow",
  "hedge.short": "Hedge",
  "boulder.large": "Boulder",
  "boulder.split": "Split boulder",
  "crystal.cluster": "Crystals",
  "stream.piece": "Stream",
  "lily.pad": "Lily pad",
  "lily.flower": "Water lily",
  "bench.wood": "Bench",
  "well.stone": "Well",
  "fence.picket": "Fence",
  "lantern.post": "Lantern",
  "mailbox.post": "Mailbox",
  "garden.bed": "Garden bed",
  "path.stones": "Stepping stones",
  "birdhouse.pole": "Birdhouse",
  "windmill.small": "Windmill",
  "signpost.wood": "Signpost",
  "boat.rowing": "Rowing boat",
  "bridge.small": "Bridge",
  "laptop.open": "Open laptop",
  "briefcase.leather": "Briefcase",
  "lamp.desk": "Desk lamp",
  "folder.papers": "Folder of papers",
  "chair.office": "Office chair",
  "clipboard.notes": "Clipboard",
  "whiteboard.small": "Whiteboard",
  "book.stack": "Stack of books",
  "pencil.yellow": "Pencil",
  "backpack.school": "School backpack",
  "paper.test": "Test paper",
  "clock.alarm": "Alarm clock",
  "calculator.pocket": "Calculator",
  "sign.busstop": "Bus stop sign",
  "ticket.train": "Train ticket",
  "headphones.over": "Headphones",
  "car.small": "Small car",
  "cone.traffic": "Traffic cone",
  "key.car": "Car key",
  "suitcase.rolling": "Suitcase",
  "map.folded": "Folded map",
  "camera.travel": "Camera",
  "boots.hiking": "Hiking boots",
  "stick.walking": "Walking stick",
  "cap.sun": "Sun cap",
  "shoes.running": "Running shoes",
  "bottle.water": "Water bottle",
  "watch.sport": "Sports watch",
  "bicycle.red": "Bicycle",
  "helmet.bike": "Bike helmet",
  "ring.swim": "Swim ring",
  "goggles.swim": "Swim goggles",
  "towel.beach": "Beach towel",
  "ball.soccer": "Football",
  "dumbbell.iron": "Dumbbell",
  "racket.tennis": "Tennis racket",
  "pot.cooking": "Cooking pot",
  "pan.frying": "Frying pan",
  "spoon.wooden": "Wooden spoon",
  "cake.birthday": "Birthday cake",
  "plate.dinner": "Dinner plate",
  "bowl.noodles": "Bowl of noodles",
  "pizza.slice": "Slice of pizza",
  "cup.coffee": "Coffee cup",
  "teapot.round": "Teapot",
  "mug.tea": "Mug of tea",
  "bag.shopping": "Shopping bag",
  "cart.shopping": "Shopping cart",
  "basket.market": "Market basket",
  "broom.straw": "Broom",
  "bucket.mop": "Mop and bucket",
  "basket.laundry": "Laundry basket",
  "bed.small": "Bed",
  "pillow.soft": "Pillow",
  "hammock.garden": "Hammock",
  "bottle.medicine": "Medicine bottle",
  "thermometer.glass": "Thermometer",
  "bandage.roll": "Bandage",
  "frame.photo": "Photo frame",
  "table.dining": "Dining table",
  "quilt.patch": "Patchwork quilt",
  "blocks.toy": "Toy blocks",
  "ball.beach": "Beach ball",
  "kite.diamond": "Kite",
  "table.picnic": "Picnic table",
  "cards.playing": "Playing cards",
  "heart.balloon": "Heart balloon",
  "rose.vase": "Rose in a vase",
  "letter.love": "Love letter",
  "bowl.pet": "Pet bowl",
  "bone.dog": "Dog bone",
  "kennel.small": "Dog kennel",
  "book.open": "Open book",
  "glasses.reading": "Reading glasses",
  "notebook.spiral": "Notebook",
  "pen.fountain": "Fountain pen",
  "typewriter.old": "Typewriter",
  "guitar.acoustic": "Guitar",
  "piano.small": "Small piano",
  "popcorn.box": "Box of popcorn",
  "tv.small": "Television",
  "ticket.cinema": "Cinema ticket",
  "controller.game": "Game controller",
  "dice.pair": "Pair of dice",
  "yarn.ball": "Ball of yarn",
  "scissors.craft": "Scissors",
  "hammer.claw": "Hammer",
  "can.watering": "Watering can",
  "pot.plant": "Potted plant",
  "spade.garden": "Garden spade",
  "phone.mobile": "Mobile phone",
  "phone.old": "Old telephone",
  "tablet.screen": "Tablet",
  "coin.stack": "Stack of coins",
  "piggy.bank": "Piggy bank",
  "wallet.leather": "Wallet",
  "key.house": "House key",
  "door.front": "Front door",
  "sofa.small": "Sofa",
  "balloon.party": "Party balloons",
  "gift.box": "Gift box",
  "ring.wedding": "Wedding ring",
  "cake.wedding": "Wedding cake",
  "bouquet.flowers": "Bouquet",
  "candle.lit": "Lit candle",
  "wreath.leaves": "Wreath",
  "stone.memorial": "Memorial stone",
  "journal.closed": "Journal",
  "hourglass.sand": "Hourglass",
  "umbrella.open": "Umbrella",
  "boots.rain": "Rain boots",
  "snowman.small": "Snowman",
  "box.cardboard": "Cardboard box",
  "jar.glass": "Glass jar",
  "bag.paper": "Paper bag",
};

/** The subjects topics and everyday objects are filed under (journey data 2). */
const worldSubjects: Record<string, string> = {
  work: "Work",
  meeting: "A meeting",
  study: "Study",
  exam: "An exam",
  commute: "The commute",
  driving: "Driving",
  travel: "A trip",
  walking: "A walk",
  running: "A run",
  cycling: "Cycling",
  swimming: "Swimming",
  sport: "Sport and the gym",
  cooking: "Cooking",
  "eating-out": "Eating out",
  coffee: "Coffee and tea",
  shopping: "Shopping",
  chores: "Chores",
  rest: "Sleep and rest",
  health: "Illness and the doctor",
  family: "Family",
  children: "Children",
  friends: "Friends",
  love: "Love",
  pet: "A pet",
  reading: "Reading",
  writing: "Writing",
  music: "Music",
  film: "A film or show",
  games: "A game",
  crafts: "Making things",
  gardening: "Gardening",
  call: "A call",
  money: "Money",
  home: "The home",
  celebration: "A celebration",
  wedding: "A wedding",
  loss: "A loss",
  reflection: "Reflection",
  weather: "The weather",
  other: "Something else",
};

/** What the companion says it did with memory, on the reply that did it (memory FR-008). */
const markLabels: Record<MarkKind, string> = {
  note: "I noted",
  remembered: "I'll remember",
  profile: "I updated your profile",
  keptInNote: "My long-term memory is full, so I noted",
  noteMade: "I made a note",
  noteChanged: "I updated the note",
  alarmSet: "I set an alarm",
  alarmChanged: "I moved the alarm",
  alarmCancelled: "I cancelled the alarm",
};

// `resubmit`: the same key is worth sending again; otherwise the user edits it.
type Recovery = { message: string; action: string; resubmit: boolean };

/** onboarding FR-012: each outcome has its own message and its own way forward. */
export const keyOutcomes: Record<Exclude<KeyCheckOutcome["outcome"], "Accepted">, Recovery> = {
  Incomplete: {
    message: "That doesn't look like a complete key. Paste the whole key, without spaces in the middle.",
    action: "Fix the key",
    resubmit: false,
  },
  Rejected: {
    message: "Claude didn't accept this key. Check that you copied all of it, or create a new one.",
    action: "Enter another key",
    resubmit: false,
  },
  NoCredit: {
    message:
      "This key is genuine, but the account has no credit. Add credit in your Claude account, then I'll try again.",
    action: "Try again",
    resubmit: true,
  },
  Unreachable: {
    message:
      "I couldn't reach Claude, so your key hasn't been checked yet. Check your internet connection, then I'll try again.",
    action: "Retry",
    resubmit: true,
  },
  StorageUnavailable: {
    message:
      "Your key was accepted, but Windows Credential Manager wouldn't store it. I won't keep it anywhere else.",
    action: "Try again",
    resubmit: true,
  },
};

/** PER-9: one fixed sample line per personality template, in its own voice. The name follows the name field. */
export const samples: Record<string, { as: string; line: (name: string) => string }> = {
  "warm-friend": {
    as: "As a warm friend",
    line: (name) => `Hi, I'm ${name}. I'm here to listen, and I'll remember what matters to you.`,
  },
  "curious-scholar": {
    as: "As a curious scholar",
    line: (name) => `Hello, I'm ${name}. Ask me anything. I'll probably ask you something back.`,
  },
  "cheerful-sidekick": {
    as: "As a cheerful sidekick",
    line: (name) => `Hey! ${name} here. Ready when you are.`,
  },
  "calm-mentor": {
    as: "As a calm mentor",
    line: (name) => `Hello. I'm ${name}. Take your time, and we'll think it through together.`,
  },
  blank: {
    as: "With a blank page",
    line: (name) => `Hi, I'm ${name}. Who I am is up to you. Start writing.`,
  },
};

/** Each weather condition in a few words. */
const weatherWords: Record<WeatherCondition, string> = {
  clear: "Clear",
  partlyCloudy: "Partly cloudy",
  cloudy: "Cloudy",
  fog: "Fog",
  drizzle: "Drizzle",
  rain: "Rain",
  heavyRain: "Heavy rain",
  snow: "Snow",
  thunderstorm: "Thunderstorm",
};

export const t = {
  loading: "Loading…",
  next: "Next",
  back: "Back",
  meet: (name: string) => `Meet ${name}`,
  skip: "Skip for now",
  retry: "Retry",
  cancel: "Cancel",
  stepsLabel: "Setup steps",
  stepCaption: (n: number, total: number, step: string) => `Step ${n} of ${total} · ${step}`,

  /** PER-2, PER-3: the companion's interface lines, in one warm, neutral first-person voice. */
  line: {
    key: "Hi! I'm almost here. I think with Claude, so I need your own Claude API key before we can talk. You can create one at the address below.",
    checking: "One moment, I'm checking your key with Claude.",
    accepted: "That worked, thank you. Next, let's decide who I am.",
    existing: (name: string) =>
      `I found ${name} here from an earlier setup. Do you want to keep ${name}, or start fresh?`,
    soul: (as: string, sample: string) =>
      `${as} I'd sound like this: “${sample}” Pick another personality to hear it, or rewrite me below.`,
    overBudget: (budget: number) =>
      `This is more than I can use. Text beyond ${budget.toLocaleString()} characters won't be used in conversation. You can still continue.`,
    avatar: "Choose how I look.",
    avatarOnly: (name: string) =>
      `Choose how I look. Right now there is one look, ${name}, and it's already selected.`,
    about:
      "Two quick things, both optional. What should I call you? And which language would you like us to talk in?",
    finishing: "One moment, I'm getting everything ready.",
  },

  greeting: (callMe: string | null, hour: number) => {
    const part = hour >= 5 && hour < 12 ? "morning" : hour >= 12 && hour < 18 ? "afternoon" : "evening";
    return callMe ? `Good ${part}, ${callMe}` : `Good ${part}`;
  },
  presence: {
    here: (name: string) => `${name} is here`,
    offline: (name: string) => `${name} is offline`,
    noKey: (name: string) => `${name} is waiting for a key`,
    blocked: (name: string) => `${name} can't reply`,
  },

  copy: "Copy",
  copied: "Copied",
  keyLabel: "Claude API key",
  checkKey: "Check key",
  checking: "Checking…",
  checkingHint: "This can take up to 10 seconds.",
  keyStored: "It's stored only in Windows Credential Manager, never in a file.",
  keyAccepted: (lastFour: string) => `Key accepted (ends in ${lastFour}).`,
  changeKey: "Use a different key",

  keepCompanion: "Keep this companion",
  startFresh: "Start fresh",
  startFreshConfirm:
    "Start fresh? The current companion's files will be moved to a backup folder next to the workspace. Nothing is deleted.",

  templatesLabel: "Personality",
  soulTextLabel: "Personality text",
  discardConfirm: "Switch personality? Your edits to the current text will be lost.",
  discard: "Switch and discard edits",
  nameLabel: "Name",
  nameHint: (name: string) => `Leave empty to use ${name}.`,
  budget: (used: number, budget: number) =>
    `${used.toLocaleString()} of ${budget.toLocaleString()} characters`,

  charactersLabel: "Character",

  callMeLabel: (name: string) => `What ${name} calls you`,
  languageLabel: "Language for our conversations",

  conversationArea: "Conversation",
  noKey: "I can't talk until you add an API key.",
  addKey: "Add a key",
  problems: {
    Unreachable:
      "I can't reach Claude right now, so I can't reply. Check your internet connection and I'll try again.",
    KeyRejected:
      "Claude no longer accepts your API key, so I can't reply. Enter another key and we can carry on.",
    NoCredit:
      "Your Claude account has no credit left, so I can't reply. Add credit to the account, or enter another key.",
  } satisfies Partial<Record<ProviderError, string>>,
  enterAnotherKey: "Enter another key",
  recreated: (files: string[]) =>
    `These files were missing, so I recreated them from defaults: ${files.join(", ")}.`,
  settings: "Settings",

  settingsTitle: "Settings",
  languageSection: "Language",
  appLanguageLabel: "Language of the app",
  languageHint:
    "The language of buttons, settings and messages from the app. The companion replies in the language you write in, or the one in your profile.",
  keySection: "API key",
  keyPresent: (lastFour: string) => `A key is stored, ending in ${lastFour}.`,
  keyAbsent: "No key is stored.",
  replaceKey: "Replace key",
  keyKept: "Your current key is still in place.",
  keyAddressHint: `Create a key at ${KEY_ADDRESS}`,
  keyReplaced: "Key replaced.",
  removeKey: "Remove key",
  removeConfirm: "Remove the API key? My Pec can't talk to Claude until you add one again.",
  modelSection: "Model",
  memorySection: "Memory",
  dreamingSection: "Dreaming",
  dreamingLabel: "Dream at night",
  // journey FR-008.
  dreamWorldLabel: "Draw my days in the world",
  dreamWorldHint:
    "While dreaming, an everyday thing is drawn in the world for what you told about your day. Only what you wrote is read, and you can forget any story.",
  dreamingHint:
    "While the app runs, even in the tray, the companion goes over the day's notes once a night and keeps what keeps coming up. If the computer was asleep, it dreams a few minutes after you stop chatting.",
  dreamTimeLabel: "Dream time",
  dreamModelLabel: "Model for dreaming",
  recallOffDreams: "Bringing up related memories is off, so fewer notes come up again and dreams keep less.",
  // The avatar.
  avatar: {
    /** avatar R10: the name, the activity unless idle, the mood unless neutral. */
    label: (name: string, activity: Activity, mood: Mood) =>
      [name, activity === "idle" ? null : activities[activity], mood === "neutral" ? null : mood]
        .filter(Boolean)
        .join(", "),
    section: "Appearance",
    still: "Still portrait",
    picture: "Your own picture…",
    pictureHint: "A PNG, JPEG or WebP picture of up to 5 MB. It is copied into the companion's folder.",
    pictureRefused: {
      pictureType: "That file isn't a PNG, JPEG or WebP picture.",
      pictureSize: "That picture is larger than 5 MB.",
    } as Record<string, string>,
    floating: (name: string) => `Floating ${name}`,
    floatingHint: "A small window on your desktop, above the others. Click it to open the chat.",
    tryAgain: "Try again",
    /** avatar FR-017: why the window shows the still portrait (avatar R4). */
    problems: {
      missing: "The character's files are missing, so the still portrait is shown.",
      damaged: "The character's files couldn't be read, so the still portrait is shown.",
      noGraphics: "This computer's graphics can't draw the character, so the still portrait is shown.",
      notInBuild:
        "The animated character isn't part of this build of My Pec, so the still portrait is shown.",
      timedOut: "The character took too long to load, so the still portrait is shown.",
      stopped: "The character stopped drawing, so the still portrait is shown.",
    } as Record<RendererProblem, string>,
    /** Problems with the manifest itself (avatar FR-031, FR-032). */
    manifest: {
      Unreadable:
        "avatar/manifest.json couldn't be read, so the still portrait is shown. Choose a look to write it again.",
      MissingModel: "The character's files are missing, so the still portrait is shown.",
      CharacterGone:
        "The chosen character isn't part of this version of My Pec, so the default character is used.",
    } as Record<AvatarProblem, string>,
  },
  dreams: {
    open: "Dreams",
    title: (name: string) => `${name}'s dreams`,
    dreaming: (name: string) => `${name} is dreaming…`,
    line: (name: string) =>
      `At night ${name} goes over the notes and keeps what keeps coming up. This is the diary, and each night's dream.`,
    diaryLabel: "Dream diary",
    empty: "No dreams yet. The first one comes after a night with something to dream about.",
    runsLabel: "Nights",
    run: (added: number, merged: number, superseded: number) =>
      `${added} kept, ${merged} merged, ${superseded} replaced`,
    unfinished: "not finished",
    fellBack: "kept plainly",
    undone: "undone",
    tokens: (n: number) => `${n.toLocaleString()} tokens`,
    dreamNow: "Dream now",
    undo: "Undo last dream",
    undoneNote: "The last dream was undone; memory is as it was before it.",
    nothingToUndo: "There is no dream to undo.",
    changedConfirm: (files: string[]) =>
      `${files.join(" and ")} changed after the dream. Undoing it discards those changes too.`,
    undoAnyway: "Undo anyway",
  },

  world: {
    open: "World",
    title: (name: string) => `${name}'s world`,
    pictureLabel: (name: string) => `${name}'s world`,
    // world FR-036: only what the world file holds; no feelings, no absence, no frequency.
    line: (count: number, latest: string[]) =>
      count === 0
        ? "This is my home. Everything I draw grows out from here."
        : `I've drawn ${count} ${count === 1 ? "thing" : "things"} around my home. Most recently: ${latest.join(", ")}.`,
    summary: (name: string, count: number, latest: string[]) =>
      `${name}'s world: ${count} ${count === 1 ? "item" : "items"}${latest.length ? `, most recently ${latest.join(", ")}` : ""}.`,
    // journey FR-032: the newest day with topics, from the record alone.
    journeyLine: (date: string, picks: { name: string; keyword: string }[]) =>
      `For ${date} I drew ${picks.map((p) => `${p.name.toLowerCase()} for “${p.keyword}”`).join(" and ")}.`,
    about: "About this world",
    born: "Born",
    unknownBirth: "Not recorded",
    howItGrows:
      "A few things grow on each day you send a message, never more for talking more. During the night's dream I draw something for what happened in your day, so the map becomes a diary of your days. Nothing is ever removed or fades.",
    tipLabel: (name: string, date: string, keywords: string[]) => [name, date, ...keywords].join(", "),
    story: (name: string, date: string, keyword: string, line: string) =>
      `${name}, ${date}: “${keyword}”. ${line}`,
    whyThisOne: "Why this one",
    oddsCaption: (keyword: string) => `The things that could have been drawn for “${keyword}”`,
    oddsItem: "Thing",
    oddsChance: "Chance",
    picked: "drawn",
    related: "It came from a related subject.",
    rest: "Everything else",
    percent: (share: number) => `${Math.round(share * 100)}%`,
    sceneryLabel: "Also drawn that day",
    notes: "Notes from that day",
    forgetStory: "Forget this story",
    forgetStoryConfirm: (keyword: string, line: string) =>
      `Forget this story for good? “${keyword}” and “${line}” are removed. The drawing stays, with its date.`,
    storyForgotten: "The story was forgotten. The drawing stays.",
    forgottenStory: "A story you forgot.",
    timeLabel: "Show the world as it was on",
    timeValue: (date: string) => date,
    play: "Play",
    pause: "Pause",
    makeupTitle: "What your days are made of",
    makeupEmpty: "Nothing drawn for your days yet.",
    unknown: (id: string) => `Unknown item (${id})`,
    itemLabel: (name: string, date: string) => `${name}, drawn ${date}`,
    listTitle: "The diary",
    skipped: (count: number) =>
      `${count} line${count === 1 ? "" : "s"} of the world file couldn't be read and ${count === 1 ? "was" : "were"} left as they are.`,
    controls: {
      label: "World map. Use plus and minus to zoom, the arrow keys to move, and 0 to see the whole world.",
      zoomIn: "Zoom in",
      zoomOut: "Zoom out",
      whole: "Whole world",
    },
    name: (id: string) => worldItems[id] ?? id,
    subject: (id: string) => worldSubjects[id] ?? id,
  },
  memory: {
    title: "Memory",
    filesLabel: "Memory files",
    kinds: {
      soul: "Soul",
      profile: "Your profile",
      memory: "Long-term memory",
      daily: "Daily note",
      diary: "Dream diary",
    } satisfies Record<FileKind, string>,
    day: (date: string) =>
      new Date(`${date}T00:00:00`).toLocaleDateString(undefined, {
        weekday: "long",
        day: "numeric",
        month: "long",
        year: "numeric",
      }),
    unreadable: "can't be read",
    openFolder: "Open folder",
    searchLabel: "Search memory",
    search: "Search",
    searchUnavailable: "Search is unavailable right now. Try again in a moment.",
    noHits: "Nothing found.",
    empty: "This file is empty.",
    edit: "Edit",
    save: "Save",
    saved: "Saved.",
    undo: "Undo the companion's last change",
    undone: "The companion's change was undone.",
    nothingToUndo: "There's nothing left to undo.",
    undoEdited:
      "You've changed this file since the companion's change. Undo it anyway? Your changes since then will be lost.",
    conflict:
      "This file changed since you opened it. Reload it to see the change, or keep your version and save it over the change.",
    reload: "Reload",
    keepMine: "Keep my version",
    forget: "Forget",
    forgetDay: "Forget this day",
    forgetConfirm: (text: string) =>
      `Forget this for good? “${text}” is removed from memory and from search, and can't be brought back.`,
    forgotten: "Forgotten.",
    stillSeen:
      "The companion may still see this in our current conversation until a fresh context starts. The history stays.",
  },
  // Search by meaning (search FR-008, FR-009, FR-023).
  semantic: {
    title: "Search by meaning",
    hint: "Find memories by what they mean, not only by matching words, in any language.",
    offer: (size: string) =>
      `My Pec can also find memories by meaning, not just by matching words. This needs a one-time download of a small model (${size}) from Hugging Face (huggingface.co). It runs on this computer, and your memories never leave it.`,
    download: "Download",
    notNow: "Not now",
    cancel: "Cancel",
    tryAgain: "Try again",
    remove: "Remove the model",
    removeConfirm:
      "Remove the model? Search by meaning stops and the space is freed. You can download it again later.",
    switchLabel: "Find memories by meaning",
    downloading: (done: string, total: string) => `Downloading the model: ${done} of ${total}.`,
    preparing: (done: number, total: number) => `Preparing your memories: ${done} of ${total}.`,
    ready: (size: string) => `Ready. The model uses ${size} on this computer.`,
    off: "Off. Memories are found by matching words.",
    notDownloaded: "Not downloaded. Memories are found by matching words.",
    problems: {
      Damaged:
        "The download was damaged and has been removed. Memories are found by matching words until you try again.",
      NoSpace: "There isn't enough free disk space for the model.",
      Unreachable: "The model's download site can't be reached right now.",
      CouldNotLoad: "The model couldn't be started. Memories are found by matching words.",
    } satisfies Record<SemanticProblem, string>,
    size: (bytes: number) => `${Math.max(1, Math.round(bytes / 1_000_000))} MB`,
  },
  soundLabel: "Play a sound when a reply is done",
  version: (version: string) => `My Pec ${version}`,
  autoRecallLabel: "Bring up related memories automatically",
  autoRecallHint:
    "Before each reply, notes related to your message are looked up and given to the companion. When this is off, it searches only when it decides to.",
  modelLabel: "Model for conversation",
  modelStale: "This list may be out of date.",
  modelFallback: (from: string, to: string) => `${from} is no longer offered, so ${to} is selected instead.`,
  conversationSection: "Conversation",
  budgetLabel: "Working size (tokens)",
  budgetHint: (min: number, max: number) =>
    `When our conversation reaches this size, I summarize the older part and keep the recent turns. Smaller costs less per message. ${min.toLocaleString()} to ${max.toLocaleString()}.`,
  compactionModelLabel: "Model for summarizing",
  sameAsChat: "Same as the conversation model",
  compactionModelMissing: "The chosen model is no longer offered, so the conversation model is used instead.",

  chat: {
    composerLabel: (name: string) => `Message ${name}`,
    send: "Send",
    stop: "Stop",
    regenerate: "Regenerate",
    thinking: (name: string) => `${name} is thinking…`,
    waiting: (seconds: number) => `Claude is busy. Trying again in ${seconds} s…`,
    stopped: "Stopped",
    incomplete: "Incomplete",
    empty: (name: string) => `Say hello to ${name}.`,
    divider: "Fresh context",
    note: (file: string) => `${file} was updated`,
    earlier: "Loading earlier messages…",
    image: (alt: string) => `Image: ${alt || "untitled"}`,
    freshContext: "Start a fresh context",
    compacting: "Tidying up the older part of our conversation…",
    compacted: "Earlier conversation summarized",
    hideSummary: "Hide the summary",
    compactNow: "Compact now",
    compactNowMessage:
      "I'll summarize the older part of our conversation and keep the recent turns. Everything stays in the history.",
    focusLabel: "Anything I should make sure to keep? (optional)",
    nothingToCompact: "There's nothing to compact yet.",
    compactFailed: "I couldn't compact the conversation just now. Try again in a moment.",
    shortened: (files: string[]) =>
      `Only part of ${files.join(", ")} is given to the companion, because it is over its size budget. The file itself is unchanged.`,
    skipped: (count: number) =>
      `${count} line${count === 1 ? "" : "s"} of the conversation file couldn't be read and ${count === 1 ? "was" : "were"} skipped.`,
    interrupted: "The app closed before I finished replying. Your message is kept.",
    failures: {
      Unreachable: "I couldn't reach Claude. Your message is kept. Check your internet connection and retry.",
      Busy: "Claude is busy right now. Your message is kept. Try again in a moment.",
      KeyRejected: "Claude no longer accepts your API key, so I can't reply. Your message is kept.",
      NoCredit: "Your Claude account has no credit left, so I can't reply. Your message is kept.",
      ContextTooLong:
        "Our conversation is too long for Claude to read at once, even after tidying it up. Your message is kept. Retry, or start a fresh context.",
      Declined: "I couldn't answer that message. Try saying it another way.",
      Invalid: "Something went wrong sending this message. Try again.",
    } satisfies Record<ProviderError, string>,
    refusals: {
      Busy: "Wait for the reply to finish, or stop it.",
      Empty: "Type a message first.",
      TooLong: "This message is longer than 32,000 characters. Shorten it to send it.",
      NoKey: "Conversation is unavailable until you add an API key.",
    } satisfies Record<SendRefusal, string>,
    modelNote: "A new model takes effect when our conversation is next tidied up.",
    switchNow: "Switch now",
    updatePending:
      "I've learned a few new things since we started this conversation. I can use them once I tidy it up. The history stays here.",
    catchUp: "Tidy up now",
    outdated:
      "I can keep notes and remember things now, starting from a fresh context. The history stays here.",
    marksLabel: "Saved to memory",
    memoryFull:
      "My long-term memory is full. Until it's tidied, what I'm asked to remember goes to today's notes.",
    openInMemory: (path: string) => `Open ${path} in memory`,
    mark: (mark: MemoryMark) =>
      mark.ok ? `${markLabels[mark.kind]}: ${mark.excerpt}` : `I couldn't save this: ${mark.excerpt}`,
  },

  // Extensions.
  notes: {
    open: "Sticky notes",
    title: "Sticky notes",
    line: (name: string) =>
      `Your sticky notes, on the desktop or put away. ${name} can make them and read them when you ask.`,
    empty: "No notes yet. Make one here, or ask me to put something on a sticky.",
    newNote: "New note",
    untitled: "Empty note",
    show: "Show",
    hide: "Put away",
    delete: "Delete",
    deleteConfirm: (first: string) =>
      `Delete "${first}"? It moves to the notes folder's deleted folder, where you can get it back.`,
    shown: "On the desktop",
    hidden: "Put away",
    placeholder: "Write here…",
    colour: (c: string) => `Make it ${c}`,
    colours: { yellow: "yellow", green: "green", blue: "blue", pink: "pink", grey: "grey" },
    onTop: "Keep on top",
    hideNote: "Put the note away",
    conflict: "This note changed in another app, so your text was kept in a new note beside it.",
    tooLong: "A note holds at most 4,000 characters.",
  },
  alarms: {
    open: "Alarms",
    title: "Alarms",
    line: (name: string) =>
      `Reminders that ring with a Windows notification while My Pec runs. Ask ${name}, or add one here.`,
    empty: "No alarms yet.",
    add: "Add alarm",
    edit: "Edit",
    save: "Save alarm",
    cancel: "Cancel",
    delete: "Delete",
    snooze: "Snooze",
    done: "Done",
    labelLabel: "Remind me to",
    timeLabel: "Time",
    repeatLabel: "Repeat",
    dateLabel: "Day",
    daysLabel: "On",
    repeats: { once: "Once", daily: "Every day", weekdays: "On chosen days", weekly: "Every week" },
    days: { Mon: "Mon", Tue: "Tue", Wed: "Wed", Thu: "Thu", Fri: "Fri", Sat: "Sat", Sun: "Sun" },
    states: { waiting: "Waiting", snoozed: "Snoozed", done: "Done", missed: "Missed" },
    next: (when: string) => `Next: ${when}`,
    unreadable: (n: number) =>
      n === 1
        ? "One line in alarms.md couldn't be read. It's left as it is and won't ring."
        : `${n} lines in alarms.md couldn't be read. They're left as they are and won't ring.`,
    held: "Windows is holding back My Pec's notifications. Check Notifications in Windows Settings.",
    invalid: {
      label: "Write what to remind you of, in one line of up to 200 characters.",
      time: "Choose a time.",
      date: "Choose a day that hasn't passed.",
      days: "Choose at least one day.",
      key: "That alarm changed meanwhile. The list is up to date now.",
    } as Record<string, string>,
  },
  extensions: {
    section: "Extensions",
    hint: "Each one adds something the companion can do for you. Turning one off hides it and leaves its files as they are.",
    files: (folder: string) => `Files: ${folder}`,
    autostart: "Start My Pec when I sign in to Windows",
    autostartHint: "Alarms only ring while My Pec is running. It also lets the companion dream overnight.",
    offer: (name: string) =>
      `Alarms only ring while My Pec is running. Should I start when you sign in to Windows? It also lets ${name} dream overnight.`,
    yes: "Yes, start at sign-in",
    notNow: "Not now",
  },

  // The weather.
  weather: {
    conditions: weatherWords,
    line: (w: WeatherView) => `${weatherWords[w.condition]} · ${w.temperature}°`,
    cityLabel: "City",
    search: "Search",
    searching: "Searching…",
    noPlace: "No place found with that name.",
    unreachable: "I couldn't reach the weather service. Try again in a moment.",
    results: "Places found",
    chosen: (place: string) => `Showing the weather in ${place}.`,
    noneChosen: "Choose a city to see its weather.",
    units: "Units",
    celsius: "Celsius (°C)",
    fahrenheit: "Fahrenheit (°F)",
    updated: (time: string) => `Updated ${time}.`,
    failing: "Couldn't update the weather; trying again later.",
    unreadable: "The place in weather.md couldn't be read. Choose a place to replace it.",
    sent: "When you search, the city name you type is sent to Open-Meteo to find it. About every 30 minutes, the chosen place's coordinates are sent to Open-Meteo for its weather. Nothing else is sent.",
    credit: "Weather data by Open-Meteo.com",
  },

  closeNotice: "I'll be right here in the system tray. To quit, right-click my icon and choose Quit.",
  ok: "OK",

  errors: {
    WorkspaceNotWritable: (path: string) =>
      `I couldn't save my files to ${path}. Check that the folder can be written, then I'll try again.`,
    InvalidInput: (field: string) =>
      `Something in the form wasn't accepted (${field}). This is a bug in My Pec.`,
    Internal: () => "Something went wrong. Please try again.",
  },
};

export function errorMessage(error: AppError): string {
  switch (error.kind) {
    case "WorkspaceNotWritable":
      return t.errors.WorkspaceNotWritable(error.path);
    case "InvalidInput":
      return t.errors.InvalidInput(error.field);
    case "Internal":
      return t.errors.Internal();
  }
}
