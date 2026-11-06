"use client";

import { Music, MusicRemove, VolumeMute, VolumeUp } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { useStoredSwitch } from "@/shared/hooks/use-stored-switch";
import { IconButton } from "@/shared/ui/icon-button";
import { vetoMusic, vetoSound } from "../lib/veto-sound-preference";

/**
 * Liga e desliga os sons do veto neste navegador, no cabeçalho da série e da
 * sala, e, com eles ligados, a música à parte (quem quer ouvir a vez sem a
 * música). O ícone mostra o estado; o nome, o que o clique faz.
 */
export function VetoSoundToggle() {
  const t = useTranslations("Match.veto.sound");
  const [enabled, setEnabled] = useStoredSwitch(vetoSound);
  const [music, setMusic] = useStoredSwitch(vetoMusic);
  return (
    <>
      {enabled && (
        <IconButton
          label={music ? t("muteMusic") : t("unmuteMusic")}
          onClick={() => setMusic(!music)}
        >
          {music ? <Music /> : <MusicRemove />}
        </IconButton>
      )}
      <IconButton
        label={enabled ? t("mute") : t("unmute")}
        onClick={() => setEnabled(!enabled)}
      >
        {enabled ? <VolumeUp /> : <VolumeMute />}
      </IconButton>
    </>
  );
}
