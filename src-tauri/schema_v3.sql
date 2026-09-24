-- Migration v3: insere categorias fixas dos indiretos
INSERT INTO categoria (nome, tipo, disponivel_em, ativa, comprovante_esperado)
VALUES ('Aporte Período', 'neutro', 'indireto', 1, 0);

INSERT INTO categoria (nome, tipo, disponivel_em, ativa, comprovante_esperado)
VALUES ('Aporte Realizado', 'neutro', 'indireto', 1, 0);
