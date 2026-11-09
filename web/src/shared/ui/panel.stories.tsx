import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { Panel } from "./panel";
import { Section } from "./section";
import { TextInput } from "./text-input";

const meta = {
  title: "Página/Panel",
  component: Panel,
  parameters: { layout: "padded" },
} satisfies Meta<typeof Panel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O texto de leitura, com o título da seção dentro. */
export const Default: Story = {
  args: {
    children: (
      <Section title="Como funciona" description="Gerado da configuração.">
        <p>
          O formato é suíço, em 5 rodadas. Na primeira rodada, os pares saem da
          ordem da chave.
        </p>
      </Section>
    ),
  },
};

/** O campo dentro do container usa a camada seguinte: cinza sobre o branco. */
export const WithField: Story = {
  args: {
    children: (
      <Stack gap={5}>
        <TextInput id="panel-name" labelText="Nome" defaultValue="Liga" />
      </Stack>
    ),
  },
};

/** Dois assuntos, um container para cada, com o espaço do `Stack`. */
export const Stacked: Story = {
  args: { children: null },
  render: () => (
    <Stack gap={5}>
      <Panel>
        <Section title="Como funciona">
          <p>Texto gerado.</p>
        </Section>
      </Panel>
      <Panel>
        <Section title="Regras da organização">
          <p>Texto da organização.</p>
        </Section>
      </Panel>
    </Stack>
  ),
};
