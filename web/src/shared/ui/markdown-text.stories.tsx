import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { MarkdownText } from "./markdown-text";

const rules = `# Conduta
Respeite os adversários no chat.
Quem ofender alguém é desclassificado.

## Queda de conexão
1. Avise no canal **#suporte** antes de sair.
2. Volte em até *5 minutos*.
   - A partida recomeça do placar em que parou.
   - Sem volta, é W.O.

### Sala
Quem tem a melhor ordem da chave cria a sala.

> Dúvida sobre uma regra vale o que a organização decidir.

---

Mais detalhes no [regulamento](https://telefrag.com.br) ou em suporte@telefrag.com.br.`;

const meta = {
  title: "Estado e conteúdo/MarkdownText",
  component: MarkdownText,
  args: { children: rules },
  argTypes: { children: { control: "text" } },
} satisfies Meta<typeof MarkdownText>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Títulos, listas (com uma aninhada), negrito, itálico, citação, linha e links. */
export const Default: Story = {};

/** Texto de antes do Markdown: as quebras de linha e o recuo continuam legíveis. */
export const PlainText: Story = {
  args: {
    children:
      "Duelo em md3, final em md5.\nTrazer o próprio mouse.\n\n    Recuo de alinhamento, não é código.\nQuem não fizer o check-in até 20h30 perde a vaga.",
  },
};

/** HTML sai como texto; link que não é http, https ou mailto sai sem link; imagem fora do `imagePrefix` sai como a descrição. */
export const Unsafe: Story = {
  args: {
    children:
      "<b>negrito em HTML</b> <script>alert(1)</script>\n[javascript](javascript:alert(1)) [data](data:text/html,oi) [relativo](/admin) [certo](https://telefrag.com.br)\n![Mapa da final](https://exemplo.com/mapa.png)",
  },
};

/** `imagePrefix`: só a imagem com esse começo aparece (aqui, as de `picsum.photos`), na largura da coluna. */
export const Images: Story = {
  args: {
    children:
      "![Aceita](https://picsum.photos/640/240)\n\n![Recusada](https://exemplo.com/mapa.png)",
    imagePrefix: "https://picsum.photos/",
  },
};

/** A tabela larga rola dentro da caixa. */
export const Table: Story = {
  args: {
    children:
      "| Fase | Série | Prazo |\n|---|:-:|---|\n| Grupos | md1 | sábado às 20h |\n| Quartas | md3 | domingo às 18h |\n| Final | md5 | domingo às 21h |",
  },
};

export const Phone: Story = {
  ...Default,
  globals: { viewport: { value: "phone", isRotated: false } },
};
