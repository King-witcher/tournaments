import { pickWeighted, type Weighted } from "@/shared/lib/pick-weighted";
import type {
  AnnouncementLine,
  LoopSound,
  SoundBatch,
} from "@/shared/lib/sound-player";
import type { VetoSoundEvent } from "./veto-sound-events";
import { VETO_SOUND_FILES as FILES } from "./veto-sound-files";

/*
 * Como os sons do veto convivem:
 *
 * - O passo (ban, pick, voto) toca o efeito e a fala dele juntos, na hora:
 *   a porta e o nome do mapa escolhido, o ban e a despedida do campeão, a
 *   confirmação e o nome do campeão. Passos seguidos se sobrepõem em vez de
 *   esperar numa fila (picks rápidos virariam uma lista de nomes atrasada).
 * - Várias falas de uma vez (a revelação às cegas): até 4, uma a cada
 *   0,35 s, para cada uma se ouvir.
 * - O anúncio (o que acontece agora: começo, votação, a vez, o resultado da
 *   votação, o fim) vem depois da fala do passo que o causou e da parte forte
 *   de um efeito sem fala (a porta que tranca cobriria "your pick"). Os de
 *   um mesmo passo saem em sequência ("vote passed", o mapa, "prepare to
 *   fight").
 * - A vez de quem vê (ban, pick, voto) toca o efeito da vez junto da fala
 *   dela ("your pick", "vote now"; no ban, que não tem fala, só o efeito).
 * - O que pede uma ação (a vez e o efeito dela, `enemys_pick`, `vote_now`)
 *   fica velho no próximo passo: é cortado no meio por ele, e o que ainda não
 *   começou não toca. O que conta algo (`prepare_draft`, `vote_passed` e o
 *   mapa, `prepare_to_fight`) termina, e o anúncio seguinte vem depois.
 * - A contagem é só de quem age e para assim que ele age ou a vez muda
 *   (`COUNTDOWN`), mesmo no meio do som.
 * - A música (`VETO_MUSIC`) repete enquanto o veto está em andamento, por
 *   baixo de tudo; no começo, entra depois da parte forte do "match found".
 */

/** O idioma do locutor: o do site (o inglês é o original do jogo). */
export type SoundLocale = keyof typeof FILES.announcer;

export const soundLocale = (locale: string): SoundLocale =>
  locale in FILES.announcer ? (locale as SoundLocale) : "en";

/** Um som do veto: o efeito e a fala do passo, juntos, e o anúncio. */
export type VetoCue = {
  effect?: string;
  voice?: readonly string[];
  announce?: readonly AnnouncementLine[];
};

/**
 * Quanto o anúncio do mesmo lote espera um efeito sem fala, pela parte forte
 * dele (o envelope medido): a porta que tranca, quase 0,8 s de ruído largo; o
 * voto, 0,5 s; o clique do ban de campeão, 0,3 s; o "match found", o golpe do
 * começo (o resto, mais baixo, fica embaixo do "prepare draft").
 */
const HOLD_S: Record<string, number> = {
  [FILES.effects.mapBan]: 0.7,
  [FILES.effects.vote]: 0.5,
  [FILES.effects.championBan]: 0.3,
  [FILES.effects.matchFound]: 0.6,
};

const lookup = (table: Record<string, string>, key: string) =>
  Object.hasOwn(table, key) ? table[key] : undefined;

const listed = (table: Record<string, readonly string[]>, key: string) =>
  Object.hasOwn(table, key) ? table[key] : [];

/**
 * As variantes de cada evento, com o peso de cada uma: uma sai por vez. O
 * locutor diz o mapa escolhido, não o banido, e no pick só o nome do campeão
 * (do usuário). Mapa ou campeão sem fala (os que o locutor não diz, um
 * campeão novo) fica só com o efeito.
 */
function variants(
  event: VetoSoundEvent,
  locale: SoundLocale,
): Weighted<VetoCue>[] {
  const { lines, maps, champions } = FILES.announcer[locale];
  const one = (cue: VetoCue) => [{ value: cue, weight: 1 }];
  const mapName = (slug: string) => {
    const name = lookup(maps, slug);
    return name ? [name] : [];
  };
  const tell = (...urls: string[]) =>
    one({ announce: urls.map((url) => ({ url })) });
  const ask = (url: string) =>
    one({ announce: [{ url, interruptible: true }] });
  switch (event.kind) {
    case "begin":
      return one({
        effect: FILES.effects.matchFound,
        announce: [{ url: lines.prepareDraft }],
      });
    case "mapBan":
      return one({ effect: FILES.effects.mapBan });
    case "mapPick":
      return one({ effect: FILES.effects.mapPick, voice: mapName(event.map) });
    case "vote":
      return one({ effect: FILES.effects.vote });
    case "voteNow":
      return ask(lines.voteNow);
    case "votePassed":
      return tell(lines.votePassed, ...mapName(event.map));
    case "championBan": {
      const effect = FILES.effects.championBan;
      const goodbye = listed(FILES.goodbye, event.champion);
      if (goodbye.length === 0) return one({ effect });
      return goodbye.map((line) => ({
        value: { effect, voice: [line] },
        weight: 1,
      }));
    }
    case "championPick": {
      const name = lookup(champions, event.champion);
      return one({
        effect: FILES.effects.championPick,
        voice: name ? [name] : [],
      });
    }
    case "yourTurn": {
      // O efeito começa junto da fala seguinte: "your pick" ou, no voto, o
      // "vote now" que vem logo depois no lote. O ban não tem fala.
      const effect = {
        url: FILES.effects.yourTurn,
        interruptible: true,
        overlap: true,
      };
      const spoken = event.action === "pick" ? [lines.yourPick] : [];
      return one({
        announce: [
          effect,
          ...spoken.map((url) => ({ url, interruptible: true })),
        ],
      });
    }
    case "enemyPick":
      return ask(lines.enemyPick);
    case "done":
      return tell(lines.prepareToFight);
  }
}

/** O som de um evento, já sorteado entre as variantes. */
export function vetoCue(
  event: VetoSoundEvent,
  locale: SoundLocale,
  random: () => number = Math.random,
): VetoCue | undefined {
  return pickWeighted(variants(event, locale), random);
}

/** Falas de um lote que tocam (a revelação às cegas traz várias de uma vez). */
const MAX_VOICES = 4;
/** O espaço entre elas, para cada uma começar audível. */
const STAGGER_S = 0.35;

/** Os sons de um lote de eventos (o que mudou de uma vez), pelas regras do topo. */
export function vetoBatch(
  events: readonly VetoSoundEvent[],
  locale: SoundLocale,
  random: () => number = Math.random,
): SoundBatch {
  // Cada efeito uma vez só: vários iguais juntos somariam o volume.
  const effects = new Set<string>();
  const voices: { urls: readonly string[]; delay: number }[] = [];
  const announcement: AnnouncementLine[] = [];
  for (const event of events) {
    const cue = vetoCue(event, locale, random);
    if (!cue) continue;
    if (cue.effect) effects.add(cue.effect);
    if (cue.voice?.length && voices.length < MAX_VOICES) {
      voices.push({ urls: cue.voice, delay: voices.length * STAGGER_S });
    }
    if (cue.announce) announcement.push(...cue.announce);
  }
  return {
    effects: [...effects],
    voices,
    announcement,
    announceAfter: Math.max(0, ...[...effects].map((e) => HOLD_S[e] ?? 0)),
    interrupt: events.length > 0,
  };
}

/**
 * O que vale baixar antes de tocar: as falas e os efeitos de qualquer veto e
 * os nomes dos mapas do pool e dos campeões do torneio (as despedidas vêm no
 * uso).
 */
export function vetoSoundsToPreload(
  locale: SoundLocale,
  pool: readonly string[],
  champions: readonly string[],
): string[] {
  const announcer = FILES.announcer[locale];
  return [
    ...Object.values(announcer.lines),
    ...Object.values(FILES.effects),
    ...pool.flatMap((slug) => lookup(announcer.maps, slug) ?? []),
    ...champions.flatMap((slug) => lookup(announcer.champions, slug) ?? []),
  ];
}

/** A contagem dos últimos segundos da vez, e o grupo dela no `sound-player`. */
export const TICK_SOUND = FILES.effects.tick;
export const COUNTDOWN = "veto-countdown";

/** A música do veto em andamento (`playLoop`). */
export const VETO_MUSIC: LoopSound = FILES.music;
