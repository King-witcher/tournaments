import { Add, TrashCan } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Button, type ButtonKind } from "./button";
import { Stack } from "./carbon/stack";

const kinds: ButtonKind[] = [
  "primary",
  "secondary",
  "tertiary",
  "ghost",
  "danger",
  "danger--tertiary",
  "danger--ghost",
];

const meta = {
  title: "Ação/Button",
  component: Button,
  args: { children: "Salvar", onClick: fn() },
  argTypes: {
    kind: { control: "select", options: kinds },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    renderIcon: { control: false },
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};

export const Secondary: Story = { args: { kind: "secondary" } };

export const Tertiary: Story = { args: { kind: "tertiary" } };

export const Ghost: Story = { args: { kind: "ghost" } };

export const Danger: Story = {
  args: { kind: "danger", children: "Excluir", renderIcon: TrashCan },
};

export const DangerTertiary: Story = {
  args: { kind: "danger--tertiary", children: "Excluir" },
};

export const DangerGhost: Story = {
  args: { kind: "danger--ghost", children: "Excluir" },
};

export const WithIcon: Story = {
  args: { children: "Criar torneio", renderIcon: Add },
};

export const Disabled: Story = { args: { disabled: true } };

/** Ignora o clique e mostra o indicador, sem `disabled` (o foco fica). */
export const Pending: Story = { args: { pending: true } };

/** Todos os tipos em cada estado, para comparar a hierarquia. */
export const Kinds: Story = {
  render: (args) => (
    <Stack gap={5}>
      {[{}, { disabled: true }, { pending: true }].map((state) => (
        <Stack key={JSON.stringify(state)} orientation="horizontal" gap={3}>
          {kinds.map((kind) => (
            <Button key={kind} {...args} {...state} kind={kind}>
              {kind}
            </Button>
          ))}
        </Stack>
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Button key={size} {...args} size={size}>
          {size}
        </Button>
      ))}
    </Stack>
  ),
};
