import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useActionState, useId, useState } from "react";
import { fn } from "storybook/test";

import { useFocusInvalid } from "@/shared/hooks/use-focus-invalid";
import { useFormValidity } from "@/shared/hooks/use-form-validity";
import {
  type FormResult,
  fieldId,
  formError,
  invalidProps,
} from "@/shared/lib/form-result";
import { slugChars } from "@/shared/lib/input-filters";
import { Form } from "./carbon/form";
import { SelectItem } from "./carbon/select";
import { Stack } from "./carbon/stack";
import { Checkbox } from "./checkbox";
import { DateTimeInput } from "./date-time-input";
import { FormActions } from "./form-actions";
import { FormError } from "./form-error";
import { FormSection } from "./form-section";
import { Notice } from "./notice";
import { NumberInput } from "./number-input";
import { RadioGroup } from "./radio-group";
import { Select } from "./select";
import { TextArea } from "./text-area";
import { TextInput } from "./text-input";

type Field = "name" | "slug";

type Values = {
  name: string;
  slug: string;
  format: string;
  slots: number | null;
  startsAt: string | null;
  series: string;
  description: string;
  checkIn: boolean;
};

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Um formulário de torneio sem API: o envio espera um instante e responde.
 * Nome "Copa Telefrag" volta como erro do campo; endereço "erro" volta como
 * erro do formulário; o resto salva.
 */
function TournamentForm({ onSubmit }: { onSubmit: (values: Values) => void }) {
  const prefix = useId();
  const { ref, canSubmit } = useFormValidity();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [format, setFormat] = useState("");
  const [slots, setSlots] = useState<number | null>(16);
  const [startsAt, setStartsAt] = useState<string | null>(null);
  const [series, setSeries] = useState("md3");
  const [description, setDescription] = useState("");
  const [checkIn, setCheckIn] = useState(true);

  const [result, submit, pending] = useActionState<FormResult<Field>>(
    async () => {
      onSubmit({
        name,
        slug,
        format,
        slots,
        startsAt,
        series,
        description,
        checkIn,
      });
      await sleep(800);
      if (name.trim().toLowerCase() === "copa telefrag") {
        return { error: "Já existe um torneio com este nome.", field: "name" };
      }
      if (slug === "erro") {
        return { error: "Não foi possível criar o torneio. Tente de novo." };
      }
      return { saved: true };
    },
    {},
  );
  useFocusInvalid(prefix, result);
  const general = formError(result);
  // A data tem de estar à frente de agora: regra que só o formulário sabe.
  const inThePast = startsAt !== null && new Date(startsAt) <= new Date();

  return (
    <Form ref={ref} action={submit}>
      <Stack gap={7}>
        <FormSection legend="Dados do torneio">
          <TextInput
            id={fieldId(prefix, "name")}
            labelText="Nome do torneio"
            name="name"
            required
            maxLength={64}
            autoComplete="off"
            value={name}
            onChange={(event) => setName(event.target.value)}
            {...invalidProps(result, "name")}
          />
          <TextInput
            id={fieldId(prefix, "slug")}
            labelText="Endereço"
            helperText="Aparece no link do torneio."
            name="slug"
            required
            minLength={3}
            maxLength={32}
            pattern="[a-z0-9]+(-[a-z0-9]+)*"
            patternMessage="Use letras minúsculas, números e hífen."
            filter={slugChars}
            autoComplete="off"
            spellCheck={false}
            value={slug}
            onChange={(event) => setSlug(event.target.value)}
            {...invalidProps(result, "slug")}
          />
          <TextArea
            id={fieldId(prefix, "description")}
            labelText="Descrição"
            rows={3}
            enableCounter
            maxCount={280}
            value={description}
            onChange={(event) => setDescription(event.target.value)}
          />
        </FormSection>
        <FormSection legend="Formato">
          <Select
            id={fieldId(prefix, "format")}
            labelText="Formato"
            required
            value={format}
            onChange={(event) => setFormat(event.target.value)}
          >
            <SelectItem value="" text="Escolha o formato" />
            <SelectItem value="duel" text="Duelo" />
            <SelectItem value="ffa" text="Todos contra todos" />
            <SelectItem value="2v2" text="2v2" />
          </Select>
          <NumberInput
            id={fieldId(prefix, "slots")}
            labelText="Número de vagas"
            required
            min={2}
            max={64}
            value={slots}
            onChange={setSlots}
          />
          <RadioGroup
            legendText="Série"
            name={fieldId(prefix, "series")}
            orientation="horizontal"
            options={[
              { value: "md1", label: "md1" },
              { value: "md3", label: "md3" },
              { value: "md5", label: "md5" },
            ]}
            value={series}
            onChange={setSeries}
          />
        </FormSection>
        <FormSection legend="Datas">
          <DateTimeInput
            id={fieldId(prefix, "startsAt")}
            labelText="Início"
            required
            value={startsAt}
            onChange={setStartsAt}
            problem={inThePast ? "Escolha uma data no futuro." : undefined}
          />
          <Checkbox
            id={fieldId(prefix, "checkIn")}
            labelText="Exigir check-in"
            checked={checkIn}
            onChange={setCheckIn}
          />
        </FormSection>
        {result.saved && <Notice kind="success">Torneio criado.</Notice>}
        {general && <FormError>{general}</FormError>}
        <FormActions
          submitLabel="Criar torneio"
          pending={pending}
          disabled={!canSubmit}
          onCancel={fn()}
        />
      </Stack>
    </Form>
  );
}

const meta = {
  title: "Formulário/Exemplo completo",
  component: TournamentForm,
  args: { onSubmit: fn() },
  argTypes: { onSubmit: { control: false } },
} satisfies Meta<typeof TournamentForm>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * O botão fica desabilitado enquanto faltar campo obrigatório ou houver campo
 * inválido. Tente: nome "Copa Telefrag" (erro do campo, com foco nele),
 * endereço "erro" (erro do formulário), início no passado, vagas fora de 2 a 64.
 */
export const Empty: Story = {};
