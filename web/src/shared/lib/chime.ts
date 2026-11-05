import { withRunningAudio } from "./audio";

/**
 * Um som curto gerado pelo Web Audio (sem arquivo), para avisos: dois bipes
 * curtos subindo (~0,3 s), em volume baixo. Só toca depois de um gesto da
 * pessoa na página (`unlockAudioOnGesture`, em `audio.ts`).
 */
export function playChime() {
  withRunningAudio(beep);
}

function beep(audio: AudioContext) {
  const start = audio.currentTime;
  for (const [frequency, offset] of [
    [660, 0],
    [880, 0.12],
  ]) {
    const oscillator = audio.createOscillator();
    const gain = audio.createGain();
    oscillator.type = "sine";
    oscillator.frequency.value = frequency;
    const at = start + offset;
    gain.gain.setValueAtTime(0.0001, at);
    gain.gain.exponentialRampToValueAtTime(0.2, at + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, at + 0.18);
    oscillator.connect(gain).connect(audio.destination);
    oscillator.start(at);
    oscillator.stop(at + 0.2);
  }
}
