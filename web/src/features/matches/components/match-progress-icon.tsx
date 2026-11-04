import { CheckmarkOutline, CircleDash, Incomplete } from "@carbon/icons-react";
import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";
import type { MatchProgress } from "../lib/status";
import styles from "./match-progress-icon.module.scss";

const ICONS = {
  completed: CheckmarkOutline,
  in_progress: Incomplete,
  pending: CircleDash,
};

/**
 * O estado da série pela forma, com os símbolos do `ProgressIndicator` do
 * Carbon (os mesmos dos passos da página da série), para varrer uma lista de
 * cartões sem ler: a cor sozinha não bastaria. Tem nome para o leitor de tela.
 */
export function MatchProgressIcon({ progress }: { progress: MatchProgress }) {
  const t = useTranslations("Bracket.progress");
  const Icon = ICONS[progress];
  return (
    <Icon
      aria-label={t(progress)}
      className={cn(styles.icon, progress === "in_progress" && styles.active)}
    />
  );
}
