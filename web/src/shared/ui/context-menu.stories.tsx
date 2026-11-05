import { Copy, Edit } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { type ComponentProps, useRef, useState } from "react";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { Tile } from "./carbon/tile";
import { ContextMenu } from "./context-menu";
import type { MenuEntry } from "./use-menu-actions";

type Args = Omit<ComponentProps<typeof ContextMenu>, "trigger">;

/** A série com o menu do clique direito. */
function SeriesTile(args: Args) {
  const target = useRef<HTMLDivElement>(null);
  return (
    <>
      <Tile ref={target}>
        Série C1 · Lobo x Mariana (clique com o botão direito)
      </Tile>
      <ContextMenu {...args} trigger={target} />
    </>
  );
}

const actions: MenuEntry[] = [
  { label: "Editar série", icon: Edit, shortcut: "E", onSelect: fn() },
  { label: "Copiar o link", icon: Copy, onSelect: fn() },
  { label: "Reabrir série", disabled: true },
  "divider",
  {
    label: "Dar W.O.",
    destructive: true,
    onSelect: fn(),
    confirm: { question: "Mariana vence a série por W.O." },
  },
];

const meta = {
  title: "Ação/ContextMenu",
  component: ContextMenu,
  // `trigger` vazio: o `SeriesTile` passa a ref do Tile.
  args: { label: "Ações da série", items: actions, trigger: { current: null } },
  argTypes: {
    items: { control: "object" },
    trigger: { control: false },
    size: { control: "inline-radio", options: ["xs", "sm", "md", "lg"] },
  },
  render: (args) => <SeriesTile {...args} />,
  parameters: { docs: { story: { inline: false, height: "400px" } } },
} satisfies Meta<typeof ContextMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

const openMenu: Story["play"] = async ({ canvasElement }) => {
  const target = within(canvasElement).getByText(/Série C1/);
  await userEvent.pointer({ keys: "[MouseRight]", target });
  await expect(await screen.findByRole("menu")).toBeVisible();
};

/** Clique direito na série: ícones, atalho, item desabilitado e o destrutivo no fim. */
export const Default: Story = { play: openMenu };

/** Item destrutivo com `confirm`: o Modal pergunta antes. */
export const WithConfirm: Story = {
  play: async (context) => {
    await openMenu(context);
    await userEvent.click(
      await screen.findByRole("menuitem", { name: "Dar W.O." }),
    );
    await expect(await screen.findByRole("dialog")).toBeVisible();
  },
};

/** Opções que ligam e desligam, num grupo, e uma escolha entre várias num submenu. */
function SelectableTile(args: Args) {
  const [scores, setScores] = useState(true);
  const [maps, setMaps] = useState(false);
  const [order, setOrder] = useState("seed");
  return (
    <SeriesTile
      {...args}
      items={[
        {
          group: "Mostrar",
          items: [
            { label: "Placar", checked: scores, onChange: setScores },
            { label: "Mapas", checked: maps, onChange: setMaps },
          ],
        },
        "divider",
        {
          label: "Ordenar por",
          items: [
            {
              label: "Ordenar por",
              value: order,
              onChange: setOrder,
              options: [
                { value: "seed", label: "Ordem da chave" },
                { value: "rating", label: "Rating" },
                { value: "name", label: "Nome" },
              ],
            },
          ],
        },
      ]}
    />
  );
}

export const WithSelectableItems: Story = {
  render: (args) => <SelectableTile {...args} />,
  play: openMenu,
};

export const Medium: Story = { args: { size: "md" }, play: openMenu };
