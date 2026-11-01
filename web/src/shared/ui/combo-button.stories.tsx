import { Copy, DocumentExport } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { ComboButton } from "./combo-button";

const meta = {
  title: "Ação/ComboButton",
  component: ComboButton,
  args: {
    label: "Publicar",
    onClick: fn(),
    items: [
      { label: "Salvar como rascunho", onSelect: fn() },
      { label: "Duplicar torneio", onSelect: fn() },
      { label: "Agendar publicação", onSelect: fn(), disabled: true },
    ],
  },
  argTypes: {
    items: { control: "object" },
    size: { control: "inline-radio", options: ["xs", "sm", "md", "lg"] },
    menuAlignment: {
      control: "select",
      options: [
        "top",
        "top-start",
        "top-end",
        "bottom",
        "bottom-start",
        "bottom-end",
      ],
    },
    tooltipAlignment: {
      control: "select",
      options: ["top", "bottom", "left", "right"],
    },
  },
} satisfies Meta<typeof ComboButton>;

export default meta;
type Story = StoryObj<typeof meta>;

const openParameters = { docs: { story: { inline: false, height: "400px" } } };

/** Abre o menu pela seta (o segundo botão; o tooltip dela é "Mais ações"). */
const openMenu: Story["play"] = async ({ canvasElement }) => {
  await userEvent.click(within(canvasElement).getAllByRole("button")[1]);
  await expect(await screen.findByRole("menu")).toBeVisible();
};

/** A ação principal e a seta das alternativas. */
export const Default: Story = {};

export const MenuOpen: Story = {
  parameters: openParameters,
  play: openMenu,
};

export const Disabled: Story = { args: { disabled: true } };

/** A ação principal em andamento: ignora o clique, o indicador fica ao lado e o menu trava. */
export const Pending: Story = { args: { pending: true } };

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={5}>
      {(["xs", "sm", "md", "lg"] as const).map((size) => (
        <ComboButton key={size} {...args} size={size} label={size} />
      ))}
    </Stack>
  ),
};

/** Alternativa destrutiva no fim, depois de um divisor, com confirmação. */
export const WithDanger: Story = {
  args: {
    items: [
      { label: "Salvar como rascunho", onSelect: fn() },
      "divider",
      {
        label: "Descartar alterações",
        destructive: true,
        onSelect: fn(),
        confirm: { question: "O que mudou desde a última vez se perde." },
      },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

export const WithIcons: Story = {
  args: {
    label: "Salvar",
    items: [
      { label: "Salvar uma cópia", icon: Copy, onSelect: fn() },
      { label: "Exportar", icon: DocumentExport, onSelect: fn() },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

/** O menu acima do botão, para quando ele fica no pé da tela. */
export const MenuAlignmentTop: Story = {
  args: { menuAlignment: "top" },
  parameters: { layout: "centered", ...openParameters },
  play: openMenu,
};
