-- Migration v4: remove UNIQUE(periodo_id, tipo, ordem) da tabela lancamento

PRAGMA foreign_keys = OFF;

CREATE TABLE lancamento_new (
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
    deletado_em     TEXT
);

INSERT INTO lancamento_new SELECT * FROM lancamento;

DROP TABLE lancamento;
ALTER TABLE lancamento_new RENAME TO lancamento;

PRAGMA foreign_keys = ON;
