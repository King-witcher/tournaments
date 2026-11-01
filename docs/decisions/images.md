# Imagens públicas: logo e capa

O logo da comunidade e a capa do torneio (01/11). Decidido pelo usuário, salvo o "assumido".

## Forma e onde aparecem

- **Logo quadrado (1:1), capa 2:1.** O logo é identidade e aparece pequeno, ao lado do nome. A capa é 2:1 porque é a proporção do `HeroImage` do `PageHeader` do Carbon no `lg` (3:2 abaixo), está no conjunto de proporções do 2x Grid, e a `og:image` (1,91:1) sai dela quase sem corte.
- **Logo**: antes do título no cabeçalho da comunidade (o lugar do ícone do título no `PageHeader` do Carbon), nos tiles de "Minhas comunidades" e no título do convite. Fica fora do painel de comunidades do cabeçalho, porque o `Switcher` do Carbon é só texto (assumido).
- **Capa**: à direita do cabeçalho do torneio, só do `lg` para cima (no celular empurraria a ação primária e "Sua situação" para baixo); no topo dos tiles de "Seus torneios" e no tile do link de inscrição. A aba Torneios da comunidade (`DataTable`) fica sem imagem: numa linha de 48 px a capa teria 96×48.
- **Sem imagem**: numa grade com alguma capa ou logo, todo tile leva o espaço (sem capa, o logo da comunidade ou a inicial no centro), para as linhas não desalinharem; sem nenhuma, nenhum leva. Fora da grade, nada no lugar (assumido).
- `alt=""` em todas: o nome já está em texto ao lado.

## Quem troca e como

- **Logo: só o owner**, em Configurações (como o rating e o Discord). **Capa: organizer e owner**, na aba Geral de Editar torneio, em qualquer estado do torneio.
- **Na edição, salva sozinho**, fora do "Salvar" do formulário: o envio acontece ao escolher o arquivo (como o file uploader do Carbon), e no formulário ficaria o estado "imagem enviada, formulário não salvo".
- **Também na criação** (Criar comunidade e Novo torneio, 01/11), sem deixar nada perdido no bucket se a pessoa desistir: a imagem é cortada ao escolher e fica só no navegador (com a prévia) até a comunidade ou o torneio serem criados; aí segue o envio de sempre. Se ela não chega, a criação vale e a tela oferece "Tentar de novo" ou seguir sem a imagem. Pedir a URL antes, com o upload pendente sem dono, deixaria objetos para a rotina diária limpar e exigiria mudar a tabela; esperar a criação não deixa nenhum (a aba fechada no meio do envio cai no upload pendente, que a rotina já apaga).
- **Sem recorte manual** (o Carbon não tem cropper): o navegador corta pelo centro e reduz para **256×256** (logo) e **1200×600** (capa), em **JPEG**, até **1 MB**. Só JPEG (a API recusa WebP): o `ImageResponse` da prévia dos links não desenha WebP. Sem transparência, o logo sai sobre branco, o que também evita um logo escuro transparente sumir no tema g100.

## Bucket e URL

- **Bucket público à parte, com o Block Public Access ligado**, lido só pelo CloudFront (OAC). Um bucket só, com o CloudFront liberado por prefixo, deixaria os screenshots separados do mundo por uma linha da política; à parte, o privado nem tem o CloudFront na política. Domínio próprio no CDN (`img.telefrag.com.br`; o de staging fica em `img-staging.telefrag.com.br`, porque `staging.` é CNAME da Vercel, cujo CAA não libera a Amazon), para as prévias já postadas sobreviverem a uma troca de provedor.
- **O banco guarda só o nome do objeto**; a API monta a URL com `S3_PUBLIC_URL` (configuração do processo, definida na subida: `shared::public_url`). Trocar de CDN é trocar a variável.
- **Nome do objeto**: `logos/<comunidade>/<aleatório>.<ext>` e `covers/<comunidade>/<torneio>/<aleatório>.<ext>`, como os screenshots. Como o nome nunca muda de conteúdo, o envio grava `Cache-Control: public, max-age=31536000, immutable` (assinado na URL): CDN e navegador guardam por um ano e nada é invalidado; trocar a imagem é trocar o nome no banco.
- **Upload como o dos screenshots** (URL assinada, `If-None-Match: *`, registro que confere por HEAD), com o upload pendente em `image_uploads`. **Até 10 uploads pedidos e não registrados por pessoa por hora** (`images.too_many_uploads`; assumido): os registrados trocam a imagem anterior, que sai do bucket, então não acumulam.
- **Sem órfãos**: o banco põe na fila `image_deletions` todo objeto que deixa de ser usado (trocado, removido, de upload abandonado, da comunidade ou do torneio apagados, inclusive pela cascata), e a rotina diária esvazia a fila depois de a URL de upload vencer, como a dos screenshots.

## og:image

- As páginas da comunidade e do torneio são só para membros (`tenants.md`), e o robô que monta a prévia entra sem sessão: a `og:image` por comunidade ou torneio só existe no **convite**, que já é público.
- **A imagem inteira, sem moldura, e o texto no título e na descrição** (01/11, pedido do usuário): uma imagem montada com logo e texto ficava bonita no Discord, mas o WhatsApp corta a miniatura em quadrado e sumia quase tudo. No link de inscrição com capa, a capa (2:1, `summary_large_image`); senão o logo (1:1, `summary`, miniatura que o WhatsApp não corta). O título é o do convite ("Convite para entrar em X", "Inscrição em Y") e o `og:site_name` é Telefrag. A descrição é a do torneio, no link de inscrição (numa linha, até 300 caracteres); no de comunidade, nenhuma. Nada que muda com o tempo (membros, vagas): a rede guarda a prévia na primeira vez e não atualiza. Nem o nome da comunidade, que o título do convite já traz ou que pouco diz no de inscrição. Sem logo nem capa, a imagem do site. A URL da imagem é a do CDN, que muda a cada troca, então a prévia nova não esbarra no cache do Discord. Uma rota pública só para a prévia do torneio ficou de fora: os slugs são previsíveis, e daria para descobrir nome e capa de torneios de comunidades fechadas.
