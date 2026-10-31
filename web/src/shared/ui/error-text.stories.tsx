import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Button } from "./button";
import { ErrorText } from "./error-text";

const meta = {
  title: "Formulário/ErrorText",
  component: ErrorText,
  args: { children: "Não foi possível remover o jogador." },
  argTypes: { children: { control: "text" } },
} satisfies Meta<typeof ErrorText>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Texto longo quebra de linha, mesmo dentro de uma célula de tabela. */
export const LongText: Story = {
  args: {
    children:
      "Não foi possível remover o jogador porque ele já jogou uma série deste torneio. Substitua o jogador em vez de removê-lo.",
  },
};

/** Junto do controle que falhou, como o botão de uma linha. */
export const BesideButton: Story = {
  render: (args) => (
    <Stack gap={3}>
      <Button kind="danger--tertiary">Remover</Button>
      <ErrorText {...args} />
    </Stack>
  ),
};
