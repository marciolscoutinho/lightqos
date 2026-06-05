# LightQOS — Project Page v2

Página estática para GitHub Pages do projeto **LightQOS — Quantum OS for the Age of Light**.

## Características

- Página do projeto, não portfólio pessoal;
- Português de Portugal e Inglês;
- seletor de idioma PT/EN;
- rato/cursor animado;
- tema claro/escuro;
- imagem visual do projeto com autor em contexto LightQOS;
- logo LightQOS no header, boot e hero;
- conteúdo técnico baseado no projeto zipado;
- arquitetura EFAL, EMF, TLM, HIO e The Light AI;
- módulos reais do repositório;
- exemplos e comandos rápidos.

## Publicar no GitHub Pages

```bash
git init
git add .
git commit -m "Add LightQOS project page"
git branch -M main
git remote add origin https://github.com/marciolscoutinho/lightqos-project-page.git
git push -u origin main
```

Depois:

```text
Settings → Pages → Deploy from branch → main → /root → Save
```


## Ajuste visual

Esta versão inclui correções de layout para evitar sobreposições e equilibrar o tamanho das imagens em desktop e mobile.


## Atualizações desta versão

- logo maior no canto superior esquerdo;
- remoção do fundo confuso por trás do título LightQOS;
- nova página `founder.html` com foto e breve biografia;
- menus reorganizados;
- secção de arquitetura transformada em cartões com imagens próprias.


## Atualização v4

- Substituída a imagem do founder na página principal por uma imagem conceptual relacionada com **Quantum Operative System**;
- Mantida a página separada `founder.html` com a breve biografia do criador;
- Estrutura geral da página preservada.


## Ajuste de menu

- Removido o prefixo `./` dos itens do menu para uma navegação mais limpa e profissional.


## Atualização V5

- Na página principal, as duas imagens foram combinadas numa única composição horizontal;
- O cursor foi alterado para usar o logótipo LightQOS;
- Foi adicionado rasto visual ao cursor;
- O cursor usa transparência para um efeito mais elegante.


## Atualização V5.1 — Correção de carregamento e cursor

- Reescrito o `script.js` de forma mais robusta.
- Corrigido o carregamento da página para evitar bloqueio no ecrã inicial.
- Adicionado fallback CSS para o ecrã de boot desaparecer mesmo se houver erro de JavaScript.
- Criado cursor pequeno otimizado com o logótipo LightQOS.
- Mantido o rasto do cursor com transparência.


## Atualização V5.2

- Cursor aumentado para um efeito mais visível.
- Logótipo superior esquerdo da página principal aumentado.
- Removido o logótipo do bloco de texto da página principal.
- Removida a presença da imagem do founder na página principal.
- Criada uma nova imagem horizontal limpa baseada apenas no visual Quantum Operative System.


## Atualização V5.3 — Correção do menu sticky

- Corrigido o problema em que os textos ficavam tapados/cortados pelo menu ao clicar nos links.
- Adicionado `scroll-padding-top` e `scroll-margin-top`.
- Adicionado ajuste JavaScript para scroll suave com compensação real da altura do header.


## Atualização V5.4 — Secções ajustadas ao ecrã

- Reduzida a altura visual das secções.
- Diminuído espaçamento vertical, tamanho dos cartões e altura da imagem principal.
- A arquitetura passa a usar uma grelha compacta.
- Os módulos usam grelha compacta para caberem melhor no ecrã.
- Mantido o logótipo superior esquerdo grande, mas com header mais compacto.
- Ajustado o offset do scroll para manter cada secção visível sem necessidade de reduzir o zoom.


## Atualização V5.5

- Aumentado o logótipo do canto superior esquerdo.
- Reduzido o tamanho do texto nas secções Founder, Architecture e Modules.
- Restaurado o cursor anterior (ponto + anel), removendo o cursor com logótipo.


## Atualização V5.6

- Aumentado novamente o logótipo no canto superior esquerdo.
- Gerada uma nova imagem principal panorâmica para caber melhor no layout horizontal.
- Ajustado o hero principal para encaixar melhor a imagem sem cortes nem excesso de altura.


## Atualização V5.7

- Substituída a imagem principal da página inicial pela nova imagem horizontal fornecida.
- Gerado novo pacote da página com o hero principal atualizado.


## Atualização V5.8 — Imagens da arquitetura

- Substituídas as imagens da secção Architecture pelas imagens finais fornecidas.
- Mapeamento:
  - Python SDK → `python_sdk.png`
  - The Light AI → `the_light_ai.png`
  - EFAL · EMF · TLM · HIO → `quantum_layers.png`
  - Rust Kernel · PyO3 · Math → `rustkernel_pyo3_math.png`
  - IBM · IonQ · Qblox · Zurich · Simulator → `connector_drivers.png`
- Ajustada a grelha da arquitetura para valorizar imagens horizontais.


## Atualização V5.9 — Architecture compacta

- Mantidas as novas imagens fornecidas.
- Restaurado o formato compacto anterior da secção Architecture.
- Os 5 blocos aparecem alinhados em grelha horizontal no desktop.
- O primeiro cartão deixa de ocupar duas colunas.
- Texto e imagens foram reduzidos para caberem melhor no ecrã sem necessidade de reduzir o zoom.


## Atualização V5.10 — Architecture com imagens maiores

- O texto da secção Architecture foi subido e compactado.
- As imagens dos cartões foram aumentadas.
- Mantido o layout de 5 cartões na mesma página em desktop.
- Texto dos cartões reduzido para libertar espaço visual para as imagens.


## Atualização V5.11 — Architecture com imagens ainda maiores

- A secção Architecture foi reorganizada em disposição 3 + 2 no desktop.
- As imagens ficaram maiores do que na versão anterior.
- O texto da secção e dos cartões foi compactado/subido para libertar espaço.
- Tudo continua enquadrado na mesma página, sem necessidade de reduzir o zoom.


## Atualização V5.12 — Architecture com imagens simplificadas

- As imagens da secção Architecture foram redesenhadas para ficarem mais simples e legíveis em tamanho menor.
- Mantido o mesmo significado de cada bloco:
  - Python SDK
  - The Light AI
  - Quantum Layers
  - Rust Kernel · PyO3 · Math
  - IBM · IonQ · Qblox · Zurich · Simulator
- O layout mantém tudo enquadrado na mesma página sem ser necessário reduzir o zoom.


## Atualização V5.13 — Architecture com novas imagens substituídas

Foram substituídas novamente as 5 imagens da secção **Architecture** pelas novas versões fornecidas:

- Python SDK
- The Light AI
- Quantum Layers
- Rust Kernel · PyO3 · Math
- IBM · IonQ · Qblox · Zurich · Simulator

O layout da versão v5.12 foi mantido, alterando apenas os visuais dos cartões.


## Atualização V5.14

- Logótipo da página principal aumentado novamente.
- Texto dos menus aumentado.
- Espaçamento entre itens do menu aumentado.
- Na secção Architecture, os 5 cartões passaram a usar imagens com o mesmo tamanho visual.
- Mantido o enquadramento da secção na mesma página sem necessidade de reduzir o zoom.


## Atualização V5.15

- Logótipo da página principal aumentado novamente.
- Menu superior deslocado mais para a esquerda.
- Ajustado o espaçamento e largura dos itens do menu para evitar que “Install” desça de linha.
- Ligeira compactação dos controlos da direita para libertar espaço horizontal.


## Atualização V5.16

- Logótipo da página principal aumentado de forma significativa.
- Menu superior reajustado para manter todos os itens na mesma linha, incluindo Examples e Install.
- Imagem principal ajustada para ficar com altura equivalente ao painel LightQOS ao lado.


## Atualização V5.17

- Ajustadas as imagens da página principal para serem vistas na íntegra sem necessidade de reduzir o zoom.
- Ajustadas as imagens da secção Architecture para mostrarem o visual completo, sem cortes.
- Mantido o enquadramento da página e a consistência visual dos cartões.


## Atualização V5.18

- A secção Architecture passou a funcionar em scroll horizontal.
- As imagens dos cartões foram aumentadas.
- Os textos correspondentes também foram aumentados.
- Cada cartão ficou maior e mais legível.
- Adicionada barra de scroll horizontal estilizada.


## Atualização V5.19

- Na secção Architecture, o scroll horizontal agora também pode ser feito arrastando com o rato.
- Adicionado cursor visual de `grab` / `grabbing`.
- Mantido o scroll horizontal com barra e compatibilidade com rato.


## Atualização V5.20

- Logótipo da página principal aumentado fortemente.
- Texto do menu aumentado.
- Substituída a imagem "The Light AI" na secção Architecture pela nova imagem fornecida.


## Atualização V5.21

- Na página principal, o hero foi unificado visualmente num só bloco, juntando texto e imagem no mesmo painel.
- O logótipo principal foi aumentado novamente de forma muito significativa.
- Mantida a imagem principal dentro do mesmo bloco do texto.
- Atualizada novamente a imagem "The Light AI" na secção Architecture.


## Atualização V5.22

- O menu foi aproximado do topo para não ficar demasiado descido com o logótipo grande.
- O logótipo mantém-se muito grande, mas com altura mais controlada.
- Ajustados espaçamentos do header para um aspeto mais equilibrado.


## Atualização V5.23

- O menu foi colocado ao nível do logótipo, na mesma linha horizontal do header.
- O header passou a usar uma grelha de 3 colunas: logótipo, menu e ações.
- Ajustado o espaçamento para evitar que o menu desça para uma linha inferior em desktop.


## Atualização V5.24

- Todas as páginas foram ajustadas para aparecerem ligeiramente mais acima.
- O retângulo/área dos botões do menu foi reduzido para caber melhor no ecrã.
- O header ficou mais compacto.
- As secções ficaram com menos espaço vertical para ajudar a caber tudo no ecrã.


## Atualização V5.25

- O texto da página principal foi reduzido para uma versão mais curta.
- O texto do menu foi aumentado ligeiramente.


## Atualização V5.26

- Removido o texto descritivo da secção Architecture:
  - PT: "O projeto está organizado como uma plataforma técnica: SDK, IA, kernel Rust, camadas quânticas, drivers e simulação."
- Removido também o equivalente em inglês para manter consistência visual.


## Atualização V5.27

- O texto "Arquitetura por camadas" na página Architecture foi reduzido.


## Atualização V5.28

- O texto "Arquitetura por camadas" foi reduzido ainda mais.
- A área acima dos cartões da secção Architecture foi ligeiramente compactada para ajudar a caber tudo no ecrã.


## Atualização V5.29

- Na secção Architecture, o bloco "Architecture" foi subido ligeiramente.
- O texto "Layered architecture" também foi subido.
- Foram reduzidos pequenos espaçamentos acima dos cartões para ajudar a ver tudo completo sem cortes.


## Atualização V5.30

- O texto "Layered architecture" foi reduzido ligeiramente.
- Adicionadas setas laterais na secção Architecture para indicar e controlar o scroll horizontal das imagens.
- As setas permitem deslocar visualmente os cartões para a esquerda e para a direita.


## Atualização V5.31

- Reduzido o tamanho do texto "LightQOS" na página principal.


## Atualização V5.32

- Na página principal, o texto "// página oficial do projeto" foi alterado para "// página oficial".
- Foi removido o texto "LightQOS" da página principal.


## Atualização V5.33

- O texto "LightQOS" foi restaurado na página principal.
- O título foi colocado com tamanho mais pequeno.
- As margens/padding do retângulo horizontal principal foram reduzidas.


## Atualização V5.34

- Reduzidas quase ao mínimo as margens/padding do retângulo principal da página inicial.
- O bloco com imagem e texto ficou mais próximo do tamanho útil do conteúdo.
- Mantida apenas uma pequena margem visual para não colar totalmente às bordas.


## Atualização V5.35

- Na página principal, a imagem principal ficou centrada ao meio.
- Os textos principais foram reorganizados para aparecerem por cima e por baixo da imagem.
- A imagem central foi mantida ligeiramente maior e centrada.


## Atualização V5.36

- Corrigida a sobreposição da imagem com o texto superior na página principal.
- O hero foi reestruturado: texto superior, imagem central e texto inferior em blocos separados.
- A altura máxima da imagem foi limitada para a página caber melhor no ecrã sem reduzir o zoom.


## Atualização V5.37

- A imagem central da página principal foi aumentada ligeiramente.
- O texto foi ajustado para acompanhar o novo tamanho da imagem sem voltar a provocar sobreposição.
- Mantido o objetivo de caber melhor no ecrã sem necessidade de reduzir o zoom.


## Atualização V5.38

- Removido da página principal o texto descritivo inferior.
- Removidas as etiquetas: Quantum OS, Rust Kernel, Python SDK e The Light AI.
- Removidos os botões: Explorar arquitetura e Ver instalação.
- O hero foi reajustado para não deixar espaços vazios e para valorizar a imagem central.


## Atualização V5.39

- Substituída a imagem principal da página inicial por uma nova imagem central que melhor descreve o LightQOS.
- Mantida a restante estrutura do website.


## Atualização V5.40

- Aumentada a imagem principal da página inicial, mantendo o enquadramento sem necessidade de reduzir o zoom.
- Removido o texto “LightQOS” da página principal.
- Colocado “// página oficial” no canto superior esquerdo do bloco principal.


## Atualização V5.41

- Aumentado o tamanho da imagem principal da página inicial.
- Aumentado o texto “// official page” / “// página oficial”.
- Mantido o enquadramento para continuar a caber bem no ecrã.


## Atualização V5.42

- Aumentado novamente o tamanho da imagem principal da página inicial.
- Aumentado novamente o texto “// official page” / “// página oficial”.
- Mantido o enquadramento para continuar a caber no ecrã sem precisar reduzir demasiado o zoom.


## Atualização V5.43

- A imagem principal da página inicial foi aumentada ligeiramente.
- Mantido o restante layout e o texto “// official page”.


## Atualização V5.44

- Aumentada ligeiramente a imagem principal da página inicial.
- Mantido o restante layout da página.
