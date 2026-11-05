import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { CodeSnippetSkeleton } from "./carbon/code-snippet";
import { Stack } from "./carbon/stack";
import { CodeSnippet } from "./code-snippet";

const link = "https://telefrag.gg/pt/invite/7Hq2xKp9Lm4vRt8sWz3nYb6cDf1gJe5a";

const rules = Array.from(
  { length: 20 },
  (_, index) => `${index + 1}. Partida ${index + 1}: mapa escolhido no veto.`,
).join("\n");

const meta = {
  title: "Estado e conteúdo/CodeSnippet",
  component: CodeSnippet,
  args: { type: "single", "aria-label": "Link de convite", children: link },
  argTypes: {
    type: { control: "inline-radio", options: ["inline", "single", "multi"] },
    "aria-label": { control: "text" },
    children: { control: "text" },
  },
} satisfies Meta<typeof CodeSnippet>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Uma linha que rola na horizontal, com o botão de copiar. */
export const Single: Story = {};

/**
 * No meio de uma frase: o trecho inteiro é o botão. O Carbon o embrulha num
 * `div` (a dica de "Copiado"), então a frase não pode ser um `<p>`.
 */
export const Inline: Story = {
  args: { type: "inline", "aria-label": undefined, children: "arena-sul" },
  render: (args) => (
    <div>
      O endereço da comunidade é <CodeSnippet {...args} />.
    </div>
  ),
};

/** Mais de 15 linhas: aparece "Mostrar mais". */
export const Multi: Story = {
  args: { type: "multi", "aria-label": "Regras", children: rules },
};

export const MultiShort: Story = {
  args: {
    type: "multi",
    "aria-label": "Regras",
    children: rules.split("\n").slice(0, 4).join("\n"),
  },
};

/** `wrapText`: quebra as linhas longas em vez de rolar. */
export const WrapText: Story = {
  args: {
    type: "multi",
    wrapText: true,
    "aria-label": "Link de convite",
    children: `${link}?origem=discord&torneio=copa-de-inverno-2026&lado=superior`,
  },
};

export const HideCopyButton: Story = { args: { hideCopyButton: true } };

export const Disabled: Story = { args: { disabled: true } };

/** `CodeSnippetSkeleton` do Carbon, nos formatos de uma e de várias linhas. */
export const Skeleton: Story = {
  render: () => (
    <Stack gap={5}>
      <CodeSnippetSkeleton type="single" className={undefined} />
      <CodeSnippetSkeleton type="multi" className={undefined} />
    </Stack>
  ),
};

export const Types: Story = {
  render: (args) => (
    <Stack gap={5}>
      <div>
        O endereço da comunidade é{" "}
        <CodeSnippet type="inline">arena-sul</CodeSnippet>.
      </div>
      <CodeSnippet {...args} type="single" />
      <CodeSnippet {...args} type="multi">
        {rules}
      </CodeSnippet>
    </Stack>
  ),
};
