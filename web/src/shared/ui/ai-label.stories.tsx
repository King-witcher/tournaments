import { AILabelActions, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { AILabel, AILabelContent } from "./ai-label";
import { Button } from "./button";

const sizes = ["mini", "2xs", "xs", "sm", "md", "lg", "xl"] as const;

const explanation = (
  <AILabelContent>
    <Stack gap={3}>
      <h2>Ordem da chave sugerida</h2>
      <p>
        A IA ordenou os inscritos pelo rating e pelos últimos resultados, para
        os mais fortes só se cruzarem no fim.
      </p>
    </Stack>
  </AILabelContent>
);

// Aberta, a explicação sai por cima da página: um quadro com altura própria.
const frame = {
  layout: "centered",
  docs: { story: { inline: false, height: "320px" } },
};

const meta = {
  title: "Estado e conteúdo/AILabel",
  component: AILabel,
  args: { children: explanation },
  argTypes: {
    kind: { control: "inline-radio", options: ["default", "inline"] },
    size: { control: "select", options: sizes },
    textLabel: { control: "text" },
    children: { control: false },
    onRevertClick: { control: false },
  },
} satisfies Meta<typeof AILabel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Clique na marca: abre a explicação. */
export const Default: Story = {};

/** Aberta, como fica depois do clique. */
export const Open: Story = {
  args: { defaultOpen: true },
  parameters: frame,
};

/** No fim da explicação, as ações (`AILabelActions` do Carbon). */
export const WithActions: Story = {
  args: {
    defaultOpen: true,
    children: (
      <AILabelContent>
        <Stack gap={3}>
          <h2>Ordem da chave sugerida</h2>
          <p>A IA ordenou os inscritos pelo rating.</p>
        </Stack>
        <AILabelActions>
          <Button kind="ghost">Ver o rating</Button>
          <Button>Usar a ordem</Button>
        </AILabelActions>
      </AILabelContent>
    ),
  },
  parameters: frame,
};

export const Inline: Story = { args: { kind: "inline" } };

/** `textLabel`: um texto ao lado da marca, no formato `inline`. */
export const InlineWithText: Story = {
  args: { kind: "inline", textLabel: "Sugerida por IA" },
};

/** Depois que a pessoa editou: o botão volta ao conteúdo da IA. */
export const Revert: Story = {
  args: { revertActive: true, onRevertClick: fn() },
};

export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={5}>
      {sizes.map((size) => (
        <AILabel key={size} {...args} size={size} />
      ))}
    </Stack>
  ),
};

export const InlineSizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={5}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <AILabel
          key={size}
          {...args}
          kind="inline"
          size={size}
          textLabel="Sugerida por IA"
        />
      ))}
    </Stack>
  ),
};
