# 00 — Visão do Produto

## Problema

Autores independentes costumam dividir o processo de criação entre várias ferramentas: editor de texto, organizador de capítulos, software de diagramação, conversor EPUB, gerador de PDF e ferramentas separadas para capa/metadados. Isso cria fricção, exige conhecimento técnico e aumenta o risco de inconsistências entre manuscrito e versão publicada.

## Visão

BookMaker será uma **engine de produção editorial**. O usuário trabalha dentro de um projeto editorial único que acompanha o conteúdo desde a estrutura inicial até o arquivo final.

A V1 será especializada em livros. A arquitetura, porém, considera desde o início que um projeto pode futuramente conter documentos com modelos de layout diferentes.

## Público inicial

- autores independentes;
- escritores de ficção e não ficção;
- estudantes e pesquisadores que precisam gerar livros/obras estruturadas;
- pequenas editoras e revisores;
- roteiristas que posteriormente possam converter material para quadrinhos;
- criadores que querem controle editorial sem dominar InDesign.

## Proposta de valor

**Escrever, organizar, formatar e publicar um livro em um fluxo contínuo e compreensível.**

O BookMaker deve esconder decisões técnicas sempre que puder tomar uma decisão editorial segura por padrão e permitir customização quando ela realmente fizer diferença.

## O que o BookMaker não é

Na V1, BookMaker não é:

- processador de texto genérico como Word;
- software de desktop publishing irrestrito como InDesign;
- ferramenta de desenho;
- sistema de worldbuilding;
- plataforma social;
- serviço obrigatório em nuvem;
- gerador de conteúdo por IA;
- substituto para registro legal de direitos autorais.

## Pilares

### 1. Escrita
Editor rápido, previsível, semanticamente estruturado e confortável para manuscritos longos.

### 2. Organização
Partes, capítulos, cenas/seções, front matter, back matter e navegação em árvore.

### 3. Edição
Busca, revisão, comentários posteriormente, estatísticas, reorganização e preview.

### 4. Produção editorial
Temas, tipografia, margens, cabeçalhos, paginação, sumário, página de copyright e metadados.

### 5. Publicação
EPUB e PDF de alta qualidade gerados a partir do mesmo projeto.

### 6. Extensibilidade
O modelo de projeto deve aceitar no futuro documentos de canvas para páginas de HQ/mangá.

## North Star

Um novo usuário deve conseguir:

1. criar um projeto;
2. adicionar capítulos;
3. escrever conteúdo;
4. preencher os dados editoriais;
5. escolher um tema;
6. visualizar o livro;
7. gerar um EPUB/PDF válido;

sem precisar consultar documentação técnica externa.
