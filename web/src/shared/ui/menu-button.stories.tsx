import { Document, DocumentPdf, Image, TableSplit } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { Stack } from "./carbon/stack";
import { MenuButton } from "./menu-button";
import type { MenuEntry } from "./use-menu-actions";

const items: MenuEntry[] = [
  { label: "Chave em PDF", onSelect: fn() },
  { label: "Classificação em CSV", onSelect: fn() },
  { label: "Inscritos em CSV", onSelect: fn(), disabled: true },
];

const meta = {
  title: "Ação/MenuButton",
  component: MenuButton,
  args: { label: "Exportar", items },
  argTypes: {
    items: { control: "object" },
    kind: {
      control: "inline-radio",
      options: ["primary", "tertiary", "ghost"],
    },
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
  },
} satisfies Meta<typeof MenuButton>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O menu aberto cobre a página de documentação: cada história com ele aberto fica num quadro. */
const openParameters = { docs: { story: { inline: false, height: "400px" } } };

const openMenu: Story["play"] = async ({ canvasElement }) => {
  await userEvent.click(within(canvasElement).getAllByRole("button")[0]);
  await expect(await screen.findByRole("menu")).toBeVisible();
};

/** Fechado: o nome comum das ações e a seta. */
export const Default: Story = {};

/** Aberto: o item desabilitado fica no lugar, sem responder. */
export const MenuOpen: Story = {
  parameters: openParameters,
  play: openMenu,
};

export const Tertiary: Story = { args: { kind: "tertiary" } };

export const Ghost: Story = { args: { kind: "ghost" } };

export const Disabled: Story = { args: { disabled: true } };

/** Uma das ações está em andamento: o indicador aparece ao lado e os itens travam. */
export const Pending: Story = {
  args: { pending: true },
  parameters: openParameters,
  play: openMenu,
};

/** Os três tipos lado a lado. */
export const Kinds: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["primary", "tertiary", "ghost"] as const).map((kind) => (
        <MenuButton key={kind} {...args} kind={kind} label={kind} />
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), `sm` e `md` ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["xs", "sm", "md", "lg"] as const).map((size) => (
        <MenuButton key={size} {...args} size={size} label={size} />
      ))}
    </Stack>
  ),
};

/** Item destrutivo no fim, separado por um divisor: vermelho e, com `confirm`, o Modal. */
export const WithDanger: Story = {
  args: {
    label: "Ações",
    items: [
      { label: "Editar torneio", onSelect: fn() },
      { label: "Duplicar torneio", onSelect: fn() },
      "divider",
      {
        label: "Excluir torneio",
        destructive: true,
        onSelect: fn(),
        confirm: { question: "O torneio e todas as séries serão apagados." },
      },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

/** Divisores separam os grupos de ações. */
export const WithDividers: Story = {
  args: {
    items: [
      { label: "Chave em PDF", onSelect: fn() },
      { label: "Chave em PNG", onSelect: fn() },
      "divider",
      { label: "Classificação em CSV", onSelect: fn() },
      { label: "Inscritos em CSV", onSelect: fn() },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

export const WithIcons: Story = {
  args: {
    items: [
      { label: "Chave em PDF", icon: DocumentPdf, onSelect: fn() },
      { label: "Chave em PNG", icon: Image, onSelect: fn() },
      { label: "Classificação em CSV", icon: TableSplit, onSelect: fn() },
      { label: "Regulamento", icon: Document, onSelect: fn() },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

/** Item com `items` abre um submenu ao lado (seta para a direita no teclado). */
export const WithNestedMenu: Story = {
  args: {
    label: "Ações",
    items: [
      { label: "Salvar", shortcut: "Ctrl+S", onSelect: fn() },
      {
        label: "Exportar como",
        items: [
          { label: "PDF", onSelect: fn() },
          { label: "PNG", onSelect: fn() },
          { label: "CSV", onSelect: fn() },
        ],
      },
      "divider",
      { label: "Excluir", destructive: true, onSelect: fn() },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

/** Item com `href` leva a outra página pelo roteador. */
export const WithLink: Story = {
  args: {
    label: "Ir para",
    items: [
      { label: "Inscritos", href: "/login" },
      { label: "Chave", href: "/login" },
    ],
  },
  parameters: openParameters,
  play: openMenu,
};

/** As seis posições do menu em relação ao botão (`menuAlignment`). */
export const WithMenuAlignment: Story = {
  parameters: {
    layout: "centered",
    docs: { story: { inline: false, height: "600px" } },
  },
  render: (args) => (
    <Stack gap={13}>
      <Stack orientation="horizontal" gap={7}>
        {(["bottom", "bottom-start", "bottom-end"] as const).map((align) => (
          <MenuButton
            key={align}
            {...args}
            label={align}
            menuAlignment={align}
          />
        ))}
      </Stack>
      <Stack orientation="horizontal" gap={7}>
        {(["top", "top-start", "top-end"] as const).map((align) => (
          <MenuButton
            key={align}
            {...args}
            label={align}
            menuAlignment={align}
          />
        ))}
      </Stack>
    </Stack>
  ),
};

/** Opção que liga e desliga e escolha entre várias, sempre controladas. */
function Selectable(args: Parameters<typeof MenuButton>[0]) {
  const [checkIn, setCheckIn] = useState(true);
  const [format, setFormat] = useState("md3");
  return (
    <MenuButton
      {...args}
      label="Exibir"
      items={[
        { label: "Só com check-in", checked: checkIn, onChange: setCheckIn },
        "divider",
        {
          label: "Série",
          value: format,
          onChange: setFormat,
          options: [
            { value: "md1", label: "Md1" },
            { value: "md3", label: "Md3" },
            { value: "md5", label: "Md5" },
          ],
        },
      ]}
    />
  );
}

export const WithSelectableItems: Story = {
  render: (args) => <Selectable {...args} />,
  parameters: openParameters,
  play: openMenu,
};
