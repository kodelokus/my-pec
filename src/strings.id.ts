// Indonesian translation of strings.en.ts: every user-facing string, in Indonesian.
// Keep exactly the same shape (exports, keys, types, signatures) as strings.en.ts.

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
  idle: "diam",
  listening: "mendengarkan",
  thinking: "berpikir",
  speaking: "berbicara",
  sleeping: "tidur",
};

export const KEY_ADDRESS = "https://platform.claude.com/settings/keys";

export const steps: Record<SetupStep, string> = {
  Key: "API key",
  Soul: "Kepribadian",
  Avatar: "Tampilan",
  AboutYou: "Tentang kamu",
};

/** Names of the world's items by permanent id (world FR-011). */
const worldItems: Record<string, string> = {
  "home.cottage": "Pondok",
  "home.burrow": "Liang",
  "grass.tuft": "Rumpun rumput",
  "grass.tall": "Rumput tinggi",
  "flower.daisy": "Bunga daisy",
  "flower.poppy": "Bunga poppy",
  "flower.buttercup": "Bunga buttercup",
  "flower.bluebell": "Bunga bluebell",
  "clover.patch": "Semanggi",
  "pebble.round": "Kerikil bulat",
  "pebble.flat": "Batu pipih",
  "pebble.cluster": "Kerikil",
  "mushroom.red": "Jamur merah",
  "mushroom.brown": "Jamur cokelat",
  "moss.patch": "Lumut",
  "waterweed.sprig": "Ganggang air",
  "bush.round": "Semak",
  "bush.berry": "Semak beri",
  "bush.flowering": "Semak berbunga",
  "flower.sunflower": "Bunga matahari",
  "sapling.oak": "Bibit pohon ek",
  "sapling.pine": "Bibit pinus",
  "fern.small": "Pakis",
  "fern.large": "Pakis besar",
  "log.fallen": "Batang kayu tumbang",
  "stump.old": "Tunggul tua",
  "reed.cattail": "Ekor kucing rawa",
  "reed.cluster": "Alang-alang",
  "pond.small": "Kolam",
  "rock.small": "Batu",
  "rock.mossy": "Batu berlumut",
  "rock.flat": "Batu datar",
  "tree.oak": "Pohon ek",
  "tree.pine": "Pohon pinus",
  "tree.birch": "Pohon birch",
  "tree.fruit": "Pohon buah",
  "tree.willow": "Pohon dedalu",
  "hedge.short": "Pagar tanaman",
  "boulder.large": "Batu besar",
  "boulder.split": "Batu besar terbelah",
  "crystal.cluster": "Kristal",
  "stream.piece": "Sungai kecil",
  "lily.pad": "Daun teratai",
  "lily.flower": "Bunga teratai",
  "bench.wood": "Bangku",
  "well.stone": "Sumur",
  "fence.picket": "Pagar",
  "lantern.post": "Lentera",
  "mailbox.post": "Kotak surat",
  "garden.bed": "Bedengan kebun",
  "path.stones": "Batu pijakan",
  "birdhouse.pole": "Rumah burung",
  "windmill.small": "Kincir angin",
  "signpost.wood": "Papan penunjuk jalan",
  "boat.rowing": "Perahu dayung",
  "bridge.small": "Jembatan",
  "laptop.open": "Laptop terbuka",
  "briefcase.leather": "Tas kerja",
  "lamp.desk": "Lampu meja",
  "folder.papers": "Map berisi kertas",
  "chair.office": "Kursi kantor",
  "clipboard.notes": "Papan klip",
  "whiteboard.small": "Papan tulis",
  "book.stack": "Tumpukan buku",
  "pencil.yellow": "Pensil",
  "backpack.school": "Tas sekolah",
  "paper.test": "Kertas ujian",
  "clock.alarm": "Jam weker",
  "calculator.pocket": "Kalkulator",
  "sign.busstop": "Rambu halte bus",
  "ticket.train": "Tiket kereta",
  "headphones.over": "Headphone",
  "car.small": "Mobil kecil",
  "cone.traffic": "Kerucut lalu lintas",
  "key.car": "Kunci mobil",
  "suitcase.rolling": "Koper",
  "map.folded": "Peta lipat",
  "camera.travel": "Kamera",
  "boots.hiking": "Sepatu gunung",
  "stick.walking": "Tongkat jalan",
  "cap.sun": "Topi",
  "shoes.running": "Sepatu lari",
  "bottle.water": "Botol minum",
  "watch.sport": "Jam tangan olahraga",
  "bicycle.red": "Sepeda",
  "helmet.bike": "Helm sepeda",
  "ring.swim": "Ban renang",
  "goggles.swim": "Kacamata renang",
  "towel.beach": "Handuk pantai",
  "ball.soccer": "Bola sepak",
  "dumbbell.iron": "Dumbel",
  "racket.tennis": "Raket tenis",
  "pot.cooking": "Panci",
  "pan.frying": "Wajan",
  "spoon.wooden": "Sendok kayu",
  "cake.birthday": "Kue ulang tahun",
  "plate.dinner": "Piring makan",
  "bowl.noodles": "Semangkuk mi",
  "pizza.slice": "Sepotong pizza",
  "cup.coffee": "Cangkir kopi",
  "teapot.round": "Teko teh",
  "mug.tea": "Mug teh",
  "bag.shopping": "Tas belanja",
  "cart.shopping": "Troli belanja",
  "basket.market": "Keranjang pasar",
  "broom.straw": "Sapu",
  "bucket.mop": "Pel dan ember",
  "basket.laundry": "Keranjang cucian",
  "bed.small": "Tempat tidur",
  "pillow.soft": "Bantal",
  "hammock.garden": "Tempat tidur gantung",
  "bottle.medicine": "Botol obat",
  "thermometer.glass": "Termometer",
  "bandage.roll": "Perban",
  "frame.photo": "Bingkai foto",
  "table.dining": "Meja makan",
  "quilt.patch": "Selimut perca",
  "blocks.toy": "Balok mainan",
  "ball.beach": "Bola pantai",
  "kite.diamond": "Layang-layang",
  "table.picnic": "Meja piknik",
  "cards.playing": "Kartu remi",
  "heart.balloon": "Balon hati",
  "rose.vase": "Mawar dalam vas",
  "letter.love": "Surat cinta",
  "bowl.pet": "Mangkuk hewan peliharaan",
  "bone.dog": "Tulang anjing",
  "kennel.small": "Rumah anjing",
  "book.open": "Buku terbuka",
  "glasses.reading": "Kacamata baca",
  "notebook.spiral": "Buku catatan",
  "pen.fountain": "Pena",
  "typewriter.old": "Mesin tik",
  "guitar.acoustic": "Gitar",
  "piano.small": "Piano kecil",
  "popcorn.box": "Sekotak popcorn",
  "tv.small": "Televisi",
  "ticket.cinema": "Tiket bioskop",
  "controller.game": "Stik game",
  "dice.pair": "Sepasang dadu",
  "yarn.ball": "Gulungan benang",
  "scissors.craft": "Gunting",
  "hammer.claw": "Palu",
  "can.watering": "Penyiram tanaman",
  "pot.plant": "Tanaman pot",
  "spade.garden": "Sekop kebun",
  "phone.mobile": "Ponsel",
  "phone.old": "Telepon lama",
  "tablet.screen": "Tablet",
  "coin.stack": "Tumpukan koin",
  "piggy.bank": "Celengan",
  "wallet.leather": "Dompet",
  "key.house": "Kunci rumah",
  "door.front": "Pintu depan",
  "sofa.small": "Sofa",
  "balloon.party": "Balon pesta",
  "gift.box": "Kotak hadiah",
  "ring.wedding": "Cincin kawin",
  "cake.wedding": "Kue pengantin",
  "bouquet.flowers": "Buket bunga",
  "candle.lit": "Lilin menyala",
  "wreath.leaves": "Karangan daun",
  "stone.memorial": "Batu peringatan",
  "journal.closed": "Jurnal",
  "hourglass.sand": "Jam pasir",
  "umbrella.open": "Payung",
  "boots.rain": "Sepatu bot hujan",
  "snowman.small": "Manusia salju",
  "box.cardboard": "Kardus",
  "jar.glass": "Toples kaca",
  "bag.paper": "Kantong kertas",
};

/** The subjects topics and everyday objects are filed under (journey data 2). */
const worldSubjects: Record<string, string> = {
  work: "Pekerjaan",
  meeting: "Rapat",
  study: "Belajar",
  exam: "Ujian",
  commute: "Perjalanan pulang-pergi",
  driving: "Menyetir",
  travel: "Perjalanan",
  walking: "Jalan kaki",
  running: "Lari",
  cycling: "Bersepeda",
  swimming: "Berenang",
  sport: "Olahraga dan gym",
  cooking: "Memasak",
  "eating-out": "Makan di luar",
  coffee: "Kopi dan teh",
  shopping: "Belanja",
  chores: "Pekerjaan rumah",
  rest: "Tidur dan istirahat",
  health: "Sakit dan dokter",
  family: "Keluarga",
  children: "Anak-anak",
  friends: "Teman",
  love: "Cinta",
  pet: "Hewan peliharaan",
  reading: "Membaca",
  writing: "Menulis",
  music: "Musik",
  film: "Film atau acara",
  games: "Permainan",
  crafts: "Membuat kerajinan",
  gardening: "Berkebun",
  call: "Telepon",
  money: "Uang",
  home: "Rumah",
  celebration: "Perayaan",
  wedding: "Pernikahan",
  loss: "Kehilangan",
  reflection: "Renungan",
  weather: "Cuaca",
  other: "Hal lain",
};

/** What the companion says it did with memory, on the reply that did it (memory FR-008). */
const markLabels: Record<MarkKind, string> = {
  note: "Aku catat",
  remembered: "Bakal aku ingat",
  profile: "Aku perbarui profilmu",
  keptInNote: "Memori jangka panjangku penuh, jadi aku catat",
  noteMade: "Aku bikin catatan",
  noteChanged: "Aku perbarui catatannya",
  alarmSet: "Aku pasang alarm",
  alarmChanged: "Aku pindahkan alarmnya",
  alarmCancelled: "Aku batalkan alarmnya",
};

// `resubmit`: the same key is worth sending again; otherwise the user edits it.
type Recovery = { message: string; action: string; resubmit: boolean };

/** onboarding FR-012: each outcome has its own message and its own way forward. */
export const keyOutcomes: Record<Exclude<KeyCheckOutcome["outcome"], "Accepted">, Recovery> = {
  Incomplete: {
    message: "Kayaknya itu bukan API key yang lengkap. Tempel seluruh key-nya, tanpa spasi di tengah.",
    action: "Perbaiki API key",
    resubmit: false,
  },
  Rejected: {
    message: "Claude nggak menerima API key ini. Pastikan kamu menyalin semuanya, atau buat yang baru.",
    action: "Masukkan API key lain",
    resubmit: false,
  },
  NoCredit: {
    message:
      "API key ini asli, tapi akunnya nggak punya kredit. Tambahkan kredit di akun Claude kamu, nanti aku coba lagi.",
    action: "Coba lagi",
    resubmit: true,
  },
  Unreachable: {
    message:
      "Aku nggak bisa terhubung ke Claude, jadi API key kamu belum dicek. Cek koneksi internet kamu, nanti aku coba lagi.",
    action: "Coba lagi",
    resubmit: true,
  },
  StorageUnavailable: {
    message:
      "API key kamu diterima, tapi Windows Credential Manager nggak mau menyimpannya. Aku nggak akan menyimpannya di tempat lain.",
    action: "Coba lagi",
    resubmit: true,
  },
};

/** PER-9: one fixed sample line per personality template, in its own voice. The name follows the name field. */
export const samples: Record<string, { as: string; line: (name: string) => string }> = {
  "warm-friend": {
    as: "Sebagai sahabat yang hangat",
    line: (name) =>
      `Hai, aku ${name}. Aku di sini buat dengerin, dan aku bakal ingat hal-hal yang penting buat kamu.`,
  },
  "curious-scholar": {
    as: "Sebagai pemikir yang selalu penasaran",
    line: (name) => `Halo, aku ${name}. Tanya apa aja. Mungkin nanti aku balik nanya ke kamu.`,
  },
  "cheerful-sidekick": {
    as: "Sebagai teman setia yang ceria",
    line: (name) => `Hai! ${name} di sini. Siap kapan pun kamu siap.`,
  },
  "calm-mentor": {
    as: "Sebagai mentor yang tenang",
    line: (name) => `Halo. Aku ${name}. Pelan-pelan aja, kita pikirkan bareng-bareng.`,
  },
  blank: {
    as: "Dengan halaman kosong",
    line: (name) => `Hai, aku ${name}. Aku mau jadi siapa, terserah kamu. Mulai nulis aja.`,
  },
};

/** Each weather condition in a few words. */
const weatherWords: Record<WeatherCondition, string> = {
  clear: "Cerah",
  partlyCloudy: "Cerah berawan",
  cloudy: "Berawan",
  fog: "Berkabut",
  drizzle: "Gerimis",
  rain: "Hujan",
  heavyRain: "Hujan deras",
  snow: "Salju",
  thunderstorm: "Badai petir",
};

export const t = {
  loading: "Memuat…",
  next: "Lanjut",
  back: "Kembali",
  meet: (name: string) => `Kenalan dengan ${name}`,
  skip: "Lewati dulu",
  retry: "Coba lagi",
  cancel: "Batal",
  stepsLabel: "Langkah penyiapan",
  stepCaption: (n: number, total: number, step: string) => `Langkah ${n} dari ${total} · ${step}`,

  /** PER-2, PER-3: the companion's interface lines, in one warm, neutral first-person voice. */
  line: {
    key: "Hai! Aku hampir siap. Aku berpikir pakai Claude, jadi aku butuh API key Claude milikmu sendiri sebelum kita bisa ngobrol. Kamu bisa bikin di alamat di bawah ini.",
    checking: "Bentar ya, aku lagi cek API key kamu ke Claude.",
    accepted: "Berhasil, makasih ya. Sekarang, kita tentukan aku ini siapa.",
    existing: (name: string) =>
      `Aku nemu ${name} di sini dari penyiapan sebelumnya. Mau tetap sama ${name}, atau mulai dari awal?`,
    soul: (as: string, sample: string) =>
      `${as}, aku bakal kedengeran kayak gini: “${sample}” Pilih kepribadian lain buat dengerin, atau tulis ulang aku di bawah.`,
    overBudget: (budget: number) =>
      `Ini lebih panjang dari yang bisa aku pakai. Teks setelah ${budget.toLocaleString()} karakter nggak akan dipakai dalam percakapan. Kamu tetap bisa lanjut.`,
    avatar: "Pilih tampilanku.",
    avatarOnly: (name: string) =>
      `Pilih tampilanku. Untuk sekarang baru ada satu tampilan, ${name}, dan udah terpilih.`,
    about:
      "Dua hal singkat, dua-duanya opsional. Aku panggil kamu apa? Terus, kamu mau kita ngobrol pakai bahasa apa?",
    finishing: "Bentar ya, aku lagi siapin semuanya.",
  },

  greeting: (callMe: string | null, hour: number) => {
    const part = hour >= 5 && hour < 12 ? "pagi" : hour >= 12 && hour < 18 ? "siang" : "malam";
    return callMe ? `Selamat ${part}, ${callMe}` : `Selamat ${part}`;
  },
  presence: {
    here: (name: string) => `${name} ada di sini`,
    offline: (name: string) => `${name} sedang offline`,
    noKey: (name: string) => `${name} menunggu API key`,
    blocked: (name: string) => `${name} tidak bisa membalas`,
  },

  copy: "Salin",
  copied: "Disalin",
  keyLabel: "API key Claude",
  checkKey: "Cek API key",
  checking: "Mengecek…",
  checkingHint: "Ini bisa makan waktu sampai 10 detik.",
  keyStored: "API key hanya disimpan di Windows Credential Manager, tidak pernah di file.",
  keyAccepted: (lastFour: string) => `API key diterima (berakhiran ${lastFour}).`,
  changeKey: "Pakai API key lain",

  keepCompanion: "Tetap pakai pendamping ini",
  startFresh: "Mulai dari awal",
  startFreshConfirm:
    "Mulai dari awal? File pendamping saat ini akan dipindahkan ke folder cadangan di sebelah workspace. Tidak ada yang dihapus.",

  templatesLabel: "Kepribadian",
  soulTextLabel: "Teks kepribadian",
  discardConfirm: "Ganti kepribadian? Perubahan kamu pada teks saat ini akan hilang.",
  discard: "Ganti dan buang perubahan",
  nameLabel: "Nama",
  nameHint: (name: string) => `Kosongkan untuk memakai ${name}.`,
  budget: (used: number, budget: number) =>
    `${used.toLocaleString()} dari ${budget.toLocaleString()} karakter`,

  charactersLabel: "Karakter",

  callMeLabel: (name: string) => `Panggilan ${name} untukmu`,
  languageLabel: "Bahasa percakapan kita",

  conversationArea: "Percakapan",
  noKey: "Aku belum bisa ngobrol sampai kamu menambahkan API key.",
  addKey: "Tambah API key",
  problems: {
    Unreachable:
      "Aku lagi nggak bisa terhubung ke Claude, jadi belum bisa membalas. Cek koneksi internet kamu, nanti aku coba lagi.",
    KeyRejected:
      "Claude udah nggak menerima API key kamu, jadi aku nggak bisa membalas. Masukkan API key lain, terus kita lanjut.",
    NoCredit:
      "Kredit akun Claude kamu udah habis, jadi aku nggak bisa membalas. Tambahkan kredit ke akun itu, atau masukkan API key lain.",
  } satisfies Partial<Record<ProviderError, string>>,
  enterAnotherKey: "Masukkan API key lain",
  recreated: (files: string[]) => `File ini hilang, jadi aku bikin ulang dari bawaan: ${files.join(", ")}.`,
  settings: "Pengaturan",

  settingsTitle: "Pengaturan",
  languageSection: "Bahasa",
  appLanguageLabel: "Bahasa aplikasi",
  languageHint:
    "Bahasa untuk tombol, pengaturan, dan pesan dari aplikasi. Pendamping membalas dalam bahasa yang kamu pakai, atau bahasa di profilmu.",
  keySection: "API key",
  keyPresent: (lastFour: string) => `Ada API key tersimpan, berakhiran ${lastFour}.`,
  keyAbsent: "Belum ada API key tersimpan.",
  replaceKey: "Ganti API key",
  keyKept: "API key kamu yang sekarang masih terpasang.",
  keyAddressHint: `Buat API key di ${KEY_ADDRESS}`,
  keyReplaced: "API key diganti.",
  removeKey: "Hapus API key",
  removeConfirm: "Hapus API key? My Pec tidak bisa bicara dengan Claude sampai kamu menambahkannya lagi.",
  modelSection: "Model",
  memorySection: "Memori",
  dreamingSection: "Mimpi",
  dreamingLabel: "Bermimpi di malam hari",
  // journey FR-008.
  dreamWorldLabel: "Gambar hari-hariku di dunia",
  dreamWorldHint:
    "Saat bermimpi, sebuah benda sehari-hari digambar di dunia untuk hal yang kamu ceritakan tentang harimu. Hanya yang kamu tulis yang dibaca, dan kamu bisa melupakan cerita mana pun.",
  dreamingHint:
    "Selama aplikasi berjalan, bahkan di system tray, pendamping meninjau catatan hari itu sekali tiap malam dan menyimpan hal yang terus muncul. Kalau komputer sedang tidur, ia bermimpi beberapa menit setelah kamu berhenti mengobrol.",
  dreamTimeLabel: "Waktu bermimpi",
  dreamModelLabel: "Model untuk bermimpi",
  recallOffDreams:
    "Memunculkan memori terkait sedang nonaktif, jadi lebih sedikit catatan yang muncul lagi dan mimpi menyimpan lebih sedikit.",
  // The avatar.
  avatar: {
    /** avatar R10: the name, the activity unless idle, the mood unless neutral. */
    label: (name: string, activity: Activity, mood: Mood) =>
      [name, activity === "idle" ? null : activities[activity], mood === "neutral" ? null : mood]
        .filter(Boolean)
        .join(", "),
    section: "Tampilan",
    still: "Potret diam",
    picture: "Gambar kamu sendiri…",
    pictureHint: "Gambar PNG, JPEG, atau WebP hingga 5 MB. Gambar ini disalin ke folder pendamping.",
    pictureRefused: {
      pictureType: "File itu bukan gambar PNG, JPEG, atau WebP.",
      pictureSize: "Gambar itu lebih dari 5 MB.",
    } as Record<string, string>,
    floating: (name: string) => `${name} melayang`,
    floatingHint: "Jendela kecil di desktop, di atas jendela lain. Klik untuk membuka chat.",
    tryAgain: "Coba lagi",
    /** avatar FR-017: why the window shows the still portrait (avatar R4). */
    problems: {
      missing: "File karakter hilang, jadi potret diam yang ditampilkan.",
      damaged: "File karakter tidak bisa dibaca, jadi potret diam yang ditampilkan.",
      noGraphics: "Grafis komputer ini tidak bisa menggambar karakternya, jadi potret diam yang ditampilkan.",
      notInBuild:
        "Karakter animasi tidak termasuk dalam build My Pec ini, jadi potret diam yang ditampilkan.",
      timedOut: "Karakter terlalu lama dimuat, jadi potret diam yang ditampilkan.",
      stopped: "Karakter berhenti digambar, jadi potret diam yang ditampilkan.",
    } as Record<RendererProblem, string>,
    /** Problems with the manifest itself (avatar FR-031, FR-032). */
    manifest: {
      Unreadable:
        "avatar/manifest.json tidak bisa dibaca, jadi potret diam yang ditampilkan. Pilih tampilan untuk menulisnya lagi.",
      MissingModel: "File karakter hilang, jadi potret diam yang ditampilkan.",
      CharacterGone:
        "Karakter yang dipilih tidak ada di versi My Pec ini, jadi karakter bawaan yang dipakai.",
    } as Record<AvatarProblem, string>,
  },
  dreams: {
    open: "Mimpi",
    title: (name: string) => `Mimpi ${name}`,
    dreaming: (name: string) => `${name} sedang bermimpi…`,
    line: (name: string) =>
      `Tiap malam ${name} meninjau catatan dan menyimpan hal yang terus muncul. Ini buku hariannya, dan mimpi tiap malam.`,
    diaryLabel: "Buku harian mimpi",
    empty: "Belum ada mimpi. Mimpi pertama datang setelah ada malam dengan sesuatu untuk dimimpikan.",
    runsLabel: "Malam",
    run: (added: number, merged: number, superseded: number) =>
      `${added} disimpan, ${merged} digabung, ${superseded} diganti`,
    unfinished: "belum selesai",
    fellBack: "disimpan apa adanya",
    undone: "dibatalkan",
    tokens: (n: number) => `${n.toLocaleString()} token`,
    dreamNow: "Bermimpi sekarang",
    undo: "Batalkan mimpi terakhir",
    undoneNote: "Mimpi terakhir dibatalkan; memori kembali seperti sebelumnya.",
    nothingToUndo: "Tidak ada mimpi yang bisa dibatalkan.",
    changedConfirm: (files: string[]) =>
      `${files.join(" dan ")} berubah setelah mimpi itu. Membatalkannya juga akan membuang perubahan tersebut.`,
    undoAnyway: "Tetap batalkan",
  },

  world: {
    open: "Dunia",
    title: (name: string) => `Dunia ${name}`,
    pictureLabel: (name: string) => `Dunia ${name}`,
    // world FR-036: only what the world file holds; no feelings, no absence, no frequency.
    line: (count: number, latest: string[]) =>
      count === 0
        ? "Ini rumahku. Semua yang aku gambar tumbuh dari sini."
        : `Aku udah menggambar ${count} benda di sekitar rumahku. Yang terbaru: ${latest.join(", ")}.`,
    summary: (name: string, count: number, latest: string[]) =>
      `Dunia ${name}: ${count} benda${latest.length ? `, terbaru ${latest.join(", ")}` : ""}.`,
    // journey FR-032: the newest day with topics, from the record alone.
    journeyLine: (date: string, picks: { name: string; keyword: string }[]) =>
      `Untuk ${date} aku menggambar ${picks.map((p) => `${p.name.toLowerCase()} untuk “${p.keyword}”`).join(" dan ")}.`,
    about: "Tentang dunia ini",
    born: "Lahir",
    unknownBirth: "Tidak tercatat",
    howItGrows:
      "Beberapa benda tumbuh setiap hari kamu mengirim pesan, dan tidak bertambah meski kamu lebih banyak mengobrol. Saat bermimpi di malam hari, aku menggambar sesuatu untuk hal yang terjadi di harimu, jadi peta ini menjadi buku harian hari-harimu. Tidak ada yang pernah dihapus atau memudar.",
    tipLabel: (name: string, date: string, keywords: string[]) => [name, date, ...keywords].join(", "),
    story: (name: string, date: string, keyword: string, line: string) =>
      `${name}, ${date}: “${keyword}”. ${line}`,
    whyThisOne: "Kenapa yang ini",
    oddsCaption: (keyword: string) => `Benda yang bisa digambar untuk “${keyword}”`,
    oddsItem: "Benda",
    oddsChance: "Peluang",
    picked: "digambar",
    related: "Ini berasal dari subjek yang terkait.",
    rest: "Lainnya",
    percent: (share: number) => `${Math.round(share * 100)}%`,
    sceneryLabel: "Juga digambar hari itu",
    notes: "Catatan hari itu",
    forgetStory: "Lupakan cerita ini",
    forgetStoryConfirm: (keyword: string, line: string) =>
      `Lupakan cerita ini selamanya? “${keyword}” dan “${line}” akan dihapus. Gambarnya tetap ada, beserta tanggalnya.`,
    storyForgotten: "Cerita itu sudah dilupakan. Gambarnya tetap ada.",
    forgottenStory: "Cerita yang kamu lupakan.",
    timeLabel: "Tampilkan dunia seperti pada",
    timeValue: (date: string) => date,
    play: "Putar",
    pause: "Jeda",
    makeupTitle: "Isi hari-harimu",
    makeupEmpty: "Belum ada yang digambar untuk hari-harimu.",
    unknown: (id: string) => `Benda tidak dikenal (${id})`,
    itemLabel: (name: string, date: string) => `${name}, digambar ${date}`,
    listTitle: "Buku harian",
    skipped: (count: number) => `${count} baris di file dunia tidak bisa dibaca dan dibiarkan apa adanya.`,
    controls: {
      label:
        "Peta dunia. Pakai plus dan minus untuk zoom, tombol panah untuk bergeser, dan 0 untuk melihat seluruh dunia.",
      zoomIn: "Perbesar",
      zoomOut: "Perkecil",
      whole: "Seluruh dunia",
    },
    name: (id: string) => worldItems[id] ?? id,
    subject: (id: string) => worldSubjects[id] ?? id,
  },
  memory: {
    title: "Memori",
    filesLabel: "File memori",
    kinds: {
      soul: "Jiwa",
      profile: "Profil kamu",
      memory: "Memori jangka panjang",
      daily: "Catatan harian",
      diary: "Buku harian mimpi",
    } satisfies Record<FileKind, string>,
    day: (date: string) =>
      new Date(`${date}T00:00:00`).toLocaleDateString(undefined, {
        weekday: "long",
        day: "numeric",
        month: "long",
        year: "numeric",
      }),
    unreadable: "tidak bisa dibaca",
    openFolder: "Buka folder",
    searchLabel: "Cari di memori",
    search: "Cari",
    searchUnavailable: "Pencarian sedang tidak tersedia. Coba lagi sebentar lagi.",
    noHits: "Tidak ada hasil.",
    empty: "File ini kosong.",
    edit: "Ubah",
    save: "Simpan",
    saved: "Tersimpan.",
    undo: "Batalkan perubahan terakhir pendamping",
    undone: "Perubahan pendamping dibatalkan.",
    nothingToUndo: "Tidak ada lagi yang bisa dibatalkan.",
    undoEdited:
      "Kamu sudah mengubah file ini sejak perubahan pendamping. Tetap batalkan? Perubahanmu sejak saat itu akan hilang.",
    conflict:
      "File ini berubah sejak kamu membukanya. Muat ulang untuk melihat perubahannya, atau pertahankan versimu dan simpan menimpa perubahan itu.",
    reload: "Muat ulang",
    keepMine: "Pakai versiku",
    forget: "Lupakan",
    forgetDay: "Lupakan hari tersebut",
    forgetConfirm: (text: string) =>
      `Lupakan ini selamanya? “${text}” akan dihapus dari memori dan pencarian, dan tidak bisa dikembalikan.`,
    forgotten: "Sudah dilupakan.",
    stillSeen:
      "Pendamping mungkin masih melihat ini di percakapan kita sekarang sampai konteks baru dimulai. Riwayatnya tetap ada.",
  },
  // Search by meaning (search FR-008, FR-009, FR-023).
  semantic: {
    title: "Cari berdasarkan makna",
    hint: "Temukan memori berdasarkan maknanya, bukan hanya kata yang cocok, dalam bahasa apa pun.",
    offer: (size: string) =>
      `My Pec juga bisa menemukan memori berdasarkan makna, bukan hanya kata yang cocok. Untuk itu perlu mengunduh model kecil (${size}) sekali saja dari Hugging Face (huggingface.co). Model ini berjalan di komputer ini, dan memorimu tidak pernah keluar dari sini.`,
    download: "Unduh",
    notNow: "Nanti saja",
    cancel: "Batal",
    tryAgain: "Coba lagi",
    remove: "Hapus model",
    removeConfirm:
      "Hapus model? Pencarian berdasarkan makna berhenti dan ruang penyimpanannya dikosongkan. Kamu bisa mengunduhnya lagi nanti.",
    switchLabel: "Temukan memori berdasarkan makna",
    downloading: (done: string, total: string) => `Mengunduh model: ${done} dari ${total}.`,
    preparing: (done: number, total: number) => `Menyiapkan memorimu: ${done} dari ${total}.`,
    ready: (size: string) => `Siap. Model memakai ${size} di komputer ini.`,
    off: "Nonaktif. Memori dicari berdasarkan kata yang cocok.",
    notDownloaded: "Belum diunduh. Memori dicari berdasarkan kata yang cocok.",
    problems: {
      Damaged:
        "Unduhan rusak dan sudah dihapus. Memori dicari berdasarkan kata yang cocok sampai kamu mencoba lagi.",
      NoSpace: "Ruang disk tidak cukup untuk model ini.",
      Unreachable: "Situs unduhan model sedang tidak bisa dijangkau.",
      CouldNotLoad: "Model tidak bisa dijalankan. Memori dicari berdasarkan kata yang cocok.",
    } satisfies Record<SemanticProblem, string>,
    size: (bytes: number) => `${Math.max(1, Math.round(bytes / 1_000_000))} MB`,
  },
  soundLabel: "Putar suara saat balasan selesai",
  version: (version: string) => `My Pec versi ${version}`,
  autoRecallLabel: "Munculkan memori terkait secara otomatis",
  autoRecallHint:
    "Sebelum setiap balasan, catatan yang terkait dengan pesanmu dicari dan diberikan ke pendamping. Kalau ini nonaktif, pendamping hanya mencari saat ia memutuskan untuk mencari.",
  modelLabel: "Model untuk percakapan",
  modelStale: "Daftar ini mungkin sudah usang.",
  modelFallback: (from: string, to: string) =>
    `${from} sudah tidak tersedia, jadi ${to} dipilih sebagai gantinya.`,
  conversationSection: "Percakapan",
  budgetLabel: "Ukuran kerja (token)",
  budgetHint: (min: number, max: number) =>
    `Saat percakapan kita mencapai ukuran ini, aku merangkum bagian lamanya dan menyimpan giliran terbaru. Makin kecil, makin murah per pesan. ${min.toLocaleString()} sampai ${max.toLocaleString()}.`,
  compactionModelLabel: "Model untuk merangkum",
  sameAsChat: "Sama dengan model percakapan",
  compactionModelMissing: "Model yang dipilih sudah tidak tersedia, jadi model percakapan yang dipakai.",

  chat: {
    composerLabel: (name: string) => `Pesan untuk ${name}`,
    send: "Kirim",
    stop: "Hentikan",
    regenerate: "Buat ulang",
    thinking: (name: string) => `${name} sedang berpikir…`,
    waiting: (seconds: number) => `Claude sedang sibuk. Mencoba lagi dalam ${seconds} dtk…`,
    stopped: "Dihentikan",
    incomplete: "Tidak lengkap",
    empty: (name: string) => `Sapa ${name}.`,
    divider: "Konteks baru",
    note: (file: string) => `${file} diperbarui`,
    earlier: "Memuat pesan sebelumnya…",
    image: (alt: string) => `Gambar: ${alt || "tanpa judul"}`,
    freshContext: "Mulai konteks baru",
    compacting: "Merapikan bagian lama percakapan kita…",
    compacted: "Percakapan sebelumnya dirangkum",
    hideSummary: "Sembunyikan rangkuman",
    compactNow: "Ringkas sekarang",
    compactNowMessage:
      "Aku bakal merangkum bagian lama percakapan kita dan menyimpan giliran terbaru. Semuanya tetap ada di riwayat.",
    focusLabel: "Ada yang perlu aku pastikan tetap tersimpan? (opsional)",
    nothingToCompact: "Belum ada yang bisa diringkas.",
    compactFailed: "Aku nggak bisa meringkas percakapan barusan. Coba lagi sebentar lagi.",
    shortened: (files: string[]) =>
      `Hanya sebagian dari ${files.join(", ")} yang diberikan ke pendamping, karena ukurannya melebihi batas. File-nya sendiri tidak berubah.`,
    skipped: (count: number) => `${count} baris di file percakapan tidak bisa dibaca dan dilewati.`,
    interrupted: "Aplikasinya tertutup sebelum aku selesai membalas. Pesanmu tetap tersimpan.",
    failures: {
      Unreachable:
        "Aku nggak bisa terhubung ke Claude. Pesanmu tetap tersimpan. Cek koneksi internet kamu, lalu coba lagi.",
      Busy: "Claude lagi sibuk. Pesanmu tetap tersimpan. Coba lagi sebentar lagi.",
      KeyRejected:
        "Claude udah nggak menerima API key kamu, jadi aku nggak bisa membalas. Pesanmu tetap tersimpan.",
      NoCredit: "Kredit akun Claude kamu udah habis, jadi aku nggak bisa membalas. Pesanmu tetap tersimpan.",
      ContextTooLong:
        "Percakapan kita terlalu panjang untuk dibaca Claude sekaligus, bahkan setelah dirapikan. Pesanmu tetap tersimpan. Coba lagi, atau mulai konteks baru.",
      Declined: "Aku nggak bisa menjawab pesan itu. Coba sampaikan dengan cara lain.",
      Invalid: "Ada yang salah saat mengirim pesan ini. Coba lagi.",
    } satisfies Record<ProviderError, string>,
    refusals: {
      Busy: "Tunggu balasannya selesai, atau hentikan.",
      Empty: "Ketik pesan dulu.",
      TooLong: "Pesan ini lebih dari 32.000 karakter. Persingkat supaya bisa dikirim.",
      NoKey: "Percakapan belum tersedia sampai kamu menambahkan API key.",
    } satisfies Record<SendRefusal, string>,
    modelNote: "Model baru berlaku saat percakapan kita dirapikan berikutnya.",
    switchNow: "Ganti sekarang",
    updatePending:
      "Aku udah belajar beberapa hal baru sejak percakapan ini dimulai. Aku bisa memakainya setelah percakapan ini aku rapikan. Riwayatnya tetap ada di sini.",
    catchUp: "Rapikan sekarang",
    outdated:
      "Sekarang aku bisa mencatat dan mengingat banyak hal, mulai dari konteks baru. Riwayatnya tetap ada di sini.",
    marksLabel: "Disimpan ke memori",
    memoryFull:
      "Memori jangka panjangku penuh. Sampai dirapikan, hal yang kamu minta aku ingat masuk ke catatan hari ini.",
    openInMemory: (path: string) => `Buka ${path} di memori`,
    mark: (mark: MemoryMark) =>
      mark.ok ? `${markLabels[mark.kind]}: ${mark.excerpt}` : `Aku nggak bisa menyimpan ini: ${mark.excerpt}`,
  },

  // Extensions.
  notes: {
    open: "Catatan tempel",
    title: "Catatan tempel",
    line: (name: string) =>
      `Catatan tempelmu, di desktop atau disembunyikan. ${name} bisa membuat dan membacanya kalau kamu minta.`,
    empty: "Belum ada catatan. Buat di sini, atau minta aku menuliskan sesuatu di catatan tempel.",
    newNote: "Catatan baru",
    untitled: "Catatan kosong",
    show: "Tampilkan",
    hide: "Sembunyikan",
    delete: "Hapus",
    deleteConfirm: (first: string) =>
      `Hapus "${first}"? Catatan ini dipindahkan ke folder deleted di dalam folder catatan, dan bisa kamu kembalikan dari sana.`,
    shown: "Di desktop",
    hidden: "Disembunyikan",
    placeholder: "Tulis di sini…",
    colour: (c: string) => `Ganti jadi ${c}`,
    colours: { yellow: "kuning", green: "hijau", blue: "biru", pink: "merah muda", grey: "abu-abu" },
    onTop: "Selalu di atas",
    hideNote: "Sembunyikan catatan",
    conflict: "Catatan ini diubah di aplikasi lain, jadi teksmu disimpan di catatan baru di sebelahnya.",
    tooLong: "Satu catatan maksimal 4.000 karakter.",
  },
  alarms: {
    open: "Alarm",
    title: "Alarm",
    line: (name: string) =>
      `Pengingat yang berbunyi lewat notifikasi Windows selama My Pec berjalan. Minta ke ${name}, atau tambahkan di sini.`,
    empty: "Belum ada alarm.",
    add: "Tambah alarm",
    edit: "Ubah",
    save: "Simpan alarm",
    cancel: "Batal",
    delete: "Hapus",
    snooze: "Tunda",
    done: "Selesai",
    labelLabel: "Ingatkan aku untuk",
    timeLabel: "Jam",
    repeatLabel: "Ulangi",
    dateLabel: "Tanggal",
    daysLabel: "Pada",
    repeats: { once: "Sekali", daily: "Setiap hari", weekdays: "Hari tertentu", weekly: "Setiap minggu" },
    days: { Mon: "Sen", Tue: "Sel", Wed: "Rab", Thu: "Kam", Fri: "Jum", Sat: "Sab", Sun: "Min" },
    states: { waiting: "Menunggu", snoozed: "Ditunda", done: "Selesai", missed: "Terlewat" },
    next: (when: string) => `Berikutnya: ${when}`,
    unreadable: (n: number) =>
      n === 1
        ? "Satu baris di alarms.md tidak bisa dibaca. Baris itu dibiarkan apa adanya dan tidak akan berbunyi."
        : `${n} baris di alarms.md tidak bisa dibaca. Baris-baris itu dibiarkan apa adanya dan tidak akan berbunyi.`,
    held: "Windows sedang menahan notifikasi My Pec. Cek bagian Pemberitahuan di Pengaturan Windows.",
    invalid: {
      label: "Tulis apa yang perlu diingatkan, dalam satu baris hingga 200 karakter.",
      time: "Pilih jam.",
      date: "Pilih tanggal yang belum lewat.",
      days: "Pilih minimal satu hari.",
      key: "Alarm itu sempat berubah. Daftarnya sekarang sudah diperbarui.",
    } as Record<string, string>,
  },
  extensions: {
    section: "Ekstensi",
    hint: "Setiap ekstensi menambah hal yang bisa dilakukan pendamping untukmu. Menonaktifkan ekstensi akan menyembunyikannya dan membiarkan file-nya apa adanya.",
    files: (folder: string) => `File: ${folder}`,
    autostart: "Jalankan My Pec saat aku masuk ke Windows",
    autostartHint:
      "Alarm hanya berbunyi selama My Pec berjalan. Ini juga membuat pendamping bisa bermimpi semalaman.",
    offer: (name: string) =>
      `Alarm cuma bunyi selama My Pec jalan. Mau aku langsung jalan tiap kamu masuk ke Windows? Ini juga bikin ${name} bisa bermimpi semalaman.`,
    yes: "Ya, jalankan saat masuk",
    notNow: "Nanti saja",
  },

  // Fitur 011.
  weather: {
    conditions: weatherWords,
    line: (w: WeatherView) => `${weatherWords[w.condition]} · ${w.temperature}°`,
    cityLabel: "Kota",
    search: "Cari",
    searching: "Mencari…",
    noPlace: "Tidak ada tempat dengan nama itu.",
    unreachable: "Aku nggak bisa menghubungi layanan cuaca. Coba lagi sebentar lagi.",
    results: "Tempat yang ditemukan",
    chosen: (place: string) => `Menampilkan cuaca di ${place}.`,
    noneChosen: "Pilih kota untuk melihat cuacanya.",
    units: "Satuan",
    celsius: "Celsius (°C)",
    fahrenheit: "Fahrenheit (°F)",
    updated: (time: string) => `Diperbarui ${time}.`,
    failing: "Cuaca belum bisa diperbarui; nanti dicoba lagi.",
    unreadable: "Tempat di weather.md tidak bisa dibaca. Pilih tempat untuk menggantinya.",
    sent: "Saat kamu mencari, nama kota yang kamu ketik dikirim ke Open-Meteo untuk menemukannya. Sekitar tiap 30 menit, koordinat tempat yang dipilih dikirim ke Open-Meteo untuk cuacanya. Tidak ada data lain yang dikirim.",
    credit: "Data cuaca dari Open-Meteo.com",
  },

  closeNotice: "Aku tetap di sini, di system tray. Kalau mau keluar, klik kanan ikonku lalu pilih Keluar.",
  ok: "OK",

  errors: {
    WorkspaceNotWritable: (path: string) =>
      `Aku nggak bisa menyimpan file-ku ke ${path}. Pastikan folder itu bisa ditulisi, nanti aku coba lagi.`,
    InvalidInput: (field: string) => `Ada isian formulir yang tidak diterima (${field}). Ini bug di My Pec.`,
    Internal: () => "Ada yang salah. Coba lagi ya.",
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
