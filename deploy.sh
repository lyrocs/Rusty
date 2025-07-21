#!/bin/bash

# --- CONFIGURATION ---
# Modifiez ces variables avec vos informations.
PI_USER="lyrocs"
PI_HOST="rusty.local" # ou l'adresse IP de votre Pi
TARGET_TRIPLE="arm-unknown-linux-musleabi" # Changez si nécessaire

# Noms et chemins des fichiers
LOCAL_BINARY_PATH="./target/${TARGET_TRIPLE}/debug/poc"
REMOTE_BINARY_NAME="poc"
REMOTE_DEST_PATH="~/" # Doit finir par un "/"

LOCAL_DATA_PATH="./data/" # Votre dossier data local
REMOTE_DATA_PATH="~/data"  # Le dossier data sur le Pi

# --- DÉBUT DU SCRIPT ---

# Active la sortie en cas d'erreur pour arrêter le script si une commande échoue
set -e

echo " Cible : ${PI_USER}@${PI_HOST}"
echo "-------------------------------------"

# 1. Compilation du projet en mode release
echo " Compiling for ${TARGET_TRIPLE}..."
cross build
echo " Build successful."
echo "-------------------------------------"

# 2. Arrêt du processus distant (s'il est en cours)
# On utilise `pkill` qui trouve et tue le processus en une seule commande.
# Le `|| true` à la fin évite que le script ne s'arrête si le processus n'est pas trouvé.
echo " Stopping remote process '${REMOTE_BINARY_NAME}' (if running)..."
ssh ${PI_USER}@${PI_HOST} "sudo pkill -f ${REMOTE_BINARY_NAME}" || true
echo " Remote process stopped."
echo "-------------------------------------"

# 3. Envoi du nouveau binaire via SCP
# Le processus étant arrêté, le fichier n'est plus verrouillé.
echo " Copying new binary to ${REMOTE_DEST_PATH}..."
scp ${LOCAL_BINARY_PATH} ${PI_USER}@${PI_HOST}:${REMOTE_DEST_PATH}${REMOTE_BINARY_NAME}
echo " Binary copied."
echo "-------------------------------------"

# --- NOUVELLE ÉTAPE ---
# 4. Synchronisation du dossier data
echo " Synchronizing data directory..."
rsync -av --delete --exclude '.DS_Store' ${LOCAL_DATA_PATH} ${PI_USER}@${PI_HOST}:${REMOTE_DATA_PATH}
echo " Data directory synchronized."
echo "-------------------------------------"

# 5. Redémarrage du processus sur le Raspberry Pi
echo " Starting new process in the background..."
# `nohup` rend le processus insensible à la fermeture de la session SSH.
# `&` le lance en arrière-plan.
# `> app.log 2>&1` redirige la sortie vers un fichier de log au lieu du terminal.
# && sudo rm mon_rpg.redb
# ssh -f -n ${PI_USER}@${PI_HOST} "cd ${REMOTE_DEST_PATH} && sudo rm mon_rpg.redb && nohup sudo ./${REMOTE_BINARY_NAME} > app.log 2>&1 &"
ssh -f -n ${PI_USER}@${PI_HOST} "cd ${REMOTE_DEST_PATH}  && nohup sudo ./${REMOTE_BINARY_NAME} > app.log 2>&1 &"
echo "-------------------------------------"
echo " DEPLOYMENT COMPLETE! "