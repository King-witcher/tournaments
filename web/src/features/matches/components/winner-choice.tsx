"use client";

type Props = {
  legend: string;
  names: { a: string; b: string };
  value: "a" | "b" | "";
  onChange: (winner: "a" | "b") => void;
};

/**
 * Quem venceu o mapa: um botão de opção por jogador, com o nome, lado a
 * lado. Opção nativa (`radio`) para o teclado e o leitor de tela; o rótulo
 * inteiro é a área de toque.
 */
export function WinnerChoice({ legend, names, value, onChange }: Props) {
  return (
    <fieldset className="flex flex-col gap-1">
      <legend className="text-sm font-medium">{legend}</legend>
      {/* Duas colunas iguais: os dois nomes à vista e fáceis de tocar no celular. */}
      <div className="grid grid-cols-2 gap-2">
        {(["a", "b"] as const).map((side) => (
          <label
            key={side}
            // Borda e fundo marcam a escolha; altura de toque confortável.
            className="flex min-w-0 cursor-pointer items-center gap-2 rounded-md border border-black/20 px-3 py-2 has-[:checked]:border-blue-600 has-[:checked]:bg-blue-600/10 dark:border-white/20"
          >
            <input
              type="radio"
              name={legend}
              checked={value === side}
              onChange={() => onChange(side)}
            />
            <span className="truncate">{names[side]}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}
