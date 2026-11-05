import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";

import { Stack } from "./carbon/stack";
import { ProgressSteps } from "./progress-steps";

const steps = [
  { key: "draft", label: "Rascunho" },
  { key: "registration", label: "Inscrições" },
  { key: "check_in", label: "Check-in" },
  { key: "in_progress", label: "Em andamento" },
  { key: "finished", label: "Encerrado" },
];

const meta = {
  title: "Estado e conteúdo/ProgressSteps",
  component: ProgressSteps,
  args: { steps, current: 2, label: "Ciclo do torneio" },
  argTypes: {
    current: {
      control: { type: "number", min: 0, max: steps.length },
    },
    steps: { control: false },
  },
} satisfies Meta<typeof ProgressSteps>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Nenhum passo feito. */
export const First: Story = { args: { current: 0 } };

export const Last: Story = { args: { current: steps.length - 1 } };

/** Com `current` além do último passo, todos aparecem como feitos. */
export const AllDone: Story = { args: { current: steps.length } };

/** A linha embaixo do nome: uma data, "agora". */
export const WithSecondaryLabels: Story = {
  args: {
    steps: [
      { key: "draft", label: "Rascunho", secondaryLabel: "05/11" },
      { key: "registration", label: "Inscrições", secondaryLabel: "até 11/11" },
      { key: "check_in", label: "Check-in", secondaryLabel: "12/11, 20h30" },
      {
        key: "in_progress",
        label: "Em andamento",
        secondaryLabel: "12/11, 21h",
      },
      { key: "finished", label: "Encerrado" },
    ],
    current: 2,
  },
};

/** Cada passo atual, para ver feito, atual e pendente em sequência. */
export const EachStep: Story = {
  render: (args) => (
    <Stack gap={7}>
      {steps.map((step, index) => (
        <ProgressSteps key={step.key} {...args} current={index} />
      ))}
    </Stack>
  ),
};

const seriesSteps = [
  { key: "ready", label: "Pronto" },
  { key: "maps", label: "Mapas" },
  { key: "champions", label: "Campeões" },
  { key: "game-1", label: "Partida 1", secondaryLabel: "15 × 9" },
  { key: "game-2", label: "Partida 2", secondaryLabel: "Agora" },
  { key: "game-3", label: "Partida 3" },
];

/**
 * Com `onSelect`, os passos feitos e o atual viram botões (nome na cor de
 * link) para escolher o que ver; `selected` marca o que está à vista, em
 * negrito. Os seguintes não respondem.
 */
export const Selectable: Story = {
  args: { steps: seriesSteps, current: 4, label: "Passos da série" },
  render: function Render(args) {
    const [selected, setSelected] = useState<number>(args.current);
    return (
      <Stack gap={5}>
        <ProgressSteps {...args} selected={selected} onSelect={setSelected} />
        <p>À vista: {seriesSteps[selected].label}.</p>
      </Stack>
    );
  },
};

/** Um passo feito escolhido: em negrito, e o atual na cor de link. */
export const SelectedStep: Story = {
  args: {
    steps: seriesSteps,
    current: 4,
    label: "Passos da série",
    selected: 1,
    onSelect: () => {},
  },
};

/** `invalid`: o passo com problema (uma partida contestada). */
export const Invalid: Story = {
  args: {
    steps: [
      ...seriesSteps.slice(0, 3),
      {
        key: "game-1",
        label: "Partida 1",
        secondaryLabel: "Contestada",
        invalid: true,
      },
      ...seriesSteps.slice(4),
    ],
    current: 4,
    label: "Passos da série",
    selected: 3,
    onSelect: () => {},
  },
};

const longSeries = [
  ...seriesSteps.slice(0, 5),
  { key: "game-3", label: "Partida 3" },
  { key: "game-4", label: "Partida 4" },
  { key: "game-5", label: "Partida 5" },
];

/**
 * Sem espaço para os passos lado a lado (128px cada), navegável e com mais
 * de 4 passos: a lista num `Dropdown`, com o estado de cada um.
 */
export const CompactWhenNarrow: Story = {
  args: {
    steps: longSeries,
    current: 4,
    label: "Passos da série",
    selected: 4,
    onSelect: () => {},
  },
  globals: { viewport: { value: "phone", isRotated: false } },
};

/** Até 4 passos, ou só leitura: no celular, a lista fica vertical. */
export const Phone: Story = {
  args: WithSecondaryLabels.args,
  globals: { viewport: { value: "phone", isRotated: false } },
};

/**
 * Navegável com até 4 passos (uma md3 sem veto) no celular: a lista
 * vertical, com os feitos e o atual clicáveis.
 */
export const SelectablePhone: Story = {
  args: {
    steps: [
      { key: "game-1", label: "Partida 1", secondaryLabel: "25 × 14" },
      { key: "game-2", label: "Partida 2", secondaryLabel: "Agora" },
      { key: "game-3", label: "Partida 3" },
    ],
    current: 1,
    label: "Passos da série",
    selected: 0,
    onSelect: () => {},
  },
  globals: { viewport: { value: "phone", isRotated: false } },
};
