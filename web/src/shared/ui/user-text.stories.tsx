import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { UserText } from "./user-text";

const meta = {
  title: "Estado e conteúdo/UserText",
  component: UserText,
  args: {
    children:
      "Duelo em md3, final em md5.\nTrazer o próprio mouse.\n\nQuem não fizer o check-in até 20h30 perde a vaga.",
  },
  argTypes: { children: { control: "text" } },
} satisfies Meta<typeof UserText>;

export default meta;
type Story = StoryObj<typeof meta>;

/** As quebras de linha e as linhas em branco ficam como o autor escreveu. */
export const Default: Story = {};

/** HTML e Markdown saem como texto, sem interpretar. */
export const NoFormatting: Story = {
  args: {
    children:
      "<b>Regras</b> e **mais regras**\n# Isto não é título\n<script>alert(1)</script>",
  },
};

/** Palavra sem espaço não estoura a largura. */
export const LongWord: Story = {
  args: {
    children:
      "https://exemplo.com.br/torneios/copa-arena-sul/regulamento-completo-da-temporada-de-inverno-2026",
  },
};

export const LongText: Story = {
  args: {
    children: Array.from(
      { length: 4 },
      (_, i) =>
        `Regra ${i + 1}. A série é decidida em md3, e os mapas saem do veto. Quem desconectar tem 5 minutos para voltar antes do W.O.`,
    ).join("\n\n"),
  },
};

export const Phone: Story = {
  ...Default,
  globals: { viewport: { value: "phone", isRotated: false } },
};

/** `clamp`: duas linhas com reticências, para o resumo de um texto longo. */
export const Clamped: Story = {
  ...LongText,
  args: { ...LongText.args, clamp: true },
};
