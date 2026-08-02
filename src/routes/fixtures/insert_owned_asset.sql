-- fixtures/base_data.sql

-- Limpieza preventiva (opcional, útil si no usas transacciones en los tests)
DELETE FROM owned_assets;
DELETE FROM assets;
DELETE FROM users;

-- 1. Insertamos el usuario de prueba
-- (Ajusta los nombres de las columnas 'username' y 'password' a como los tengas en tu BD real)
INSERT INTO users (id, username, password_hash) 
VALUES (1, 'testuser', 'un_hash_falso_cualquiera');

-- 2. Insertamos el activo global
INSERT INTO assets (id, name, unit_value) 
VALUES (1, 'Bitcoin', 50000.0);
