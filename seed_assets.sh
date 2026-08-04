#!/bin/bash

if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
    TOKEN=$ADMIN_SECRET_KEY 
fi

if [ -z "$TOKEN" ]; then
    echo "❌ ERROR: No se pudo cargar el token del .env"
    exit 1
fi

API_URL="http://localhost:3000/api/assets"

# Diccionario modificado -> "Nombre:CoinGecko_ID:Valor"
# Si el ID es 'null', el worker de Rust lo ignorará.
ASSETS=(
    # --- CRIPTOS (Vivas, soportadas y actualizadas por tu worker actual) ---
    "Bitcoin:bitcoin:65000.00"
    "Ethereum:ethereum:3200.00"
    "Solana:solana:145.50"
    "Cardano:cardano:0.45"
    "Polkadot:polkadot:7.20"
    "Chainlink:chainlink:14.00"
    "Tether (USD):tether:1.00"
    "Oro Tokenizado (PAX Gold):pax-gold:2350.00"
)

echo "🌱 Iniciando el seeding de la base de datos..."
echo "------------------------------------------------"

for ASSET in "${ASSETS[@]}"; do
    # Usamos IFS para separar los 3 valores de la cadena
    IFS=':' read -r NAME API_ID VALUE <<< "$ASSET"

    # Formateamos el api_id para el JSON (los null van sin comillas)
    if [ "$API_ID" == "null" ]; then
        API_ID_JSON="null"
    else
        API_ID_JSON="\"$API_ID\""
    fi

    echo "Enviando -> $NAME (API: $API_ID) a \$$VALUE..."

  curl -i -X POST "$API_URL" \
         -H "Authorization: Bearer $TOKEN" \
         -H "Content-Type: application/json" \
         -d "{
               \"name\": \"$NAME\",
               \"api_id\": $API_ID_JSON,
               \"unit_value\": $VALUE
             }"

    echo "" 
done

echo "------------------------------------------------"
echo "✅ Seed completado con éxito."