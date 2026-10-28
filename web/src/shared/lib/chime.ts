/**
 * Um som curto gerado pelo Web Audio (sem arquivo), para avisos. O navegador
 * só deixa tocar depois de um gesto da pessoa na página: `unlockAudioOnGesture`
 * prepara o `AudioContext` no primeiro clique ou tecla (e o retoma se o
 * sistema o suspender), e `playChime` não toca nada antes disso.
 */

let context: AudioContext | undefined;

function audioContext() {
  if (context) return context;
  const Context =
    window.AudioContext ??
    (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Context) return undefined;
  context = new Context();
  return context;
}

/** Prepara o som no próximo clique ou tecla; devolve a função que desfaz. */
export function unlockAudioOnGesture() {
  const unlock = () => {
    try {
      const audio = audioContext();
      if (audio?.state === "suspended") audio.resume().catch(() => {});
    } catch {
      // Sem Web Audio: os avisos ficam sem som.
    }
  };
  window.addEventListener("pointerdown", unlock, true);
  window.addEventListener("keydown", unlock, true);
  return () => {
    window.removeEventListener("pointerdown", unlock, true);
    window.removeEventListener("keydown", unlock, true);
  };
}

/**
 * Dois bipes curtos subindo (~0,3 s), em volume baixo. Com o áudio ainda
 * acordando (o gesto acabou de acontecer), toca se ele voltar logo; um som
 * atrasado (o sistema só devolve o áudio no próximo gesto) não toca.
 */
export function playChime() {
  const audio = context;
  if (audio?.state === "running") beep(audio);
  else if (audio?.state === "suspended") {
    const asked = performance.now();
    audio
      .resume()
      .then(() => {
        if (performance.now() - asked < 500) beep(audio);
      })
      .catch(() => {});
  }
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
