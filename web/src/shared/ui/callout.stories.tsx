import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Callout } from "./callout";
import { Stack } from "./carbon/stack";
import { Link } from "./link";

const kinds = ["info", "warning"] as const;

const meta = {
  title: "Estado e conteúdo/Callout",
  component: Callout,
  args: {
    kind: "info",
    title: "Veto de mapas",
    subtitle: "Quem está no lado de cima da chave começa banindo.",
  },
  argTypes: {
    kind: { control: "inline-radio", options: kinds },
    title: { control: "text" },
    subtitle: { control: "text" },
    actionButtonLabel: { control: "text" },
    onActionButtonClick: { control: false },
    children: { control: false },
  },
} satisfies Meta<typeof Callout>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Info: Story = {};

export const Warning: Story = {
  args: {
    kind: "warning",
    title: "Poucas vagas",
    subtitle: "Restam 2 vagas nas inscrições.",
  },
};

export const LowContrast: Story = { args: { lowContrast: true } };

/** O botão aponta para o título (`aria-describedby`), que ganha `id` gerado. */
export const WithAction: Story = {
  args: { actionButtonLabel: "Ver os mapas", onActionButtonClick: fn() },
};

/** Link no texto aponta para o título pelo `titleId`. */
export const WithLink: Story = {
  args: {
    titleId: "callout-rules",
    title: "Regras do torneio",
    subtitle: undefined,
    children: (
      <div>
        Leia as{" "}
        <Link
          inline
          href="/arena-sul/copa/rules"
          aria-describedby="callout-rules"
        >
          regras completas
        </Link>{" "}
        antes de se inscrever.
      </div>
    ),
  },
};

export const OnlyTitle: Story = {
  args: { title: "As séries são md3", subtitle: undefined },
};

/** Os dois tipos, em alto e em baixo contraste. */
export const Kinds: Story = {
  render: (args) => (
    <Stack gap={5}>
      {[false, true].map((lowContrast) =>
        kinds.map((kind) => (
          <Callout
            key={`${kind}-${lowContrast}`}
            {...args}
            kind={kind}
            lowContrast={lowContrast}
            title={kind}
          />
        )),
      )}
    </Stack>
  ),
};
