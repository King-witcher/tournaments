# Textos (messages)

Todo texto visível do site sai daqui (next-intl); o resto do frontend está em `web/CLAUDE.md`. Os emails e os avisos do Discord, escritos na API, seguem "Termos da interface" e "Como escrever".

- Um arquivo por namespace de primeiro nível (`messages/pt/Tenant.json`, `messages/es/Tenant.json` e `messages/en/Tenant.json`); chave nova vai no namespace do seu fluxo, nos três idiomas. `es` e `en` precisam ter as mesmas chaves de `pt` (o `tsc` e o `build` acusam).
- Namespace novo: crie os três arquivos e acrescente uma linha em cada lista (`pt`, `es` e `en`) de `src/i18n/namespaces.ts`, em ordem alfabética.
- O navegador recebe todos os namespaces menos os de `SERVER_ONLY_MESSAGES` (`namespaces.ts`), que só Server Components leem, e de novo a cada `router.refresh()`. Namespace novo vai sozinho. Componente de cliente que passe a usar algo da lista: tire a linha de lá (senão aparece a chave no lugar do texto). Bloco grande que só o servidor lê: acrescente.
- `Common` (textos da biblioteca) e `Nav` são da base. Num namespace que outro fluxo também usa, só acrescente chaves: não reordene nem reformate o arquivo.
- Código novo de aviso da API: `Notifications.notices.<código>` (`title` e `body`) nos três idiomas, os mesmos na página e no service worker. Só `{param}` simples (o service worker não entende plural do ICU); o prazo `confirm_by` entra como `{deadline}`.
- O espanhol é em "tú". O inglês é americano (en-US), em "you", com as mesmas regras de "Como escrever". Plural do ICU em inglês só tem `one` e `other`; ordinal (`selectordinal`) tem `one`/`two`/`few`/`other`. Data digitada `mm/dd/yyyy` (`dateText` em `shared/lib/plain-date.ts`).

## Termos da interface

| Conceito                                                              | pt      | es      | en         | No código    |
| --------------------------------------------------------------------- | ------- | ------- | ---------- | ------------ |
| Um jogo só, num mapa                                                  | partida | partida | game       | `game`       |
| Md3, md5 (o confronto entre dois lados)                               | série   | serie   | match      | `match`      |
| O todo                                                                | torneio | torneo  | tournament | `tournament` |
| A arena (pool, veto, ban e pick de mapa, mapa decisivo, nome do mapa) | mapa    | mapa    | map        | `map`        |
- "Mapa" só quando é a arena. "Mapa enviado", "rating por mapa", "mapas jogados" e "placar mapa a mapa" são partidas. "Partida pronta", "Partidas que esperam você" e "C1 · partida 3" são séries.
- Concordância: a partida foi enviada, a série foi decidida.
- Outros termos já decididos: "ordem da chave" (seeding, nunca "seed"), "captura de tela", "W.O.", "check-in", "chave", "rating", "todos contra todos" (nunca "round robin"), "queda do K".
- Dois-pontos só em aposto, lista ou rótulo ("Rating: 1500"). Para explicar, ponha ponto e comece outro período: "Faça o check-in na página do torneio. Quem não fizer, não entra em nenhum time." A API também segue isto nos emails e nas mensagens do Discord.

## Como escrever

O texto diz só o que a pessoa não sabe e não vai descobrir logo em seguida. Para cada frase, pergunte o que ela faria diferente sem a frase; se nada, corte. Cortar não é ficar vago: nomeie a coisa ("a série pronta", "o fator K"), dê a data ou o número quando importam e não tire o que a pessoa precisa para decidir (o que se perde, o que não volta, o que falta fazer).

- **Não repita a tela.** Título, etiqueta, botão e coluna já falam: a etiqueta "Inscrição feita" dispensa "Você está inscrito."; o botão "Ver os inscritos" dispensa "Até lá, confira quem está inscrito."; o título "Nova senha" dispensa "Escolha a senha nova."
- **Estado vazio é o estado e a ação**: "Sem descrição." e o botão, não "Escreva uma para os membros saberem do que se trata…".
- **Fica de fora**: regra interna ("vale para os próximos envios"), mecânica que se percebe ao usar (o pedido de refazer no veto), o que se infere do resto (quem marca "Pronto" em "Quem age pelo time": sai de quem age), definição do óbvio ("Completo é o time com 2 jogadores"), o porquê de uma exigência evidente ("A conta precisa de pelo menos um jeito de entrar.") e a exceção de cada caso entre parênteses quando a regra geral basta ("quando abrir vaga", não "(alguém sair ou as vagas aumentarem)").
- **Cada coisa na hora e para quem age**: a consequência de uma ação no modal de confirmação, não acima do botão; o limite no campo e no erro, não na descrição; a explicação longa na página que a usa, não no resumo que leva a ela.
- **Um fato por frase, curta, com ponto no fim.** Sem jargão de sistema ("API", "banco") e sem nada com cara de texto gerado (dois-pontos encadeando explicação, "ou seja", listas de todos os casos).
