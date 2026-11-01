import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";

import { ProgressStepsMenu } from "./progress-steps-menu";

const steps = [
  { key: "ready", label: "Pronto" },
  { key: "maps", label: "Mapas" },
  { key: "champions", label: "Campeões" },
  {
    key: "game-1",
    label: "Partida 1",
    secondaryLabel: "Contestada",
    invalid: true,
  },
  { key: "game-2", label: "Partida 2", secondaryLabel: "Agora" },
  { key: "game-3", label: "Partida 3" },
  { key: "game-4", label: "Partida 4" },
  { key: "game-5", label: "Partida 5" },
];

/**
 * A forma compacta do `ProgressSteps` navegável (ele a usa sozinho quando os
 * passos não cabem lado a lado): os passos num `Dropdown`, com o ícone de
 * estado de cada um e os seguintes desabilitados.
 */
const meta = {
  title: "Estado e conteúdo/ProgressStepsMenu",
  component: ProgressStepsMenu,
  args: {
    steps,
    current: 4,
    selected: 4,
    label: "Passos da série",
    onSelect: () => {},
  },
  argTypes: { steps: { control: false } },
} satisfies Meta<typeof ProgressStepsMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: function Render(args) {
    const [selected, setSelected] = useState(args.current);
    return (
      <ProgressStepsMenu {...args} selected={selected} onSelect={setSelected} />
    );
  },
};

export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
