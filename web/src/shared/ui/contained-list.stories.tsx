import { Add, Close, Events, Trophy } from "@carbon/icons-react";
import { Layer, Stack, Tag } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { type ComponentProps, useState } from "react";
import { fn } from "storybook/test";

import { ContainedList, ContainedListItem } from "./contained-list";
import { IconButton } from "./icon-button";

const MEMBERS = ["Lobo", "Mariana", "Rafael", "Camila", "Thiago", "Beatriz"];

const meta = {
  title: "Listas/ContainedList",
  component: ContainedList,
  args: {
    label: "Membros",
    children: MEMBERS.slice(0, 4).map((name) => (
      <ContainedListItem key={name}>{name}</ContainedListItem>
    )),
  },
  argTypes: {
    kind: { control: "inline-radio", options: ["on-page", "disclosed"] },
    size: { control: "inline-radio", options: ["sm", "md", "lg", "xl"] },
    isInset: { control: "boolean" },
    children: { control: false },
    action: { control: false },
    search: { control: false },
  },
} satisfies Meta<typeof ContainedList>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Na página (`on-page`), dentro de um card ou painel. */
export const Default: Story = {};

/** Dentro de algo que se abre (popover, painel temporário): o cabeçalho tem fundo. */
export const Disclosed: Story = { args: { kind: "disclosed" } };

/** As linhas divisórias recuadas, alinhadas ao texto. */
export const Inset: Story = { args: { isInset: true } };

/** `lg` é o padrão; abaixo de `md` (celular), `sm` e `md` ficam com 48px. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={7}>
      {(["sm", "md", "lg", "xl"] as const).map((size) => (
        <ContainedList key={size} {...args} size={size} label={size} />
      ))}
    </Stack>
  ),
};

/** Botão no cabeçalho (`action`). */
export const WithListAction: Story = {
  args: {
    action: (
      <IconButton label="Convidar membro" onClick={fn()}>
        <Add />
      </IconButton>
    ),
  },
};

/** Ação por item (`action` do item); o item desabilitado também a desabilita. */
export const WithActions: Story = {
  args: {
    children: MEMBERS.slice(0, 4).map((name, i) => (
      <ContainedListItem
        key={name}
        disabled={i === 1}
        action={
          <IconButton
            label={`Remover ${name}`}
            disabled={i === 1}
            onClick={fn()}
          >
            <Close />
          </IconButton>
        }
      >
        {name}
      </ContainedListItem>
    )),
  },
};

/** Com `onClick`, o item todo é um botão. */
export const WithInteractiveItems: Story = {
  args: {
    children: MEMBERS.slice(0, 4).map((name) => (
      <ContainedListItem key={name} onClick={fn()}>
        {name}
      </ContainedListItem>
    )),
  },
};

export const WithInteractiveItemsAndActions: Story = {
  args: {
    children: MEMBERS.slice(0, 4).map((name) => (
      <ContainedListItem
        key={name}
        onClick={fn()}
        action={
          <IconButton label={`Remover ${name}`} onClick={fn()}>
            <Close />
          </IconButton>
        }
      >
        {name}
      </ContainedListItem>
    )),
  },
};

/** Ícone à esquerda de cada item (`renderIcon`). */
export const WithIcons: Story = {
  args: {
    label: "Torneios",
    children: [
      { name: "Copa Arena Sul", icon: Trophy },
      { name: "Duelo de sexta", icon: Trophy },
      { name: "Mata-mata 2v2", icon: Events },
    ].map(({ name, icon }) => (
      <ContainedListItem key={name} renderIcon={icon}>
        {name}
      </ContainedListItem>
    )),
  },
};

/** Algo além do texto no título, como a contagem. */
export const WithListTitleDecorators: Story = {
  args: {
    label: (
      <Stack orientation="horizontal" gap={3}>
        <span>Membros</span>
        <Tag size="sm" type="gray">
          4
        </Tag>
      </Stack>
    ),
  },
};

/** Filtra os itens pelo texto da busca. */
function Searchable(args: ComponentProps<typeof ContainedList>) {
  const [query, setQuery] = useState("");
  const shown = MEMBERS.filter((name) =>
    name.toLowerCase().includes(query.trim().toLowerCase()),
  );
  return (
    <ContainedList {...args} search={{ ...args.search, onChange: setQuery }}>
      {shown.map((name) => (
        <ContainedListItem key={name}>{name}</ContainedListItem>
      ))}
    </ContainedList>
  );
}

/** A lupa no cabeçalho, que se expande ao clicar. */
export const WithExpandableSearch: Story = {
  render: (args) => <Searchable {...args} />,
};

/** O campo de busca sempre aberto, acima dos itens. */
export const WithPersistentSearch: Story = {
  args: { search: { onChange: fn(), persistent: true } },
  render: (args) => <Searchable {...args} />,
};

/** Dentro de um `Layer`, o fundo segue a camada. */
export const WithLayer: Story = {
  render: (args) => (
    <Stack gap={5}>
      <ContainedList {...args} />
      <Layer>
        <ContainedList {...args} />
        <Layer>
          <ContainedList {...args} />
        </Layer>
      </Layer>
    </Stack>
  ),
};
