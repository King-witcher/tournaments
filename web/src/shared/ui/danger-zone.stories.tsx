import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { ConfirmButton } from "./confirm-button";
import { DangerZone } from "./danger-zone";
import { Section } from "./section";

const meta = {
  title: "Ação/DangerZone",
  component: DangerZone,
  argTypes: { children: { control: false } },
} satisfies Meta<typeof DangerZone>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Cada ação é uma `Section level={3}` com o botão de perigo e a confirmação. */
export const Default: Story = {
  args: {
    children: (
      <Section
        level={3}
        title="Excluir comunidade"
        description="Os torneios e as inscrições são apagados."
      >
        <div>
          <ConfirmButton
            question="A comunidade e tudo o que há nela serão apagados."
            onConfirm={fn()}
          >
            Excluir comunidade
          </ConfirmButton>
        </div>
      </Section>
    ),
  },
};

export const SeveralActions: Story = {
  args: {
    children: (
      <>
        <Section
          level={3}
          title="Transferir posse"
          description="Você deixa de ser quem gerencia a comunidade."
        >
          <div>
            <ConfirmButton
              confirmSlug="copa-telefrag"
              question="A comunidade passa a ser da outra pessoa."
              onConfirm={fn()}
            >
              Transferir posse
            </ConfirmButton>
          </div>
        </Section>
        <Section level={3} title="Excluir comunidade">
          <div>
            <ConfirmButton
              question="A comunidade e tudo o que há nela serão apagados."
              onConfirm={fn()}
            >
              Excluir comunidade
            </ConfirmButton>
          </div>
        </Section>
      </>
    ),
  },
};
