# Imagens públicas: logo e capa

O logo da comunidade e a capa do torneio (01/11). Decidido pelo usuário, salvo o "assumido".

## Forma e onde aparecem

- **Logo quadrado (1:1), capa 2:1.** O logo é identidade e aparece pequeno, ao lado do nome. A capa é 2:1 porque é a proporção do `HeroImage` do `PageHeader` do Carbon no `lg` (3:2 abaixo), está no conjunto de proporções do 2x Grid, e a `og:image` (1,91:1) sai dela quase sem corte.
- **Logo**: antes do título no cabeçalho da comunidade (o lugar do ícone do título no `PageHeader` do Carbon), nos tiles de "Suas comunidades", nas linhas de "Esperando você" e no título do convite. Fica fora do painel de comunidades do cabeçalho, porque o `Switcher` do Carbon é só texto (assumido).
- **Capa**: à direita do cabeçalho do torneio, só do `lg` para cima (no celular empurraria a ação primária e "Sua situação" para baixo); no topo dos tiles de "Seus torneios" e de "Inscrições abertas" e no tile do link de inscrição. A aba Torneios da comunidade (`DataTable`) fica sem imagem: numa linha de 48 px a capa teria 96×48.
- **Sem imagem**: numa grade com alguma capa ou logo, todo tile leva o espaço (sem capa, o logo da comunidade ou a inicial no centro), para as linhas não desalinharem; sem nenhuma, nenhum leva. Fora da grade, nada no lugar (assumido).
- `alt=""` em todas: o nome já está em texto ao lado.

## Quem troca e como

- **Logo: só o owner**, em Configurações (como o rating e o Discord). **Capa: organizer e owner**, na seção Geral da aba Configurações do torneio, em qualquer estado do torneio.
- **Na edição, salva sozinho**, fora do "Salvar" do formulário: o envio acontece ao escolher o arquivo (como o file uploader do Carbon), e no formulário ficaria o estado "imagem enviada, formulário não salvo".
- **Só depois de criar** (06/11, do usuário; de 01/11 até aqui, a criação também tinha a imagem, guardada no navegador até o dono existir): o Novo torneio perdeu a capa ao virar o Modal curto, e o Criar comunidade, o logo ao virar Modal também. Os dois ficam nas Configurações, que salvam sozinhas. Saiu com isso o caso "criado, mas a imagem não chegou", que o Modal teria de mostrar no lugar do formulário.
- **Sem recorte manual** (o Carbon não tem cropper): o navegador corta pelo centro e reduz para **256×256** (logo) e **1200×600** (capa), em **JPEG**, até **1 MB**. Só JPEG (a API recusa WebP): o `ImageResponse` da prévia dos links não desenha WebP. Sem transparência, o logo sai sobre branco, o que também evita um logo escuro transparente sumir no tema g100.

## Foto de perfil

A foto da conta (06/11), na aba Perfil. Assumido, salvo o que o usuário pediu (recorte, zoom e giro antes do envio; nada no bucket sem dono; a foto só troca se o envio deu certo).

- **Vale mais que o avatar do Discord**, que continua gravado (`users.avatar_url`, atualizado a cada login) e volta quando a foto sai. A API devolve um `avatar_url` só (a foto ou o do Discord) em toda resposta que mostra pessoas, e `/me` também o `photo_url` (só a foto). Cada query lê `coalesce(u.avatar_key, u.avatar_url)` e passa por `auth::resolve_avatar`: o banco garante `avatars/...` num e `https://...` no outro.
- **Editor no navegador, num Modal** (o Carbon não tem cropper): a área quadrada com o círculo que o site mostra, arrastar, pinçar, roda do mouse ou setas para mover, o `Slider` de zoom (100% a 400%) e dois `IconButton` de girar em quartos de volta; a foto lida com a orientação do EXIF. O recorte sempre cobre o quadrado. Sai **256×256, JPEG**, sobre branco, como o logo.
- **Nada sai antes de "Salvar foto"**: cancelar não chama a API. Salvar segue o caminho do logo (URL assinada, PUT com `If-None-Match: *`, registro conferido por HEAD): a foto de antes só é trocada no registro, e o erro fica no Modal com o recorte. O upload pedido e não registrado (PUT que falhou, aba fechada) sai na rotina diária, como os outros. **O limite de 10 uploads por hora é um só** para logo, capa e foto.
- **Objeto `avatars/<aleatório>.jpg`, sem pasta**: a pessoa não tem slug, e o id interno não vai para a URL. O upload pendente da foto fica em `image_uploads` sem comunidade, com `user_id`. A conta excluída perde a foto e os uploads pendentes (para a fila do bucket).
- **Remover sem confirmação**, como o logo: enviar de novo desfaz.

## Bucket e URL

- **Bucket público à parte, com o Block Public Access ligado**, lido só pelo CloudFront (OAC). Um bucket só, com o CloudFront liberado por prefixo, deixaria os screenshots separados do mundo por uma linha da política; à parte, o privado nem tem o CloudFront na política. Domínio próprio no CDN (`img.telefrag.com.br`; o de staging fica em `img-staging.telefrag.com.br`, porque `staging.` é CNAME da Vercel, cujo CAA não libera a Amazon), para as prévias já postadas sobreviverem a uma troca de provedor.
- **O banco guarda só o nome do objeto**; a API monta a URL com `S3_PUBLIC_URL` (configuração do processo, definida na subida: `shared::public_url`). Trocar de CDN é trocar a variável.
- **Nome do objeto**: `logos/<comunidade>/<aleatório>.<ext>` e `covers/<comunidade>/<torneio>/<aleatório>.<ext>`, como os screenshots. As pastas são os slugs de quando foi enviado: o endereço pode mudar (`tenants.md`), o objeto não se move, e o banco confere a forma, não o slug de agora. Como o nome nunca muda de conteúdo, o envio grava `Cache-Control: public, max-age=31536000, immutable` (assinado na URL): CDN e navegador guardam por um ano e nada é invalidado; trocar a imagem é trocar o nome no banco.
- **Upload como o dos screenshots** (URL assinada, `If-None-Match: *`, registro que confere por HEAD), com o upload pendente em `image_uploads`. **Até 10 uploads pedidos e não registrados por pessoa por hora** (`images.too_many_uploads`; assumido): os registrados trocam a imagem anterior, que sai do bucket, então não acumulam.
- **Sem órfãos**: o banco põe na fila `image_deletions` todo objeto que deixa de ser usado (trocado, removido, de upload abandonado, da comunidade ou do torneio apagados, inclusive pela cascata), e a rotina diária esvazia a fila depois de a URL de upload vencer, como a dos screenshots.

## og:image

- **Só o convite tem prévia própria** (do usuário, confirmado em 04/11): as páginas da comunidade e do torneio são só para membros (`tenants.md`), e o robô que monta a prévia entra sem sessão. Uma rota pública só para a prévia do endereço do torneio ficou de fora: os slugs são previsíveis, e daria para descobrir nome e capa de torneios de comunidades fechadas. O endereço do torneio mostra a prévia do site.
- **A imagem inteira, sem moldura, e o texto nas tags** (do usuário, 01/11 e 04/11): o texto dentro da imagem repete o que toda rede já mostra em volta dela e fica velho no cache da rede, e um PNG com foto pesa bem mais que o JPEG da capa. Só o WhatsApp Web corta a miniatura em quadrado; o WhatsApp do celular mostra inteira, então esse corte não pesa na escolha. No link de inscrição com capa, a capa (2:1, `summary_large_image`); senão o logo (1:1, `summary`, porque 256×256 não aguenta o tamanho grande). Sem logo nem capa, a imagem do site. A URL da imagem é a do CDN, que muda a cada troca, então a prévia nova não esbarra no cache do Discord.
- **Os textos**: o título é o do convite ("Convite para entrar em X", "Inscrição em Y") e o `og:site_name` é Telefrag. No link de inscrição, a descrição é "Comunidade · Modo." e a descrição do torneio, ou as regras, ou a descrição da comunidade, numa linha, até 300 caracteres; no de comunidade, a descrição dela. Nada que muda com o tempo (membros, vagas) nem a data (pode mudar, e o texto não tem o fuso de quem lê): a rede guarda a prévia na primeira vez e não atualiza.
- **O idioma** (assumido): o link vai sem idioma, e um robô sem `Accept-Language` cairia no padrão do site. Nesse caso, o link de inscrição sai no idioma do torneio; o da comunidade, que não grava idioma, continua no padrão. Um endereço com `/es` fica em espanhol.
