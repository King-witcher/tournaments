import { useTranslations } from "next-intl";

import { cn } from "@/shared/lib/cn";
import styles from "./empty-slot.module.scss";

type Props = {
  /** Na edição, o texto começa onde o avatar dos jogadores começa (depois da alça). */
  indented?: boolean;
};

/** Uma vaga livre num time: mostra o que falta e já dá ao card a altura final. */
export function EmptySlot({ indented }: Props) {
  const t = useTranslations("Team.teams");
  return (
    <li className={cn(styles.slot, indented && styles.indented)}>
      {t("emptySlot")}
    </li>
  );
}
