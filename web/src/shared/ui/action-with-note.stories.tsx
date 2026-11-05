import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { ActionWithNote } from "./action-with-note";
import { Button } from "./button";
import { Stack } from "./carbon/stack";

const meta = {
  title: "Ação/ActionWithNote",
  component: ActionWithNote,
  args: {
    children: <Button onClick={fn()}>Iniciar torneio</Button>,
  },
  argTypes: {
    align: { control: "inline-radio", options: ["start", "end"] },
    children: { control: false },
    note: { control: "text" },
  },
} satisfies Meta<typeof ActionWithNote>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Sem nota nem erro, só o botão: o invólucro existe sempre. */
export const Plain: Story = {};

/** O motivo de não dar, embaixo do botão desabilitado. */
export const WithNote: Story = {
  args: {
    note: "Faltam 4 inscrições para o mínimo de 8.",
    children: <Button disabled>Iniciar torneio</Button>,
  },
};

/** O erro da última tentativa, já traduzido. */
export const WithError: Story = {
  args: { error: "Não foi possível iniciar o torneio. Tente de novo." },
};

export const NoteAndError: Story = {
  args: {
    note: "Faltam 4 inscrições para o mínimo de 8.",
    error: "Não foi possível iniciar o torneio. Tente de novo.",
    children: <Button disabled>Iniciar torneio</Button>,
  },
};

/** Na célula de ações de uma tabela: alinhado à direita. */
export const AlignEnd: Story = {
  args: {
    align: "end",
    note: "Faltam 4 inscrições para o mínimo de 8.",
    children: (
      <Button size="sm" disabled>
        Iniciar
      </Button>
    ),
  },
};

/** Numa linha de botões, o `note` vira a dica do botão (aqui aberta pelo mouse). */
export const Compact: Story = {
  args: {
    compact: true,
    note: "Faltam 4 inscrições para o mínimo de 8.",
    children: <Button>Iniciar torneio</Button>,
  },
  play: async ({ canvasElement }) => {
    await userEvent.hover(within(canvasElement).getByRole("button"));
    await expect(await screen.findByRole("tooltip")).toBeVisible();
  },
};

/** Botão `disabled` não recebe mouse nem foco: com `blocked`, o invólucro recebe e abre a dica. */
export const CompactBlocked: Story = {
  args: {
    compact: true,
    blocked: true,
    note: "Faltam 4 inscrições para o mínimo de 8.",
    children: <Button disabled>Iniciar torneio</Button>,
  },
  play: async ({ canvasElement }) => {
    await userEvent.hover(within(canvasElement).getByRole("group"));
    await expect(await screen.findByRole("tooltip")).toBeVisible();
  },
};

/** No modo compacto o erro continua embaixo, porque precisa ser visto. */
export const CompactWithError: Story = {
  args: {
    compact: true,
    error: "Não foi possível iniciar o torneio. Tente de novo.",
  },
};

/** Uma linha de ações do cabeçalho: com a dica, os botões vizinhos não se desalinham. */
export const HeaderRow: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      <ActionWithNote
        {...args}
        compact
        blocked
        note="Faltam 4 inscrições para o mínimo de 8."
      >
        <Button disabled>Iniciar torneio</Button>
      </ActionWithNote>
      <ActionWithNote {...args} compact>
        <Button kind="secondary">Editar</Button>
      </ActionWithNote>
    </Stack>
  ),
};
