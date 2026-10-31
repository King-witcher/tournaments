import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { StatusTag } from "./status-tag";
import { StructuredList } from "./structured-list";

const meta = {
  title: "Listas/StructuredList",
  component: StructuredList,
  args: {
    label: "Resumo do formato",
    rows: [
      { key: "format", cells: ["Formato", "Eliminação dupla"] },
      { key: "series", cells: ["Série", "Md3, final em md5"] },
      { key: "players", cells: ["Jogadores", "16"] },
      { key: "checkin", cells: ["Check-in", "30 min antes"] },
      { key: "start", cells: ["Início", "12/11, 21h"] },
    ],
  },
  argTypes: {
    rows: { control: false },
    headers: { control: false },
    condensed: { control: "boolean" },
  },
} satisfies Meta<typeof StructuredList>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Sem cabeçalho: pares rótulo e valor, com o rótulo sem quebra de linha. */
export const LabelAndValue: Story = {};

/** Com cabeçalho: as partidas de uma série. */
export const WithHeaders: Story = {
  args: {
    label: "Partidas da série",
    headers: ["Partida", "Mapa", "Placar", "Vencedor"],
    rows: [
      { key: "1", cells: ["1", "Blood Run", "15 x 11", "Lobo"] },
      { key: "2", cells: ["2", "Awoken", "9 x 15", "João Silva"] },
      { key: "3", cells: ["3", "Crucible", "15 x 13", "Lobo"] },
    ],
  },
};

/** Células com componentes, não só texto. */
export const WithTags: Story = {
  args: {
    label: "Séries da rodada",
    headers: ["Série", "Jogadores", "Situação"],
    rows: [
      {
        key: "a",
        cells: [
          "C1",
          "Lobo x Mariana",
          <StatusTag key="s" status="in_progress">
            Em andamento
          </StatusTag>,
        ],
      },
      {
        key: "b",
        cells: [
          "C2",
          "Rafael x Camila",
          <StatusTag key="s" status="waiting">
            Aguardando
          </StatusTag>,
        ],
      },
      {
        key: "c",
        cells: [
          "C3",
          "Thiago x Beatriz",
          <StatusTag key="s" status="contested">
            Contestado
          </StatusTag>,
        ],
      },
    ],
  },
};

/** Linhas mais baixas, para resumo com muitos itens. */
export const Condensed: Story = { args: { condensed: true } };

export const CondensedWithHeaders: Story = {
  args: { ...WithHeaders.args, condensed: true },
};

export const Phone: Story = {
  ...WithHeaders,
  globals: { viewport: { value: "phone", isRotated: false } },
};
