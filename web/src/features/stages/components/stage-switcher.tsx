"use client";

import { useOptimistic, useTransition } from "react";

import { usePathname, useRouter } from "@/i18n/navigation";
import { ContentSwitcher } from "@/shared/ui/content-switcher";
import styles from "./stage-switcher.module.scss";

type Props = {
  /** O que o seletor escolhe, para leitor de tela. */
  label: string;
  /** `value` é o que vai em `?stage=` (o número da fase ou `final`). */
  options: { value: string; text: string }[];
  /** A opção que a página mostra agora. */
  value: string;
};

/**
 * A fase que as abas Chave e Classificação mostram, no `ContentSwitcher`. A
 * escolha vai no endereço (`?stage=2`): sobrevive ao `router.refresh()` do
 * tempo real e dá link. A opção marca na hora, antes de a página nova chegar.
 */
export function StageSwitcher({ label, options, value }: Props) {
  const router = useRouter();
  const pathname = usePathname();
  const [shown, setShown] = useOptimistic(value);
  const [, startTransition] = useTransition();

  return (
    <div className={styles.switcher} data-options={options.length}>
      <ContentSwitcher
        aria-label={label}
        options={options}
        value={shown}
        onChange={(next) =>
          startTransition(() => {
            setShown(next);
            router.replace(
              { pathname, query: { stage: next } },
              { scroll: false },
            );
          })
        }
      />
    </div>
  );
}
