import { Form, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { FormSection } from "./form-section";
import { NumberInput } from "./number-input";
import { TextInput } from "./text-input";

const meta = {
  title: "Formulário/FormSection",
  component: FormSection,
  args: {
    legend: "Dados do torneio",
    children: [
      <TextInput
        key="name"
        id="form-section-name"
        labelText="Nome do torneio"
        required
      />,
      <TextInput
        key="slug"
        id="form-section-slug"
        labelText="Endereço"
        helperText="Aparece no link do torneio."
      />,
    ],
  },
  argTypes: {
    legend: { control: "text" },
    description: { control: "text" },
    children: { control: false },
  },
} satisfies Meta<typeof FormSection>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Uma frase sobre a seção, antes dos campos. */
export const WithDescription: Story = {
  args: {
    description: "Quem tem o link consegue ver a página do torneio.",
  },
};

/** Formulário com mais de um grupo: uma seção por grupo, com `Stack gap={7}` entre elas. */
export const Multiple: Story = {
  render: () => (
    <Form>
      <Stack gap={7}>
        <FormSection legend="Dados do torneio">
          <TextInput
            id="form-section-multi-name"
            labelText="Nome do torneio"
            required
          />
        </FormSection>
        <FormSection
          legend="Vagas"
          description="O sorteio das séries usa as vagas preenchidas."
        >
          <NumberInput
            id="form-section-multi-slots"
            labelText="Número de vagas"
            value={16}
            min={2}
            max={64}
            onChange={() => {}}
          />
        </FormSection>
      </Stack>
    </Form>
  ),
};
