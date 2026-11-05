"use client";

import { useLocale } from "next-intl";
import { useEffect, useRef } from "react";

import { useStoredSwitch } from "@/shared/hooks/use-stored-switch";
import { unlockAudioOnGesture } from "@/shared/lib/audio";
import { serverTime } from "@/shared/lib/server-clock";
import {
  playBatch,
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
import { vetoSound } from "../lib/veto-sound-preference";
import {
  COUNTDOWN,
  soundLocale,
  TICK_SOUND,
  vetoBatch,
  vetoSoundsToPreload,
} from "../lib/veto-sounds";
import { useVetoBoard } from "./use-veto-board";

/** A contagem toca nos últimos segundos da vez de quem age, um por segundo. */
const TICKS = 5;

/**
 * Os sons do veto que a tela mostra: compara cada estado com o anterior e
 * toca o que mudou, pelas regras de `lib/veto-sounds.ts` (o que vem junto, o
 * que espera e o que corta). O primeiro estado (abrir a página) não toca nada, nem o
 * veto refeito (o histórico encolhe). Enquanto a ação de quem vê está no ar,
 * a vez não conta: o passo marcado na hora a tira, e a falha a devolve.
 * Silenciado (`vetoSound`), para o que estiver tocando.
 */
export function useVetoSounds() {
  const { veto, now, mine, pending } = useVetoBoard();
  const locale = soundLocale(useLocale());
  const [enabled] = useStoredSwitch(vetoSound);
  const last = useRef<VetoSoundState | null>(null);

  useEffect(() => unlockAudioOnGesture(), []);

  // Os slugs juntos num texto: o veto da sala é um objeto novo a cada renderização.
  const pool = veto?.pool.map((m) => m.slug).join(" ") ?? "";
  const champions = veto?.champions.map((c) => c.slug).join(" ") ?? "";
  useEffect(() => {
    if (!enabled) stopSounds();
    else {
      const slugs = (list: string) => (list ? list.split(" ") : []);
      preloadSounds(vetoSoundsToPreload(locale, slugs(pool), slugs(champions)));
    }
  }, [enabled, locale, pool, champions]);

  // A cada renderização: a sala monta um veto novo a cada uma, e a comparação
  // é por valores simples.
  useEffect(() => {
    const before = last.current;
    if (!veto) {
      last.current = null;
      return;
    }
    let after = soundState(veto, mine);
    if (pending != null && before) after = { ...after, pick: before.pick };
    last.current = after;
    if (!enabled || !before || after.history < before.history) return;
    const events = soundEvents(before, veto, after, Date.parse(now));
    if (events.length > 0) playBatch(vetoBatch(events, locale));
  });

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
