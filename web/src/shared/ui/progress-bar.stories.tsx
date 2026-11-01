import { Stack, Tile } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useEffect, useState } from "react";

import { ProgressBar } from "./progress-bar";

const meta = {
  title: "Estado e conteúdo/ProgressBar",
  component: ProgressBar,
  args: {
    label: "Enviando a captura de tela",
    value: 40,
    helperText: "40 de 100 KB",
  },
  argTypes: {
    size: { control: "inline-radio", options: ["small", "big"] },
    status: {
      control: "inline-radio",
      options: ["active", "finished", "error"],
    },
    type: {
      control: "inline-radio",
      options: ["default", "inline", "indented"],
    },
    value: { control: { type: "range", min: 0, max: 100 } },
  },
} satisfies Meta<typeof ProgressBar>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Com `value`: quanto já foi. */
export const Determinate: Story = {};

/** Sem `value`: só que está em andamento. */
export const Indeterminate: Story = {
  args: {
    label: "Gerando a chave",
    value: undefined,
    helperText: "Distribuindo os 16 inscritos",
  },
};

/** O leitor de tela ouve "Feito". */
export const Finished: Story = {
  args: { status: "finished", helperText: "Captura enviada" },
};

export const Failed: Story = {
  args: { status: "error", helperText: "Não foi possível enviar a captura" },
};

export const Small: Story = { args: { size: "small" } };

/** O rótulo ao lado da barra. */
export const Inline: Story = {
  args: { type: "inline", helperText: undefined },
};

/** Alinhado ao conteúdo de um bloco: só dentro de tile ou painel. */
export const Indented: Story = {
  args: { type: "indented" },
  render: (args) => (
    <Tile>
      <ProgressBar {...args} />
    </Tile>
  ),
};

/** O rótulo só para o leitor de tela. */
export const HiddenLabel: Story = { args: { hideLabel: true } };

function RunningDemo(args: Story["args"]) {
  const [value, setValue] = useState(0);
  useEffect(() => {
    const timer = setInterval(
      () => setValue((current) => Math.min(current + 10, 100)),
      500,
    );
    return () => clearInterval(timer);
  }, []);
  const done = value === 100;
  return (
    <ProgressBar
      label="Enviando a captura de tela"
      {...args}
      value={value}
      status={done ? "finished" : "active"}
      helperText={done ? "Captura enviada" : `${value} de 100 KB`}
    />
  );
}

/** Anda sozinha até terminar, como num envio de verdade. */
export const Running: Story = {
  render: (args) => <RunningDemo {...args} />,
};

export const Statuses: Story = {
  render: (args) => (
    <Stack gap={7}>
      <ProgressBar {...args} />
      <ProgressBar {...args} value={undefined} label="Gerando a chave" />
      <ProgressBar {...args} status="finished" helperText="Captura enviada" />
      <ProgressBar {...args} status="error" helperText="Falha no envio" />
    </Stack>
  ),
};

export const Types: Story = {
  render: (args) => (
    <Stack gap={7}>
      {(["default", "inline", "indented"] as const).map((type) =>
        (["big", "small"] as const).map((size) => (
          <ProgressBar
            key={`${type}-${size}`}
            {...args}
            type={type}
            size={size}
            label={`${type}, ${size}`}
          />
        )),
      )}
    </Stack>
  ),
};
