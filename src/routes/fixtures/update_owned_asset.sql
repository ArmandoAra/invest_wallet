
DELETE FROM owned_assets;
DELETE FROM assets;

-- Forzamos el ID 1 para el asset
INSERT INTO assets (id, name, unit_value) 
VALUES (1, 'Bitcoin', 50000.0);

-- Forzamos el ID de este historial/registro a 1 y usamos los IDs explícitos
INSERT INTO owned_assets (id, user_id, asset_id, quantity_owned, bought_for) 
VALUES (1, 1, 1, 0.05, 50000.0);