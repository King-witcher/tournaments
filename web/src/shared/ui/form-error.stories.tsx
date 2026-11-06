import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Button } from "./button";
import { FormError } from "./form-error";

const meta = {
  title: "Formulário/FormError",
  component: FormError,
  args: { children: "Não foi possível criar o torneio. Tente de novo." },
  argTypes: { children: { control: "text" } },
} satisfies Meta<typeof FormError>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Resumo curto em negrito antes do texto. */
export const WithTitle: Story = {
  args: {
    title: "Torneio não criado",
    children: "Você já tem um torneio com este endereço.",
  },
};

/** Com o que fazer a respeito embaixo (o erro que se corrige em outra seção). */
export const WithAction: Story = {
  args: {
    children:
      "O pool precisa de pelo menos 5 mapas, um para cada partida da série mais longa.",
    action: (
      <Button kind="ghost" size="sm">
        Ir para Mapas
      </Button>
    ),
  },
};
