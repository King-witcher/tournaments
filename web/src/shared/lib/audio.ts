/**
 * O `AudioContext` da página, um só para todos os sons (o aviso do `chime` e
 * os arquivos do `sound-player`). O navegador só deixa tocar depois de um
 * gesto da pessoa: `unlockAudioOnGesture` cria o contexto no primeiro clique
 * ou tecla (e o retoma se o sistema o suspender); antes disso não há contexto,
 * e nada toca.
 */

let context: AudioContext | undefined;
const waiting = new Set<() => void>();

function create() {
  if (context) return context;
  const Context =
    window.AudioContext ??
    (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Context) return undefined;
  context = new Context();
  for (const ready of waiting) ready();
  waiting.clear();
  return context;
}

/**
 * Chama `ready` quando o som puder tocar: na hora, se um gesto já criou o
 * contexto; senão, no primeiro gesto. Antes disso, nada a fazer (baixar um
 * arquivo que talvez nunca toque, por exemplo). Devolve a função que desiste
 * de esperar.
 */
export function whenAudioReady(ready: () => void) {
  if (context) ready();
  else waiting.add(ready);
  return () => {
    waiting.delete(ready);
  };
}

/** Prepara o som no próximo clique ou tecla; devolve a função que desfaz. */
export function unlockAudioOnGesture() {
  const unlock = () => {
    try {
      const audio = create();
      if (audio?.state === "suspended") audio.resume().catch(() => {});
    } catch {
      // Sem Web Audio: a página fica sem som.
    }
  };
  window.addEventListener("pointerdown", unlock, true);
  window.addEventListener("keydown", unlock, true);
  return () => {
    window.removeEventListener("pointerdown", unlock, true);
    window.removeEventListener("keydown", unlock, true);
  };
}

/** O contexto, se um gesto já o criou (para decodificar arquivos). */
export function currentAudio(): AudioContext | undefined {
  return context;
}

/**
 * Chama `play` com o áudio rodando: na hora ou, com ele acordando (o gesto
 * acabou de acontecer), se ele voltar em até 500 ms. Um som atrasado (o
 * sistema só devolve o áudio no próximo gesto) não toca.
 */
export function withRunningAudio(play: (audio: AudioContext) => void) {
  const audio = context;
  if (audio?.state === "running") play(audio);
  else if (audio?.state === "suspended") {
    const asked = performance.now();
    audio
      .resume()
      .then(() => {
        if (performance.now() - asked < 500) play(audio);
      })
      .catch(() => {});
  }
}
