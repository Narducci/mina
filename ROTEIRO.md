# mina — Roteiro de Execução

## [x] 1. Criar o projeto

O wizard cria a pasta `mina` e toda a estrutura do projeto,
incluindo `src/`, `src-tauri/` e os arquivos de configuração.

```bash
cd ~/Projetos
pnpm create tauri-app mina
```

Respostas durante o wizard:

- Identifier: `com.orazio.mina`
- Language: `TypeScript / JavaScript`
- Package manager: `pnpm`
- UI template: `Vue`
- UI flavor: `JavaScript`

## [x] 2. Instalar dependências JavaScript

Instala os pacotes listados no `package.json` gerado pelo wizard.

```bash
cd ~/Projetos/mina
pnpm install
```

## [x] 3. Instalar o plugin SQL (lado JavaScript)

Adiciona o plugin SQL ao projeto JavaScript.

```bash
cd ~/Projetos/mina
pnpm add @tauri-apps/plugin-sql
```

## [x] 4. Instalar o plugin SQL (lado Rust)

Adiciona o plugin SQL ao projeto Rust.
A pasta `src-tauri/` foi criada no passo 1.

```bash
cd ~/Projetos/mina/src-tauri
cargo add tauri-plugin-sql --features sqlite
```

## [x] 5. Registrar o plugin no Rust

Abrir `~/Projetos/mina/src-tauri/src/lib.rs`

Adicionar a linha abaixo, imediatamente antes dela:

.plugin(tauri_plugin_sql::Builder::default().build())

Imediatamente antes dessa
.run(tauri::generate_context!())

## [x] 6. Configurar permissions

Abrir `~/Projetos/mina/src-tauri/capabilities/default.json`

Adicionar as linhas abaixo dentro do array `"permissions"`:

```json
"sql:default",
"sql:allow-execute",
"sql:allow-load",
"sql:allow-select"
```

## [x] 7. Verificar o ambiente

Se o app abrir sem erros, o ambiente está pronto.

```bash
cd ~/Projetos/mina
pnpm tauri dev
```

---

## [ ] 8. Schema do banco

A definir. Tabelas: `categoria`, `pessoa`, `banco`, `periodo`, `lancamento`, `comprovante`

## [ ] 9. Cadastros primários

- Categoria (CRUD)
- Pessoa (CRUD)
- Banco/Conta (CRUD)

## [ ] 10. Lançamentos

- Período (master)
- Lançamentos — Diretos e Indiretos
- Navegação e conciliação de período

## [ ] 11. Comprovantes

_(após especificação do processo)_

- Triagem
- Disponibilizar para lançamentos

## [ ] 12. Relatórios Judiciais

_(após especificação do formato)_

## [ ] 13. Configurações

- Pastas do sistema (comprovantes, avatares, backup)
- Tema claro/escuro
