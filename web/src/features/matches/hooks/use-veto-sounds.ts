"use client";

import { useLocale } from "next-intl";
import { useEffect, useRef } from "react";

import { useStoredSwitch } from "@/shared/hooks/use-stored-switch";
import { unlockAudioOnGesture } from "@/shared/lib/audio";
import { serverTime } from "@/shared/lib/server-clock";
import {
  playBatch,
  playLoop,
  preloadSounds,
  stopGroup,
  stopSounds,
} from "@/shared/lib/sound-player";
import { isActing } from "../lib/veto";
import {
  soundEvents,
  soundState,
  type VetoSoundState,
} from "../lib/veto-sound-events";
import { vetoMusic, vetoSound } from "../lib/veto-sound-preference";
import {
  COUNTDOWN,
  soundLocale,
  TICK_SOUND,
  VETO_MUSIC,
  vetoBatch,
  vetoSoundsToPreload,
} from "../lib/veto-sounds";
import { useVetoBoard } from "./use-veto-board";

/** A contagem toca nos últimos segundos da vez de quem age, um por segundo. */
const TICKS = 5;

/** A música entra e sai aos poucos, em tantos segundos. */
const MUSIC_FADE_S = 2;
/** Desligada pelo botão, sai em tanto (sem o estalo de cortar no meio da onda). */
const MUSIC_OFF_S = 0.2;
/** No começo do veto, a música espera a parte forte do "match found". */
const MUSIC_AFTER_BEGIN_S = 2.5;

/**
 * Os sons do veto que a tela mostra: compara cada estado com o anterior e
 * toca o que mudou, pelas regras de `lib/veto-sounds.ts` (o que vem junto, o
 * que espera e o que corta). O primeiro estado (abrir a página) não toca nada, nem o
 * veto refeito (o histórico encolhe). Enquanto a ação de quem vê está no ar,
 * a vez não conta: o passo marcado na hora a tira, e a falha a devolve.
 * Silenciado (`vetoSound`), para o que estiver tocando. A música é um estado,
 * não um evento: toca enquanto o veto está em andamento (também para quem
 * abre a página no meio dele), se ela e os sons estão ligados.
 */
export function useVetoSounds() {
  const { veto, now, mine, pending } = useVetoBoard();
  const locale = soundLocale(useLocale());
  const [enabled] = useStoredSwitch(vetoSound);
  const [music] = useStoredSwitch(vetoMusic);
  const last = useRef<VetoSoundState | null>(null);
  /** Quanto a música espera: o começo visto ao vivo espera o "match found". */
  const musicDelay = useRef(0);

  useEffect(() => unlockAudioOnGesture(), []);

  const status = veto?.status;
  const musicOn = enabled && music && status === "in_progress";
  // Os slugs juntos num texto: o veto da sala é um objeto novo a cada renderização.
  const pool = veto?.pool.map((m) => m.slug).join(" ") ?? "";
  const champions = veto?.champions.map((c) => c.slug).join(" ") ?? "";
  const musicAhead = enabled && music && status != null && status !== "done";
  useEffect(() => {
    if (!enabled) stopSounds();
    else {
      const slugs = (list: string) => (list ? list.split(" ") : []);
      preloadSounds(vetoSoundsToPreload(locale, slugs(pool), slugs(champions)));
    }
  }, [enabled, locale, pool, champions]);
  useEffect(() => {
    if (musicAhead) preloadSounds([VETO_MUSIC.url]);
  }, [musicAhead]);

  // A cada renderização: a sala monta um veto novo a cada uma, e a comparação
  // é por valores simples.
  useEffect(() => {
    const before = last.current;
    if (!veto) {
      last.current = null;
      return;
    }
    let after = soundState(veto, mine);
    if (pending != null && before) after = { ...after, turn: before.turn };
    last.current = after;
    const begun =
      before?.status === "waiting_ready" && after.status === "in_progress";
    musicDelay.current = begun ? MUSIC_AFTER_BEGIN_S : 0;
    if (!enabled || !before || after.history < before.history) return;
    const events = soundEvents(before, veto, after, Date.parse(now));
    if (events.length > 0) playBatch(vetoBatch(events, locale));
  });

  // Quem desliga a música pelo botão a quer fora logo; o fim do veto e a
  // saída da página a tiram aos poucos (desligar os sons para tudo na hora).
  const musicFadeOut = useRef(MUSIC_FADE_S);
  musicFadeOut.current = music ? MUSIC_FADE_S : MUSIC_OFF_S;
  // Depois da comparação (a ordem dos efeitos), para ver o começo dela.
  useEffect(() => {
    if (!musicOn) return;
    const stop = playLoop(VETO_MUSIC, {
      delay: musicDelay.current,
      fadeIn: MUSIC_FADE_S,
    });
    return () => stop(musicFadeOut.current);
  }, [musicOn]);

  const acting = veto != null && pending == null && isActing(veto, mine);
  const deadline = veto?.deadline ?? null;
  useEffect(() => {
    if (!enabled || !acting || !deadline) return;
    const end = Date.parse(deadline);
    const left = end - serverTime();
    const timers: number[] = [];
    const tick = () => playBatch({ effects: [TICK_SOUND], group: COUNTDOWN });
    for (let second = TICKS; second >= 1; second--) {
      const wait = left - second * 1000;
      if (wait >= 0) timers.push(window.setTimeout(tick, wait));
    }
    // Agiu, a vez mudou ou silenciou: a contagem para, mesmo no meio do som.
    return () => {
      for (const timer of timers) clearTimeout(timer);
      stopGroup(COUNTDOWN);
    };
  }, [enabled, acting, deadline]);
}
