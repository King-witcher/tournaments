"use client";

import { useLocale } from "next-intl";
import type { ComponentProps } from "react";

import { getPathname, type Href, useRouter } from "@/i18n/navigation";
import {
  TreeNode as CarbonTreeNode,
  TreeView as CarbonTreeView,
} from "./carbon/tree-view";

/**
 * Lista em árvore (`TreeView` do Carbon): itens aninhados que se abrem e
 * fecham, para navegar numa hierarquia (comunidade, torneios, fases). Com
 * um nível só, `ContainedList` ou `StructuredList`; blocos de conteúdo que
 * abrem e fecham, `Accordion`. `label` é obrigatório (nome da árvore para
 * leitor de tela); `hideLabel` o tira da tela. `size` `xs` ou `sm` (padrão).
 * Os nós são o `TreeNode` deste arquivo, com `renderIcon` para o ícone.
 */
export function TreeView(props: ComponentProps<typeof CarbonTreeView>) {
  return <CarbonTreeView {...props} />;
}

type TreeNodeProps = Omit<ComponentProps<typeof CarbonTreeNode>, "href"> & {
  /** O nó é um link: abre a página pelo roteador do i18n (mantém o idioma). */
  href?: Href;
};

/**
 * Um nó do `TreeView` (`TreeNode` do Carbon); os filhos são outros nós. Com
 * `href`, vira link para página interna: o endereço tem o prefixo de idioma
 * (abre em outra aba com Ctrl ou o botão do meio) e o clique comum navega
 * sem recarregar.
 */
export function TreeNode({ href, onClick, ...rest }: TreeNodeProps) {
  const router = useRouter();
  const locale = useLocale();

  if (href === undefined) return <CarbonTreeNode onClick={onClick} {...rest} />;

  return (
    <CarbonTreeNode
      {...rest}
      href={getPathname({ href, locale })}
      onClick={(event) => {
        onClick?.(event);
        // O Enter num link vira clique no <a>, e o Carbon chama este mesmo
        // onClick de novo com o evento de teclado: só o clique navega.
        const modified =
          event.metaKey || event.ctrlKey || event.shiftKey || event.altKey;
        if (event.type !== "click" || event.defaultPrevented || modified)
          return;
        event.preventDefault();
        router.push(href);
      }}
    />
  );
}
