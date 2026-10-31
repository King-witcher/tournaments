import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { userEvent, within } from "storybook/test";

import { InfoTip } from "./info-tip";

const meta = {
  title: "Estado e conteúdo/InfoTip",
  component: InfoTip,
  args: {
    label: "O que é check-in?",
    term: "Check-in",
    children: "Confirmar presença antes do início. Quem não faz, perde a vaga.",
  },
  argTypes: {
    label: { control: "text" },
    term: { control: "text" },
    children: { control: "text" },
  },
} satisfies Meta<typeof InfoTip>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O (i) fica preso ao termo: os dois não quebram linha entre si. */
export const WithTerm: Story = {};

/** Sem `term`, só o (i), junto de um rótulo que já está na tela. */
export const OnlyIcon: Story = { args: { term: undefined } };

/** Aberta, como fica depois do clique ou do toque. */
export const Open: Story = {
  parameters: { docs: { story: { inline: false, height: "160px" } } },
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
  },
};

export const LongText: Story = {
  args: {
    label: "O que é eliminação dupla?",
    term: "Eliminação dupla",
    children:
      "Quem perde a primeira série cai para a chave de baixo. Só sai do torneio quem perde duas. A final pode ter uma série extra.",
  },
  parameters: Open.parameters,
  play: Open.play,
};

export const Phone: Story = {
  ...LongText,
  globals: { viewport: { value: "phone", isRotated: false } },
};
