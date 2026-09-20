-- schema_v2.sql — MINA MVP v2
-- Alterações em relação à v1:
--   • conta.ativa: DEFAULT alterado de 1 para 0 (toda conta nasce inativa)
--   • lancamento: adicionado conta_id NOT NULL REFERENCES conta(id)

-- ─────────────────────────────────────────
-- CONTA (recriar com ativa DEFAULT 0)
-- ─────────────────────────────────────────
CREATE TABLE conta_v2 (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pessoa_id   INTEGER NOT NULL REFERENCES pessoa(id),
    banco       TEXT    NOT NULL,
    agencia     TEXT    NOT NULL,
    numero      TEXT    NOT NULL,
    tipo        TEXT    NOT NULL CHECK (tipo IN ('corrente','poupanca')),
    ativa       INTEGER NOT NULL DEFAULT 0,
    criado_em   TEXT    NOT NULL DEFAULT (datetime('now')),
    deletado_em TEXT,
    UNIQUE (pessoa_id, agencia, numero)
);

INSERT INTO conta_v2 (id, pessoa_id, banco, agencia, numero, tipo, ativa, criado_em, deletado_em)
    SELECT id, pessoa_id, banco, agencia, numero, tipo, ativa, criado_em, deletado_em FROM conta;

DROP TABLE conta;
ALTER TABLE conta_v2 RENAME TO conta;

-- ─────────────────────────────────────────
-- LANÇAMENTO (recriar com conta_id NOT NULL)
-- ─────────────────────────────────────────
DROP TABLE lancamento;

CREATE TABLE lancamento (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    periodo_id      INTEGER NOT NULL REFERENCES periodo(id),
    categoria_id    INTEGER NOT NULL REFERENCES categoria(id),
    conta_id        INTEGER NOT NULL REFERENCES conta(id),
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
