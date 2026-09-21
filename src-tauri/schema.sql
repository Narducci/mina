-- schema.sql — MINA MVP

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ─────────────────────────────────────────

-- ─────────────────────────────────────────
-- CONFIGURAÇÃO
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS configuracao (
    id                          INTEGER PRIMARY KEY CHECK (id = 1),
    pasta_raiz_comprovantes     TEXT    NOT NULL DEFAULT '',
    pasta_raiz_avatares         TEXT    NOT NULL DEFAULT '',
    tema                        TEXT    NOT NULL DEFAULT 'claro',
    versao_schema               INTEGER NOT NULL DEFAULT 1
);

-- ─────────────────────────────────────────
-- PESSOA
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS pessoa (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    nome            TEXT    NOT NULL,
    cpf             TEXT    UNIQUE,
    tipo            TEXT    NOT NULL CHECK (tipo IN ('curador','curatelado','contribuinte')),
    data_nascimento TEXT,
    avatar_path     TEXT,
    criado_em       TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em     TEXT
);

-- ─────────────────────────────────────────
-- CONTA
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS conta (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pessoa_id   INTEGER NOT NULL REFERENCES pessoa(id),
    banco       TEXT    NOT NULL,
    agencia     TEXT    NOT NULL,
    numero      TEXT    NOT NULL,
    tipo        TEXT    NOT NULL CHECK (tipo IN ('corrente','poupanca')),
    ativa       INTEGER NOT NULL DEFAULT 1,
    criado_em   TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em TEXT,
    UNIQUE (pessoa_id, agencia, numero)
);

-- ─────────────────────────────────────────
-- CATEGORIA
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS categoria (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    nome                 TEXT    NOT NULL UNIQUE,
    tipo                 TEXT    NOT NULL CHECK (tipo IN ('entrada','saida','neutro')),
    disponivel_em        TEXT    NOT NULL CHECK (disponivel_em IN ('direto','indireto','ambos')),
    ativa                INTEGER NOT NULL DEFAULT 1,
    comprovante_esperado INTEGER NOT NULL DEFAULT 0,
    criado_em            TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em          TEXT
);

-- ─────────────────────────────────────────
-- COMPROVANTE
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS comprovante (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    hash_arquivo     TEXT    NOT NULL UNIQUE,
    caminho_relativo TEXT    NOT NULL,
    nome             TEXT    NOT NULL,
    descricao        TEXT,
    data_documento   TEXT,
    numero_documento TEXT,
    status           TEXT    NOT NULL DEFAULT 'inbox' CHECK (status IN ('inbox','disponivel')),
    criado_em        TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em      TEXT
);

-- ─────────────────────────────────────────
-- COMPROVANTE ↔ CATEGORIA (N:M — triagem)
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS comprovante_categoria (
    comprovante_id  INTEGER NOT NULL REFERENCES comprovante(id),
    categoria_id    INTEGER NOT NULL REFERENCES categoria(id),
    PRIMARY KEY (comprovante_id, categoria_id)
);

-- ─────────────────────────────────────────
-- PERÍODO
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS periodo (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT,
    mes                  INTEGER NOT NULL CHECK (mes BETWEEN 1 AND 12),
    ano                  INTEGER NOT NULL,
    status               TEXT    NOT NULL DEFAULT 'aberto' CHECK (status IN ('aberto','conciliado')),
    saldo_inicial        REAL    NOT NULL DEFAULT 0,
    comprovante_id       INTEGER          REFERENCES comprovante(id),
    desconciliado_em     TEXT,
    desconciliado_motivo TEXT,
    criado_em            TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em          TEXT,
    UNIQUE (mes, ano)
);

-- ─────────────────────────────────────────
-- LANÇAMENTO
-- ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS lancamento (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    periodo_id      INTEGER NOT NULL REFERENCES periodo(id),
    categoria_id    INTEGER NOT NULL REFERENCES categoria(id),
    pessoa_id       INTEGER          REFERENCES pessoa(id),
    comprovante_id  INTEGER          REFERENCES comprovante(id),
    data            TEXT    NOT NULL,
    descricao       TEXT,
    tipo            TEXT    NOT NULL CHECK (tipo IN ('direto','indireto')),
    valor           REAL    NOT NULL,
    forma_pagamento TEXT,
    ordem           INTEGER,
    criado_em       TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em     TEXT,
    UNIQUE (periodo_id, tipo, ordem)
);
