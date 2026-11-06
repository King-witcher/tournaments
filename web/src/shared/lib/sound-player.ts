import { currentAudio, whenAudioReady, withRunningAudio } from "./audio";

/**
 * Toca arquivos de som curtos pelo Web Audio, em lotes (o que mudou de uma
 * vez na tela). Os efeitos e as falas de um lote começam juntos, na hora,
 * mesmo por cima do que já está tocando: nada espera numa fila. O anúncio do
 * lote (a vez, por exemplo) vem depois das falas dele (e de `announceAfter`,
 * para não cair embaixo de um efeito forte) e cancela o anúncio anterior: o
 * que ainda não começou não toca, e o que está tocando é cortado se for
 * `interruptible` (a vez que mudou de novo não espera a velha terminar);
 * senão, termina, e o novo vem depois. Lote sem anúncio, com `interrupt`, só
 * corta as partes `interruptible`. Um lote com `group` pode ser parado
 * depois (`stopGroup`: a contagem quando a pessoa age). O volume é o dos
 * arquivos, que já vêm acertados. Nada toca antes de um gesto da pessoa na
 * página (`audio.ts`).
 *
 * Som cujo arquivo demora mais de `LATE_MS` para chegar não toca, porque já
 * não diz respeito ao que está na tela. A música de fundo é à parte
 * (`playLoop`): repete até ser parada. Cada arquivo é baixado uma vez
 * (`preloadSounds` adianta, depois do primeiro gesto: antes, nada tocaria) e
 * decodificado no primeiro uso.
 */

const GAP_S = 0.1;
const LATE_MS = 2500;

const files = new Map<string, Promise<ArrayBuffer | null>>();
const buffers = new Map<string, Promise<AudioBuffer | null>>();
/** O que está tocando ou agendado, com o grupo do lote. */
const playing = new Map<AudioBufferSourceNode, string | undefined>();
/** Uma parte agendada do anúncio: de quando a quando, e se pode ser cortada. */
type Announced = { at: number; end: number; interruptible: boolean };
/** O anúncio que vale (os outros não tocam mais) e o que ele já agendou. */
let announcementId = 0;
let announcing = new Map<AudioBufferSourceNode, Announced>();
/** Muda a cada `stopSounds`: o que estava a caminho não toca mais. */
let generation = 0;
/** Muda a cada `stopGroup` do grupo: o lote dele a caminho não toca mais. */
const groupGenerations = new Map<string, number>();

function download(url: string) {
  let file = files.get(url);
  if (!file) {
    file = fetch(url)
      .then((response) => (response.ok ? response.arrayBuffer() : null))
      .catch(() => null);
    files.set(url, file);
    // Falhou (sem rede): tenta de novo no próximo uso.
    file.then((data) => {
      if (!data) files.delete(url);
    });
  }
  return file;
}

/** Baixa os arquivos antes do primeiro uso, quando o som puder tocar. */
export function preloadSounds(urls: Iterable<string>) {
  const list = [...urls];
  whenAudioReady(() => {
    for (const url of list) download(url);
  });
}

function decode(audio: AudioContext, url: string) {
  let buffer = buffers.get(url);
  if (!buffer) {
    buffer = download(url)
      .then((data) => (data ? audio.decodeAudioData(data.slice(0)) : null))
      .catch(() => null);
    buffers.set(url, buffer);
    buffer.then((decoded) => {
      if (!decoded) buffers.delete(url);
    });
  }
  return buffer;
}

const decodeAll = (audio: AudioContext, urls: readonly string[]) =>
  Promise.all(urls.map((url) => decode(audio, url)));

function stop(sources: Iterable<AudioBufferSourceNode>) {
  for (const source of sources) {
    try {
      source.stop();
    } catch {
      // Já tinha parado.
    }
  }
}

/**
 * Toca `loaded` em sequência a partir de `at`; devolve quando termina tudo.
 * Com `announced`, cada parte entra no anúncio, com o `interruptible` dela, e
 * a parte `overlap` não segura a seguinte, que começa junto.
 */
function sequence(
  audio: AudioContext,
  loaded: (AudioBuffer | null)[],
  at: number,
  group: string | undefined,
  announced?: {
    into: Map<AudioBufferSourceNode, Announced>;
    lines: readonly AnnouncementLine[];
  },
) {
  let start = at;
  let end = at;
  loaded.forEach((buffer, i) => {
    if (!buffer) return;
    const line = announced?.lines[i];
    const source = audio.createBufferSource();
    source.buffer = buffer;
    source.connect(audio.destination);
    source.onended = () => {
      playing.delete(source);
      announced?.into.delete(source);
    };
    playing.set(source, group);
    const finish = start + buffer.duration + GAP_S;
    announced?.into.set(source, {
      at: start,
      end: finish,
      interruptible: line?.interruptible ?? false,
    });
    source.start(start);
    end = Math.max(end, finish);
    if (!line?.overlap) start = finish;
  });
  return end;
}

/**
 * Tira do ar o anúncio anterior: o que não começou e o que pode ser cortado.
 * Devolve quando termina o que fica tocando (ou agora).
 */
function replaceAnnouncement(audio: AudioContext) {
  const now = audio.currentTime;
  let hold = now;
  for (const [source, part] of announcing) {
    if (part.at > now || part.interruptible) stop([source]);
    else hold = Math.max(hold, part.end);
  }
  announcing = new Map();
  return hold;
}

/**
 * Uma parte do anúncio; `interruptible`: o anúncio seguinte a corta no meio;
 * `overlap`: a parte seguinte começa junto com ela, não depois (o efeito da
 * vez com a fala dela).
 */
export type AnnouncementLine = {
  url: string;
  interruptible?: boolean;
  overlap?: boolean;
};

/** O que tocar de uma vez. */
export type SoundBatch = {
  effects?: readonly string[];
  /**
   * Cada uma em sequência por dentro e, entre si, juntas com os efeitos (ou
   * `delay` segundos depois: várias de uma vez, para cada uma se ouvir).
   */
  voices?: readonly { urls: readonly string[]; delay?: number }[];
  /** Depois das falas do lote, no lugar do anúncio anterior. */
  announcement?: readonly AnnouncementLine[];
  /** O anúncio começa no mínimo tantos segundos depois do lote. */
  announceAfter?: number;
  /**
   * O lote deixa velho o que o anúncio anterior pedia: as partes
   * `interruptible` param, mesmo sem anúncio novo.
   */
  interrupt?: boolean;
  /** Para parar o lote depois, com `stopGroup`. */
  group?: string;
};

export function playBatch({
  effects = [],
  voices = [],
  announcement = [],
  announceAfter = 0,
  interrupt = false,
  group,
}: SoundBatch) {
  const audio = currentAudio();
  if (!audio) return;
  const asked = performance.now();
  const startAt = audio.currentTime;
  const current = generation;
  const groupCurrent = group ? groupGenerations.get(group) : undefined;
  const late = () =>
    current !== generation ||
    (group != null && groupGenerations.get(group) !== groupCurrent) ||
    performance.now() - asked > LATE_MS;

  for (const url of effects) {
    decode(audio, url).then((buffer) => {
      if (!buffer || late()) return;
      withRunningAudio((running) =>
        sequence(running, [buffer], running.currentTime, group),
      );
    });
  }

  let id = 0;
  let hold = 0;
  if (announcement.length > 0) {
    id = ++announcementId;
    hold = replaceAnnouncement(audio);
  } else if (interrupt) {
    for (const [source, part] of announcing) {
      if (!part.interruptible) continue;
      stop([source]);
      announcing.delete(source);
    }
  }
  if (voices.length === 0 && id === 0) return;
  Promise.all([
    Promise.all(voices.map((voice) => decodeAll(audio, voice.urls))),
    decodeAll(
      audio,
      announcement.map((line) => line.url),
    ),
  ]).then(([spoken, announced]) => {
    if (late()) return;
    withRunningAudio((running) => {
      const now = running.currentTime;
      let end = Math.max(now, hold, startAt + announceAfter);
      spoken.forEach((loaded, i) => {
        const at = Math.max(now, startAt + (voices[i].delay ?? 0));
        end = Math.max(end, sequence(running, loaded, at, group));
      });
      if (id !== 0 && id === announcementId) {
        sequence(running, announced, end, group, {
          into: announcing,
          lines: announcement,
        });
      }
    });
  });
}

/** Uma música em repetição: o arquivo e a duração do original, em segundos. */
export type LoopSound = { url: string; seconds: number };

/**
 * O atraso do codificador no MP3 (LAME: 576 amostras, mais as 529 do
 * decodificador, a 44,1 kHz).
 */
const ENCODER_DELAY_S = 1105 / 44100;

/**
 * Onde a volta começa e termina no buffer. O MP3 decodificado traz o atraso
 * do codificador no começo e o enchimento no fim (os do site saem sem o
 * cabeçalho LAME completo, que deixaria o navegador cortá-los), e a música
 * vai do fim do atraso até a duração do original (conferido no Chromium
 * contra o original). O buffer com a duração do original já veio cortado.
 * Sem isso, cada volta teria um buraco.
 */
function loopBounds(buffer: AudioBuffer, seconds: number): [number, number] {
  const start = buffer.duration - seconds < 0.005 ? 0 : ENCODER_DELAY_S;
  return [start, Math.min(buffer.duration, start + seconds)];
}

/**
 * Toca `sound` em repetição, entrando aos poucos (`fadeIn` segundos) depois
 * de `delay` segundos; devolve a função que para, saindo aos poucos em
 * `fadeOut` segundos. Antes do primeiro gesto na página, espera por ele, e
 * parar antes disso desiste. Ao contrário dos lotes, não tem `LATE_MS`: ela
 * acompanha um estado (o veto em andamento), não um evento. `stopSounds`
 * também a para.
 */
export function playLoop(
  sound: LoopSound,
  { delay = 0, fadeIn = 0 }: { delay?: number; fadeIn?: number } = {},
) {
  let stopped = false;
  let started: { source: AudioBufferSourceNode; gain: GainNode } | undefined;
  const current = generation;
  const cancel = whenAudioReady(() => {
    const audio = currentAudio();
    if (!audio) return;
    const from = audio.currentTime;
    decode(audio, sound.url).then((buffer) => {
      if (!buffer || stopped || current !== generation) return;
      const source = audio.createBufferSource();
      source.buffer = buffer;
      source.loop = true;
      [source.loopStart, source.loopEnd] = loopBounds(buffer, sound.seconds);
      const gain = audio.createGain();
      const at = Math.max(audio.currentTime, from + delay);
      gain.gain.setValueAtTime(0, at);
      gain.gain.linearRampToValueAtTime(1, at + fadeIn);
      source.connect(gain).connect(audio.destination);
      source.onended = () => playing.delete(source);
      playing.set(source, undefined);
      source.start(at, source.loopStart);
      started = { source, gain };
    });
  });
  return (fadeOut = 0) => {
    if (stopped) return;
    stopped = true;
    cancel();
    if (!started) return;
    const { source, gain } = started;
    const now = source.context.currentTime;
    gain.gain.cancelScheduledValues(now);
    gain.gain.setValueAtTime(gain.gain.value, now);
    gain.gain.linearRampToValueAtTime(0, now + fadeOut);
    try {
      source.stop(now + fadeOut);
    } catch {
      // Já tinha parado.
    }
  };
}

/** Para o que os lotes de `group` tocam ou ainda vão tocar. */
export function stopGroup(group: string) {
  groupGenerations.set(group, (groupGenerations.get(group) ?? 0) + 1);
  stop([...playing].filter(([, of]) => of === group).map(([source]) => source));
}

/** Para o que está tocando e o que estava a caminho (ao silenciar). */
export function stopSounds() {
  generation++;
  stop(playing.keys());
  playing.clear();
  announcing = new Map();
}
