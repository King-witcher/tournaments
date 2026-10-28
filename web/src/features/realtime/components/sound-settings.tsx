"use client";

import { Stack } from "@carbon/react";
import { useTranslations } from "next-intl";

import { playChime } from "@/shared/lib/chime";
import { Button } from "@/shared/ui/button";
import { Toggle } from "@/shared/ui/toggle";
import { useSoundEnabled } from "../hooks/use-sound-enabled";

/**
 * O som dos avisos com o site aberto, na aba Avisos da conta: o interruptor
 * (vale para este navegador, na hora) e um botão para ouvir.
 */
export function SoundSettings() {
  const t = useTranslations("Notifications.sound");
  const [enabled, setEnabled] = useSoundEnabled();
  return (
    <Stack gap={5}>
      <Toggle labelText={t("label")} toggled={enabled} onToggle={setEnabled} />
      <div>
        {/* O clique já prepara o áudio (shared/lib/chime). */}
        <Button kind="tertiary" size="sm" onClick={playChime}>
          {t("test")}
        </Button>
      </div>
    </Stack>
  );
}
