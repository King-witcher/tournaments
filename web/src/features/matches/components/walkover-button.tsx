"use client";

import { useTranslations } from "next-intl";
import { useRef, useState } from "react";

import { Button } from "@/shared/ui/button";
import type { MatchTarget } from "../lib/match-target";
import { WalkoverModal } from "./walkover-modal";

type Props = {
  target: MatchTarget;
  names: { a: string; b: string };
  decided: boolean;
};

/**
 * Para organizer e owner, no cabeçalho da série (pronta ou decidida): abre o
 * Modal do W.O. Terciário, porque a série se resolve pelas partidas; é a
 * exceção, mas precisa estar à vista de quem organiza.
 */
export function WalkoverButton({ target, names, decided }: Props) {
  const t = useTranslations("Match.organizer");
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button ref={trigger} kind="tertiary" onClick={() => setOpen(true)}>
        {t("walkover")}
      </Button>
      {open && (
        <WalkoverModal
          target={target}
          names={names}
          decided={decided}
          trigger={trigger}
          onClose={() => setOpen(false)}
        />
      )}
    </>
  );
}
