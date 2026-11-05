import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { Column, Grid } from "./carbon/grid";
import { Stack } from "./carbon/stack";
import { OverflowActions } from "./overflow-actions";
import { TextInput } from "./text-input";

const meta = {
  title: "Ação/OverflowActions",
  component: OverflowActions,
  args: {
    items: [
      { label: "Editar torneio", onSelect: fn() },
      { label: "Duplicar torneio", onSelect: fn() },
      {
        label: "Excluir torneio",
        destructive: true,
        onSelect: fn(),
      },
    ],
  },
  // O menu abre para a esquerda (`flipped`), como no fim de uma linha ou do
  // cabeçalho: sem a coluna vazia ele sairia da tela.
  decorators: [
    (Story) => (
      <Grid>
        <Column sm={2} md={6} lg={12} />
        <Column sm={2} md={2} lg={4}>
          <Story />
        </Column>
      </Grid>
    ),
  ],
  argTypes: {
    items: { control: "object" },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    label: { control: "text" },
  },
} satisfies Meta<typeof OverflowActions>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O menu ⋯ fechado, como aparece na linha de uma tabela ou no cabeçalho. */
export const Default: Story = {};

/** Abre o menu: o item destrutivo vem em vermelho. */
export const MenuOpen: Story = {
  parameters: { docs: { story: { inline: false, height: "500px" } } },
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
    await expect(await screen.findByRole("menu")).toBeVisible();
  },
};

/** Um item desabilitado entre os outros. */
export const WithDisabledItem: Story = {
  args: {
    items: [
      { label: "Editar torneio", onSelect: fn() },
      { label: "Iniciar torneio", disabled: true },
      { label: "Excluir torneio", destructive: true, onSelect: fn() },
    ],
  },
  parameters: MenuOpen.parameters,
  play: MenuOpen.play,
};

/** Item com `href` leva a outra página em vez de rodar `onSelect`. */
export const WithLink: Story = {
  args: {
    items: [
      { label: "Ver inscritos", href: "/login" },
      { label: "Editar torneio", onSelect: fn() },
    ],
  },
  parameters: MenuOpen.parameters,
  play: MenuOpen.play,
};

/** Item com `confirm` abre o Modal de confirmação antes de rodar `onSelect`. */
export const WithConfirm: Story = {
  args: {
    items: [
      { label: "Editar torneio", onSelect: fn() },
      {
        label: "Excluir torneio",
        destructive: true,
        onSelect: fn(),
        confirm: { question: "O torneio e todas as séries serão apagados." },
      },
    ],
  },
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: "Excluir torneio" }),
    );
    await expect(await screen.findByRole("dialog")).toBeVisible();
  },
};

/** Grave e raro: o Modal pede o texto digitado. */
export const WithConfirmSlug: Story = {
  args: {
    items: [
      {
        label: "Transferir posse",
        destructive: true,
        onSelect: fn(),
        confirm: {
          title: "Transferir a comunidade",
          question:
            "A comunidade passa a ser da outra pessoa e você perde a posse.",
          slug: "copa-telefrag",
        },
      },
    ],
  },
  parameters: WithConfirm.parameters,
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: "Transferir posse" }),
    );
    await expect(await screen.findByRole("dialog")).toBeVisible();
  },
};

/**
 * `confirm.extra`: um campo no Modal (a nota de anular uma partida),
 * controlado por quem monta o menu.
 */
export const WithConfirmExtra: Story = {
  render: function Render() {
    const [note, setNote] = useState("");
    return (
      <OverflowActions
        items={[
          {
            label: "Anular e jogar de novo",
            destructive: true,
            onSelect: fn(),
            confirm: {
              title: "Anular a partida 2",
              question: "Os lados jogam a partida 2 de novo.",
              extra: (
                <TextInput
                  labelText="Nota no histórico"
                  value={note}
                  onChange={(e) => setNote(e.target.value)}
                />
              ),
            },
          },
        ]}
      />
    );
  },
  parameters: WithConfirm.parameters,
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
    await userEvent.click(
      await screen.findByRole("menuitem", { name: "Anular e jogar de novo" }),
    );
    await userEvent.type(
      await screen.findByLabelText("Nota no histórico"),
      "queda",
    );
    await expect(screen.getByLabelText("Nota no histórico")).toHaveValue(
      "queda",
    );
  },
};

/** Uma das ações está em andamento: o indicador aparece ao lado do ⋯ e os itens travam. */
export const Pending: Story = { args: { pending: true } };

export const CustomLabel: Story = { args: { label: "Ações da série" } };

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <OverflowActions key={size} {...args} size={size} />
      ))}
    </Stack>
  ),
};
