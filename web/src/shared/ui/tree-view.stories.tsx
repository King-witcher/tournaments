import { Folder, Trophy } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { type ComponentProps, useState } from "react";
import { fn } from "storybook/test";

import { Button } from "./button";
import { TreeNode, TreeView } from "./tree-view";

type Node = {
  id: string;
  label: string;
  /** Página do nó, na história com links. */
  href?: string;
  disabled?: boolean;
  isExpanded?: boolean;
  children?: Node[];
};

const nodes: Node[] = [
  {
    id: "arena-sul",
    label: "Arena Sul",
    href: "/arena-sul",
    isExpanded: true,
    children: [
      {
        id: "copa",
        label: "Copa Arena Sul",
        href: "/arena-sul/copa",
        isExpanded: true,
        children: [
          {
            id: "grupos",
            label: "Fase de grupos",
            href: "/arena-sul/copa/grupos",
          },
          {
            id: "mata-mata",
            label: "Mata-mata",
            href: "/arena-sul/copa/chave",
          },
        ],
      },
      { id: "duelo", label: "Duelo de sexta", href: "/arena-sul/duelo" },
      { id: "liga", label: "Liga de inverno (rascunho)", disabled: true },
    ],
  },
  {
    id: "quake-rs",
    label: "Quake RS",
    href: "/quake-rs",
    children: [
      { id: "2v2", label: "Mata-mata 2v2", href: "/quake-rs/2v2" },
      { id: "ffa", label: "FFA de domingo", href: "/quake-rs/ffa" },
    ],
  },
];

type RenderOptions = { icons?: boolean; links?: boolean; expanded?: boolean };

function renderNodes(list: Node[], options: RenderOptions = {}) {
  return list.map((node) => (
    <TreeNode
      key={node.id}
      id={node.id}
      label={node.label}
      disabled={node.disabled}
      isExpanded={options.expanded ?? node.isExpanded}
      href={options.links ? node.href : undefined}
      renderIcon={options.icons ? (node.children ? Folder : Trophy) : undefined}
    >
      {node.children && renderNodes(node.children, options)}
    </TreeNode>
  ));
}

const meta = {
  title: "Listas/TreeView",
  component: TreeView,
  args: {
    label: "Comunidades e torneios",
    onSelect: fn(),
    children: renderNodes(nodes),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["xs", "sm"] },
    children: { control: false },
  },
} satisfies Meta<typeof TreeView>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Nós que abrem e fecham (seta, Enter ou as setas do teclado); um nó desabilitado. */
export const Default: Story = {};

/** Ícone por nó (`renderIcon`): pasta para os que têm filhos. */
export const WithIcons: Story = {
  args: { children: renderNodes(nodes, { icons: true }) },
};

/** Com `href`, o nó é link para a página, pelo roteador do i18n. */
export const WithLinks: Story = {
  args: { children: renderNodes(nodes, { links: true }) },
};

/** O nome da árvore só para leitor de tela. */
export const HideLabel: Story = { args: { hideLabel: true } };

/** Vários nós selecionados com Ctrl ou Shift (experimental no Carbon). */
export const Multiselect: Story = { args: { multiselect: true } };

/** `xs` para árvore densa; `sm` é o padrão. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={7}>
      {(["xs", "sm"] as const).map((size) => (
        <TreeView key={size} {...args} size={size} label={size} />
      ))}
    </Stack>
  ),
};

/** Mais níveis: séries dentro das fases. */
export const WithComplexNesting: Story = {
  args: {
    children: renderNodes([
      {
        id: "copa",
        label: "Copa Arena Sul",
        isExpanded: true,
        children: [
          {
            id: "chave-superior",
            label: "Chave superior",
            isExpanded: true,
            children: [
              {
                id: "quartas",
                label: "Quartas de final",
                isExpanded: true,
                children: [
                  { id: "c1", label: "C1 · Lobo x Mariana" },
                  { id: "c2", label: "C2 · Rafael x Camila" },
                ],
              },
              { id: "semi", label: "Semifinal" },
            ],
          },
          { id: "chave-inferior", label: "Chave inferior" },
          { id: "final", label: "Final" },
        ],
      },
    ]),
  },
};

/** Quem usa abre e fecha tudo de fora, pelo `isExpanded` de cada nó. */
function ControlledExpansion(args: ComponentProps<typeof TreeView>) {
  const [expanded, setExpanded] = useState(false);
  return (
    <Stack gap={5}>
      <div>
        <Button
          kind="tertiary"
          size="sm"
          onClick={() => setExpanded(!expanded)}
        >
          {expanded ? "Fechar tudo" : "Abrir tudo"}
        </Button>
      </div>
      <TreeView {...args}>{renderNodes(nodes, { expanded })}</TreeView>
    </Stack>
  );
}

export const WithControlledExpansion: Story = {
  render: (args) => <ControlledExpansion {...args} />,
};
