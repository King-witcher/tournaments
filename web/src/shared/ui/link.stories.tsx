import { ArrowRight } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Link } from "./link";

const meta = {
  title: "Página/Link",
  component: Link,
  args: { href: "/arena-brasil/copa-de-duelo", children: "Ver o torneio" },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    renderIcon: { control: false },
    locale: { control: false },
    children: { control: "text" },
  },
} satisfies Meta<typeof Link>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Solto: azul, e no celular com 48px de altura para o dedo. */
export const Standalone: Story = {};

export const WithIcon: Story = {
  args: { children: "Ver os inscritos", renderIcon: ArrowRight },
};

/** Dentro de um parágrafo: sublinhado sempre, sem a altura de toque. */
export const Inline: Story = {
  render: (args) => (
    <p>
      O torneio começa amanhã. Antes, leia as <Link {...args} inline /> da Arena
      Brasil.
    </p>
  ),
  args: { href: "/arena-brasil/regras", children: "regras" },
};

export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={5}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <Link key={size} {...args} size={size}>
          {size}
        </Link>
      ))}
    </Stack>
  ),
};

/** Solto e no meio do texto, lado a lado. */
export const StandaloneAndInline: Story = {
  render: (args) => (
    <Stack gap={5}>
      <Link {...args} />
      <p>
        Veja{" "}
        <Link {...args} inline>
          o torneio
        </Link>{" "}
        antes de se inscrever.
      </p>
    </Stack>
  ),
};

/** Abaixo de `md` o link solto tem 48px de altura. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
  render: StandaloneAndInline.render,
};
