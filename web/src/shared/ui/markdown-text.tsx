import Markdown, { type Components } from "react-markdown";
import remarkBreaks from "remark-breaks";

import { cn } from "@/shared/lib/cn";
import {
  rehypeNestedLists,
  remarkUserSyntax,
  safeUrl,
} from "@/shared/lib/markdown";
import { Link } from "./carbon/link";
import { ListItem, OrderedList, UnorderedList } from "./carbon/list";
import styles from "./markdown-text.module.scss";

type Props = {
  /** O texto em Markdown, como a pessoa escreveu. */
  children: string;
  /**
   * O começo do endereço das imagens (`![descrição](endereço)`) que
   * aparecem: as do nosso CDN (`rules_image_prefix` de `GET /images/settings`).
   * Sem ele, nenhuma: a imagem sai como a descrição, em texto. Uma string, e
   * não uma função, para passar do servidor ao cliente.
   */
  imagePrefix?: string | null;
  /** Links abrem em outra aba: a pré-via de um campo, onde sair da página perde o que foi escrito. */
  newTab?: boolean;
  /** Idioma do texto, quando difere do da página (a tradução no editor). */
  lang?: string;
  className?: string;
};

const remarkPlugins = [remarkUserSyntax, remarkBreaks];
const rehypePlugins = [rehypeNestedLists];

/** Marcada por `rehypeNestedLists`: lista dentro de item de lista. */
const isNested = (props: object) => "data-nested" in props;

function components(
  imagePrefix: Props["imagePrefix"],
  newTab: boolean,
): Components {
  return {
    // A seção em volta já tem o h2: `#` e `##` são os trechos (h3, como os do
    // "Como funciona"), o resto fica um nível abaixo.
    h1: ({ node: _, ...props }) => <h3 {...props} className={styles.h3} />,
    h2: ({ node: _, ...props }) => <h3 {...props} className={styles.h3} />,
    h3: ({ node: _, ...props }) => <h4 {...props} className={styles.h4} />,
    h4: ({ node: _, ...props }) => <h4 {...props} className={styles.h4} />,
    h5: ({ node: _, ...props }) => <h4 {...props} className={styles.h4} />,
    h6: ({ node: _, ...props }) => <h4 {...props} className={styles.h4} />,
    ul: ({ node: _, children, ...props }) => (
      <UnorderedList
        isExpressive
        nested={isNested(props)}
        className={styles.list}
      >
        {children}
      </UnorderedList>
    ),
    ol: ({ node: _, start, children, ...props }) => (
      // O contador do Carbon começa sempre do 1; a lista que começa em outro
      // número usa o marcador do navegador.
      <OrderedList
        isExpressive
        native={start !== undefined}
        start={start}
        nested={isNested(props)}
        className={styles.list}
      >
        {children}
      </OrderedList>
    ),
    li: ({ node: _, children }) => <ListItem>{children}</ListItem>,
    a: ({ node: _, href, children }) =>
      href ? (
        <Link
          inline
          href={href}
          rel="noopener noreferrer nofollow ugc"
          target={newTab ? "_blank" : undefined}
        >
          {children}
        </Link>
      ) : (
        // Endereço recusado (`javascript:`, relativo): só o texto.
        <span>{children}</span>
      ),
    img: ({ node: _, src, alt }) =>
      typeof src === "string" && imagePrefix && src.startsWith(imagePrefix) ? (
        // biome-ignore lint/performance/noImgElement: imagem do texto de usuário, já reduzida no envio e sem tamanho conhecido; o `next/image` pede largura e altura.
        <img
          src={src}
          alt={alt ?? ""}
          loading="lazy"
          className={styles.image}
        />
      ) : alt ? (
        <span>{alt}</span>
      ) : null,
    table: ({ node: _, ...props }) => (
      // A tabela larga rola dentro da caixa, não a página no celular.
      <div className={styles.table}>
        <table {...props} />
      </div>
    ),
  };
}

/**
 * Texto de usuário em Markdown (as regras da organização), no tipo e na
 * coluna de leitura do "Como funciona": títulos, parágrafos, listas,
 * negrito, itálico, links, citação, linha, tabela e código. HTML escrito no
 * texto sai como texto; link só para http, https e mailto (o resto sai como
 * texto), com `rel="noopener noreferrer nofollow ugc"`; imagem só com o
 * `imagePrefix` (a do nosso CDN), na largura da coluna e com o `alt`.
 * Quebra de linha simples vira quebra, como no texto sem formatação de
 * antes, e recuo não vira código: o que foi escrito sem Markdown continua
 * legível.
 *
 * `react-markdown`: renderiza no servidor sem mandar nada ao navegador e não
 * interpreta HTML; do GFM, só link solto e tabela (`remarkUserSyntax`).
 *
 * ```tsx
 * <MarkdownText>{tournament.rules}</MarkdownText>
 * ```
 */
export function MarkdownText({
  children,
  imagePrefix,
  newTab = false,
  lang,
  className,
}: Props) {
  return (
    <div lang={lang} className={cn(styles.text, className)}>
      <Markdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        urlTransform={safeUrl}
        components={components(imagePrefix, newTab)}
      >
        {children}
      </Markdown>
    </div>
  );
}
