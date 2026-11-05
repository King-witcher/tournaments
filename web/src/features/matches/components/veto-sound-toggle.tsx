"use client";

import { VolumeMute, VolumeUp } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { useStoredSwitch } from "@/shared/hooks/use-stored-switch";
import { IconButton } from "@/shared/ui/icon-button";
import { vetoSound } from "../lib/veto-sound-preference";

/**
 * Liga e desliga os sons do veto neste navegador, no cabeçalho da série e da
 * sala. O ícone mostra o estado; o nome, o que o clique faz.
 */
export function VetoSoundToggle() {
  const t = useTranslations("Match.veto.sound");
  const [enabled, setEnabled] = useStoredSwitch(vetoSound);
  return (
    <IconButton
      label={enabled ? t("mute") : t("unmute")}
      onClick={() => setEnabled(!enabled)}
    >
      {enabled ? <VolumeUp /> : <VolumeMute />}
    </IconButton>
  );
}
