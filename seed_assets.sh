#!/bin/bash

# 1. Cargamos el .env de forma segura y estándar
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
    TOKEN=$ADMIN_SECRET_KEY # Asegúrate de que el nombre coincida con el de tu .env
fi

if [ -z "$TOKEN" ]; then
    echo "❌ ERROR: No se pudo cargar el token del .env"
    exit 1
fi

API_URL="http://localhost:3000/api/assets"

# 3. Diccionario de Assets a insertar (Formato -> "Nombre:Valor")
# Valores aproximados de mercado para tener un contexto realista
ASSETS=(
    "Bitcoin:65000.00"
    "Ethereum:3200.00"
    "Solana:145.50"
    "US Dollar:1.00"
    "Euro:1.08"
    "Libra Esterlina:1.26"
    "Oro (Onza):2350.00"
    "S&P 500 ETF:520.00"
)

echo "🌱 Iniciando el seeding de la base de datos..."
echo "------------------------------------------------"

# 4. Iteramos sobre el arreglo y hacemos un POST por cada uno
for ASSET in "${ASSETS[@]}"; do
    # Separamos el nombre del valor usando manipulación de strings de bash
    NAME="${ASSET%%:*}"
    VALUE="${ASSET##*:}"

    echo "Enviando -> $NAME ($VALUE)..."

    # Ejecutamos el request a tu API de Axum
    curl -s -X POST "$API_URL" \
         -H "Authorization: Bearer $TOKEN" \
         -H "Content-Type: application/json" \
         -d "{
               \"name\": \"$NAME\",
               \"unit_value\": $VALUE
             }" | jq . # Opcional: jq formatea la respuesta JSON de tu server para que se lea bien en terminal

    echo "" # Salto de línea visual
done

echo "------------------------------------------------"
echo "✅ Seed completado con éxito."