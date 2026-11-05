import { Add, ArrowRight } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import type { ButtonKind } from "./button";
import { ButtonLink } from "./button-link";
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
  title: "Ação/ButtonLink",
  component: ButtonLink,
  args: { href: "/login", children: "Entrar" },
  argTypes: {
    kind: { control: "select", options: kinds },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    href: { control: "text" },
    renderIcon: { control: false },
  },
} satisfies Meta<typeof ButtonLink>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {};

export const Secondary: Story = {
  args: { kind: "secondary", children: "Criar conta" },
};

export const Tertiary: Story = {
  args: { kind: "tertiary", children: "Ver os inscritos" },
};

export const Ghost: Story = { args: { kind: "ghost", children: "Editar" } };

export const Danger: Story = { args: { kind: "danger", children: "Excluir" } };

export const DangerTertiary: Story = {
  args: { kind: "danger--tertiary", children: "Excluir" },
};

export const DangerGhost: Story = {
  args: { kind: "danger--ghost", children: "Excluir" },
};

export const WithIcon: Story = {
  args: { children: "Criar comunidade", renderIcon: Add },
};

export const Small: Story = {
  args: {
    size: "sm",
    kind: "tertiary",
    children: "Ver série",
    renderIcon: ArrowRight,
  },
};

/** Todos os tipos lado a lado, para comparar a hierarquia. */
export const Kinds: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {kinds.map((kind) => (
        <ButtonLink key={kind} {...args} kind={kind}>
          {kind}
        </ButtonLink>
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <ButtonLink key={size} {...args} size={size}>
          {size}
        </ButtonLink>
      ))}
    </Stack>
  ),
};
