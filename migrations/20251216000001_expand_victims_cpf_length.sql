-- Expand victims.cpf length to store masked CPF format (000.000.000-00)

ALTER TABLE victims
    ALTER COLUMN cpf TYPE VARCHAR(14);
