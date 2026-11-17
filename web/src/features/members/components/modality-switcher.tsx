"use client";

import { useTranslations } from "next-intl";
import { useOptimistic } from "react";

import { ContentSwitcher } from "@/shared/ui/content-switcher";
import { useSearchTransition } from "../hooks/use-search-transition";
import { type Modality, opponentsHref } from "../lib/opponents-href";

type Value = "all" | Modality;

type Props = {
  /** A página da pessoa. */
  base: string;
  value: Value;
};

/**
 * A modalidade dos confrontos (todas, duelo, TDM), em cima da tabela: trocar
 * muda o endereço (`?modality=`) e marca na hora; a tabela fica em
 * esqueleto até chegar filtrada (`SearchTransition`).
 */
export function ModalitySwitcher({ base, value }: Props) {
  const t = useTranslations("Member.opponents");
  const { navigate } = useSearchTransition();
  const [shown, setShown] = useOptimistic(value);
  return (
    <ContentSwitcher<Value>
      aria-label={t("modality")}
      value={shown}
      options={[
        { value: "all", text: t("all") },
        { value: "duel", text: t("duel") },
        { value: "tdm", text: t("tdm") },
      ]}
      onChange={(next) =>
        navigate(opponentsHref(base, next === "all" ? undefined : next), () =>
          setShown(next),
        )
      }
    />
  );
}
