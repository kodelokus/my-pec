let audio: AudioContext | null = null;

/** A soft two-note chime when a reply is done, made on the spot so no sound file ships. Silent where Web Audio is missing. */
export function chime() {
  if (typeof AudioContext === "undefined") return;
  try {
    audio ??= new AudioContext();
    const now = audio.currentTime;
    for (const [offset, frequency] of [
      [0, 880],
      [0.09, 1320],
    ] as const) {
      const tone = audio.createOscillator();
      const level = audio.createGain();
      tone.type = "sine";
      tone.frequency.value = frequency;
      level.gain.setValueAtTime(0.0001, now + offset);
      level.gain.exponentialRampToValueAtTime(0.05, now + offset + 0.01);
      level.gain.exponentialRampToValueAtTime(0.0001, now + offset + 0.35);
      tone.connect(level).connect(audio.destination);
      tone.start(now + offset);
      tone.stop(now + offset + 0.4);
    }
  } catch {
    // A sound is a nicety; the reply is already on screen.
  }
}
