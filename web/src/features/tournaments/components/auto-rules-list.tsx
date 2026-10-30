import { ListItem, OrderedList, UnorderedList } from "@carbon/react";
import type { ReactNode } from "react";

import styles from "./auto-rules-list.module.scss";
import { Sentences } from "./sentences";

type Props = {
  /** Cada item são as frases dele; item sem frase nenhuma sai. */
  items: ReactNode[][];
  /** Numerada: passos na ordem em que acontecem. */
  ordered?: boolean;
};

/** Uma lista do "Como funciona", no tipo do texto em volta. */
export function AutoRulesList({ items, ordered }: Props) {
  const List = ordered ? OrderedList : UnorderedList;
  return (
    <List isExpressive className={styles.list}>
      {items
        .filter((sentences) => sentences.some(Boolean))
        .map((sentences, index) => (
          // biome-ignore lint/suspicious/noArrayIndexKey: itens fixos, na ordem.
          <ListItem key={index}>
            <Sentences>{sentences}</Sentences>
          </ListItem>
        ))}
    </List>
  );
}
