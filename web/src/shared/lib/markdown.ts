import { gfmAutolinkLiteralFromMarkdown } from "mdast-util-gfm-autolink-literal";
import { gfmTableFromMarkdown } from "mdast-util-gfm-table";
import { gfmAutolinkLiteral } from "micromark-extension-gfm-autolink-literal";
import { gfmTable } from "micromark-extension-gfm-table";

/**
 * O Markdown dos textos de usuário (as regras da organização), em cima do
 * `react-markdown` (`MarkdownText`): a sintaxe aceita, o filtro de endereço
 * e a marcação das listas aninhadas.
 */

type ParserData = {
  micromarkExtensions?: unknown[];
  fromMarkdownExtensions?: unknown[];
};

/**
 * A parte do GFM que serve aqui: endereço solto vira link (o texto antigo,
 * escrito sem Markdown, tem muitos) e tabela. Fora o resto do `remark-gfm`:
 * nota de rodapé (cria um título "Footnotes" em inglês e links internos),
 * lista de tarefas e riscado. Fora também o bloco de código por recuo de 4
 * espaços: no texto antigo, o recuo era só para alinhar, e virava código.
 */
export function remarkUserSyntax(this: { data(): object }) {
  // Os campos que o `remark-parse` lê (os tipos dele não vêm junto).
  const data = this.data() as ParserData;
  data.micromarkExtensions ??= [];
  data.fromMarkdownExtensions ??= [];
  data.micromarkExtensions.push(gfmAutolinkLiteral(), gfmTable(), {
    disable: { null: ["codeIndented"] },
  });
  data.fromMarkdownExtensions.push(
    gfmAutolinkLiteralFromMarkdown(),
    gfmTableFromMarkdown(),
  );
}

type HastNode = {
  type: string;
  tagName?: string;
  properties?: Record<string, unknown>;
  children?: HastNode[];
};

/** Lista dentro de item de lista ganha `dataNested`, para o marcador do Carbon de lista aninhada. */
export function rehypeNestedLists() {
  const mark = (node: HastNode, inItem: boolean) => {
    const isList = node.tagName === "ul" || node.tagName === "ol";
    if (isList && inItem)
      node.properties = { ...node.properties, dataNested: true };
    for (const child of node.children ?? []) {
      mark(child, inItem || node.tagName === "li");
    }
  };
  return (tree: HastNode) => mark(tree, false);
}

const allowedProtocols = new Set(["http:", "https:", "mailto:"]);

/**
 * O endereço de um link ou imagem, se for http, https ou mailto; senão,
 * vazio (o link sai como texto). Recusa `javascript:`, `data:`, relativo e
 * âncora. O `URL` normaliza antes (maiúsculas, tabulação no meio do
 * protocolo), do mesmo jeito que o navegador.
 */
export function safeUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return allowedProtocols.has(parsed.protocol) ? parsed.href : "";
  } catch {
    return "";
  }
}
