import { Download } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Button } from "./button";
import { Stack } from "./carbon/stack";
import { Tooltip } from "./tooltip";

const alignments = [
  "top",
  "top-start",
  "top-end",
  "bottom",
  "bottom-start",
  "bottom-end",
  "left",
  "left-start",
  "left-end",
  "right",
  "right-start",
  "right-end",
] as const;

// Aberta, a dica sai por cima da página: um quadro com altura própria.
const frame = {
  layout: "centered",
  docs: { story: { inline: false, height: "200px" } },
};

const meta = {
  title: "Estado e conteúdo/Tooltip",
  component: Tooltip,
  parameters: frame,
  args: {
    description: "Embaralha os inscritos pela ordem da chave.",
    defaultOpen: true,
    children: <Button kind="tertiary">Gerar a chave</Button>,
  },
  argTypes: {
    align: { control: "select", options: alignments },
    label: { control: "text" },
    description: { control: "text" },
    children: { control: false },
  },
} satisfies Meta<typeof Tooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

/** `description`: complementa o texto do botão, lido junto dele. */
export const Description: Story = {};

/** `label`: o nome completo de um botão de texto curto (o leitor lê só ele). */
export const Label: Story = {
  args: {
    description: undefined,
    label: "Baixar a classificação em CSV",
    children: (
      <Button kind="ghost" renderIcon={Download}>
        CSV
      </Button>
    ),
  },
};

/** Fechada: abre ao passar o mouse ou focar. */
export const Closed: Story = { args: { defaultOpen: false } };

export const Bottom: Story = { args: { align: "bottom" } };

export const Left: Story = { args: { align: "left" } };

export const Right: Story = { args: { align: "right" } };

/** Sem alto contraste: o fundo do tema, com sombra. */
export const LowContrast: Story = {
  args: { highContrast: false, dropShadow: true },
};

/** Sem espaço em cima, a dica troca de lado sozinha. */
export const AutoAlign: Story = {
  args: { autoAlign: true },
  parameters: { layout: "padded" },
};

/** `enterDelayMs` e `leaveDelayMs`: abre na hora e demora 2 s para fechar. */
export const Duration: Story = {
  args: { defaultOpen: false, enterDelayMs: 0, leaveDelayMs: 2000 },
};

/** Fecha ao clicar no botão (`closeOnActivation`). */
export const CloseOnActivation: Story = {
  args: { closeOnActivation: true },
};

/** Todos os lados, para comparar. */
export const Alignments: Story = {
  parameters: { docs: { story: { inline: false, height: "520px" } } },
  render: (args) => (
    <Stack gap={11}>
      {[0, 3, 6, 9].map((start) => (
        <Stack key={start} orientation="horizontal" gap={13}>
          {alignments.slice(start, start + 3).map((align) => (
            <Tooltip
              key={align}
              {...args}
              align={align}
              description={align}
              defaultOpen
            >
              <Button kind="tertiary" size="sm">
                {align}
              </Button>
            </Tooltip>
          ))}
        </Stack>
      ))}
    </Stack>
  ),
};
