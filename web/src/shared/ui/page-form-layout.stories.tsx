import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { FormActions } from "./form-actions";
import { FormError } from "./form-error";
import { FormRow } from "./form-row";
import { PageFormLayout } from "./page-form-layout";
import { TextArea } from "./text-area";
import { TextInput } from "./text-input";

const meta = {
  title: "Formulário/PageFormLayout",
  component: PageFormLayout,
  args: {
    children: [
      <TextInput key="name" id="page-form-name" labelText="Nome" required />,
      <TextArea key="about" id="page-form-about" labelText="Descrição" />,
    ],
    footer: <FormActions />,
  },
} satisfies Meta<typeof PageFormLayout>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithError: Story = {
  args: {
    footer: (
      <>
        <FormError>Não foi possível salvar.</FormError>
        <FormActions />
      </>
    ),
  },
};

export const WithRow: Story = {
  args: {
    children: [
      <FormRow key="row" columns={2}>
        <TextInput id="page-form-first" labelText="Fator K inicial" />
        <TextInput id="page-form-last" labelText="Fator K mínimo" />
      </FormRow>,
      <TextInput key="decay" id="page-form-decay" labelText="Queda do K" />,
    ],
  },
};
