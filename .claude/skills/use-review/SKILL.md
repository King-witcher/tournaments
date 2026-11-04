---
name: use-review
description: Contém instruções de como analisar um arquivo com feedback de uma revisão da plataforma, e proceder para a correção.
---

As revisões ficam em gitignore/prds/revisao-n.md, onde n é um número.
Estes arquivos contém comentários sobre o que eu acho que precisa ser mudado na plataforma, ou que achei ruim e quero discutir ideias de como poderia ter sido feito melhor.

Leia com pensamento crítico. Você é livre para discordar de mim em qualquer ponto e propor soluções melhores do que estou enxergando para os problemas que tento resolver, ou se alguma ideia minha não tiver sentido ou tiver algum problema que eu posso não estar percebendo.
Suas sugestões também devem, onde possível, explicar um motivo. O objetivo é que, apesar de a decisão final ser minha, ela venha de uma discussão entre eu e você.
Use e abuse de AskUserQuestion quando precisar de decisões ou confirmação minha, especialmente no que é caro de reverter depois. No entanto, decisões mais tranquilas ou baratas de reverter eu deixo você tomar, para acelerar o processo.

Os arquivos de revisão costumam vir com vários feedbacks simultâneos, muitas vezes em áreas com pouco ou nenhum conflito. Fazer tudo num agente só consumiria tokens e tempo em excesso por manter tudo num único contexto, então você deve separar em frentes de subagentes e worktrees, separando o que for aplicável, ou mantendo no mesmo agente o que for realmente se beneficiar de usar um único contexto ou for muito rápido de resolver. Veja a skill /worktree.

Você é o responsável por orquestrar os subagentes, lançar no momento certo e dar merge conforme terminarem.

Você também pode escolher entre Sonnet e Opus, dependendo da complexidade da tarefa.
Evite lançar Sonnet para tarefas mais complexas: ele costuma deixar passar bugs e decisões ruins (mas não cheguei a testar a fundo neste projeto com boa cobertura de testes).
Os subagentes podem lançar o agent intestigator conforme precisarem.
