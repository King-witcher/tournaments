import {
  ButtonSkeleton,
  SkeletonText,
  Stack,
  TextInputSkeleton,
  Tile,
} from "@carbon/react";
import { useTranslations } from "next-intl";

import styles from "./form-skeleton.module.scss";

type Props = {
  /** Quantos campos (rótulo e caixa). */
  fields?: number;
  /** Com o título da seção em cima. */
  heading?: boolean;
  /** Com uma linha de texto (a descrição) antes dos campos. */
  text?: boolean;
  /** Dentro de um `Tile` (entrar, cadastro). */
  tile?: boolean;
  /**
   * O que está carregando, lido pelo leitor de tela; padrão "Carregando…".
   * `false` quando outra peça da mesma página já diz (um aviso por página).
   */
  label?: string | false;
};

/**
 * Um formulário carregando, na forma dele: os campos e o botão do Carbon em
 * esqueleto, para a página não pular quando ele chega.
 */
export function FormSkeleton({
  fields = 2,
  heading = false,
  text = false,
  tile = false,
  label,
}: Props) {
  const t = useTranslations("Common");
  const body = (
    <Stack gap={6}>
      {heading && <SkeletonText heading width="40%" />}
      {text && <SkeletonText width="80%" />}
      {Array.from({ length: fields }, (_, i) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: campos fixos, sem identidade.
        <TextInputSkeleton key={i} />
      ))}
      <ButtonSkeleton />
    </Stack>
  );
  return (
    <div>
      {label !== false && (
        <span role="status" className={styles.label}>
          {label ?? t("loading")}
        </span>
      )}
      <div aria-hidden>{tile ? <Tile>{body}</Tile> : body}</div>
    </div>
  );
}
