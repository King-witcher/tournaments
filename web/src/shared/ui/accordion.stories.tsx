import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Accordion } from "./accordion";
import { Stack } from "./carbon/stack";
import { StructuredList } from "./structured-list";

const meta = {
  title: "Listas/Accordion",
  component: Accordion,
  args: {
    title: "Torneios encerrados",
    children: "Copa do Sul, Liga de verão e Duelo de sexta.",
  },
  argTypes: {
    title: { control: "text" },
    children: { control: "text" },
    open: { control: "boolean" },
  },
} satisfies Meta<typeof Accordion>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Fechado por padrão: o bloco é secundário. */
export const Closed: Story = {};

export const Open: Story = { args: { open: true } };

export const WithStructuredList: Story = {
  args: {
    title: "Organização da partida",
    open: true,
    children: (
      <StructuredList
        label="Organização da partida"
        rows={[
          { key: "host", cells: ["Quem cria a sala", "Lobo"] },
          { key: "server", cells: ["Servidor", "Porto Alegre"] },
          { key: "deadline", cells: ["Prazo", "30 min depois do check-in"] },
        ]}
      />
    ),
  },
};

/** Um acordeão por bloco: cada um abre e fecha sozinho. */
export const Stacked: Story = {
  render: () => (
    <Stack gap={5}>
      <Accordion title="Torneios encerrados" open>
        Copa do Sul, Liga de verão e Duelo de sexta.
      </Accordion>
      <Accordion title="Histórico">Nenhuma alteração.</Accordion>
    </Stack>
  ),
};

export const Phone: Story = {
  args: {
    title: "Campeonato Gaúcho de Quake Champions, temporada de inverno",
    open: true,
  },
  globals: { viewport: { value: "phone", isRotated: false } },
};
