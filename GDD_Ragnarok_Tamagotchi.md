# Game Design Document
## Ragnarok Monster Tamer — ESP32-C6 Tamagotchi RPG

---

# 1. VUE D'ENSEMBLE

## 1.1 Concept

Un jeu de collection et d'élevage de monstres inspiré de Ragnarok Online, jouable sur un petit device ESP32-C6 avec écran tactile 1.8". Le joueur capture des monstres, les entraîne via des expéditions passives, et les fait combattre dans des donjons en temps réel.

## 1.2 Pilliers de Design

| Pillier | Description |
|---------|-------------|
| **Progression** | Le joueur voit ses monstres devenir plus forts |
| **Engagement régulier** | Check-ins de 2-5 minutes, plusieurs fois par jour |
| **Stratégie accessible** | Choix tactiques simples mais impactants |
| **Collection** | Capturer tous les monstres de chaque zone |

## 1.3 Plateforme Cible

| Spec | Valeur |
|------|--------|
| Hardware | ESP32-C6 |
| Écran | 1.8" tactile (160x128 ou 128x160 pixels) |
| Input | Tactile uniquement (pas de boutons physiques) |
| Sprites | Assets Ragnarok Online (sprite sheets) |

## 1.4 Session Type

| Moment | Activité | Durée |
|--------|----------|-------|
| Matin | Check expéditions, lancer nouvelles | 2 min |
| Midi | Récupérer expéditions, 2-3 combats donjon | 5 min |
| Après-midi | Expéditions courtes, amélioration monstres | 3 min |
| Soir | Session donjon plus longue (5-10 combats) | 10 min |

---

# 2. SYSTÈMES DE JEU

## 2.1 Monstres

### 2.1.1 Structure d'un Monstre

```
Monster {
    id: string                  // Identifiant unique
    species_id: string          // Type de monstre (ex: "wolf", "familiar")
    name: string                // Nom affiché
    level: u8                   // Niveau 1-99
    xp: u32                     // XP actuelle
    xp_to_next: u32             // XP pour niveau suivant
    element: Element            // Élément du monstre
    fusion_count: u8            // Nombre de fusions (+0 à +9)
    
    // Stats
    hp_current: u16
    hp_max: u16
    atk: u16
    def: u16
    spd: u16
    
    // Skill
    skill: Skill
    
    // État
    status: MonsterStatus       // Available, InExpedition, InDungeon
}
```

### 2.1.2 Éléments

7 éléments basés sur Ragnarok Online :

| Élément | Icône | Fort contre (x1.5) | Faible contre (x0.5) |
|---------|-------|-------------------|---------------------|
| Feu | 🔥 | Terre, Vent | Eau |
| Eau | 💧 | Feu | Terre, Vent |
| Terre | 🌿 | Eau | Feu |
| Vent | 💨 | Terre | Feu |
| Foudre | ⚡ | Eau | Terre |
| Shadow | 🌑 | Holy, Ghost | Holy |
| Holy | ✨ | Shadow | Ghost |
| Ghost | 👻 | Neutre sauf vs Holy | Holy |

### 2.1.3 Limite et Gestion

| Règle | Valeur |
|-------|--------|
| Monstres max possédés | 6 |
| Monstres en équipe donjon | 3 |
| Monstres par expédition | 1-3 |

### 2.1.4 Système de Fusion (Doublons)

Quand le joueur capture un monstre déjà possédé :
- Le monstre existant gagne **+1 Fusion**
- Chaque fusion donne **+5% stats de base**
- Maximum : +9 fusions (+45% stats)

```
Formule stats:
stat_finale = stat_base * (1 + (fusion_count * 0.05))
```

### 2.1.5 Système de Niveau et XP

```
XP pour niveau suivant = niveau_actuel * 100

Exemple:
- Niveau 1 → 2 : 100 XP
- Niveau 10 → 11 : 1000 XP
- Niveau 50 → 51 : 5000 XP
```

### 2.1.6 Skills

Chaque espèce de monstre a **un skill unique** (pas de liste, pas de choix).

| Skill | Élément | Effet |
|-------|---------|-------|
| Meteor | 🔥 | Gros dégâts + applique Burn (DoT) |
| Tidal Wave | 💧 | Dégâts + applique Wet |
| Poison Spore | 🌿 | DoT poison + applique Nature |
| Thunder Bolt | ⚡ | Dégâts instantanés + applique Shock |
| Gust | 💨 | Dégâts + Swirl (propage éléments) |
| Soul Strike | 🌑 | Dégâts ignorant 30% DEF |
| Heal | ✨ | Soigne le monstre actif |

---

## 2.2 Expéditions (Mode Passif)

### 2.2.1 Concept

Le joueur envoie une équipe de 1-3 monstres explorer une map pendant un temps réel. À la fin, il récupère XP, ressources, et une chance de capture.

### 2.2.2 Slots d'Expédition

- **2 slots** d'expédition en parallèle maximum
- Chaque slot peut contenir 1-3 monstres

### 2.2.3 Durées et Récompenses

| Durée | Ratio Récompense | XP Base | Ressources Base | Chance Capture |
|-------|------------------|---------|-----------------|----------------|
| 20 min | ★★★ (meilleur) | 50 | 15 | 15% |
| 1 heure | ★★☆ | 120 | 35 | 25% |
| 4 heures | ★☆☆ | 350 | 90 | 40% |
| 8 heures | ★☆☆ | 600 | 150 | 50% |

**Note :** Les durées courtes donnent un meilleur ratio (3x 20min > 1x 1h) pour récompenser les joueurs actifs.

### 2.2.4 Éléments Requis

Chaque map d'expédition requiert **au moins un monstre** de certains éléments dans l'équipe.

```
Exemple:
- Map "Forêt Ouest" requiert: 💧 ET 🌿
- Le joueur DOIT avoir au moins 1 monstre Eau ET 1 monstre Terre
- Une fois les requis satisfaits, il peut ajouter n'importe quel monstre
```

### 2.2.5 Structure d'une Expédition

```
Expedition {
    id: string
    map_id: string
    monsters: Vec<MonsterId>        // 1-3 monstres
    duration: Duration              // 20min, 1h, 4h, 8h
    started_at: Timestamp
    completed: bool
}
```

### 2.2.6 Résultats d'Expédition

```
ExpeditionResult {
    xp_per_monster: u32
    crystals: u16
    essences: Vec<(Element, u8)>    // Essences élémentaires
    captured_monster: Option<SpeciesId>
}
```

---

## 2.3 Donjons (Mode Combat Actif)

### 2.3.1 Concept

Mode de jeu actif où le joueur combat en temps réel avec swap de monstres et réactions élémentaires.

### 2.3.2 Structure d'un Donjon

```
Dungeon {
    id: string
    name: string
    zone_id: string
    floors: infinite              // Étages infinis
    checkpoints: Vec<u16>         // Ex: [5, 10, 15, 20, 25, 30...]
    dominant_elements: Vec<Element>
    enemy_pool: Vec<SpeciesId>
}
```

### 2.3.3 Checkpoints

- Checkpoint tous les **5 étages**
- Une fois atteint, le joueur peut **recommencer depuis ce checkpoint**
- Commencer plus haut = meilleures récompenses par étage

| Départ | Récompenses/étage |
|--------|-------------------|
| Étage 1 | ★☆☆ Base |
| Étage 10 | ★★☆ x1.5 |
| Étage 20 | ★★★ x2.0 |
| Étage 30+ | ★★★ x2.5 |

### 2.3.4 Progression et Mort

- **Mort** : La run s'arrête, le joueur **garde toutes les récompenses** obtenues
- **Abandon** : Le joueur peut quitter à tout moment et garder ses gains
- **Record** : L'étage max atteint est sauvegardé

### 2.3.5 Déblocage des Zones

Atteindre certains étages débloque de nouvelles zones :

| Condition | Débloque |
|-----------|----------|
| Culvert Étage 20 | Zone Payon |
| Payon Cave Étage 15 | Zone Geffen |
| Geffen Tower Étage 15 | Zone Morroc |

---

## 2.4 Système de Combat

### 2.4.1 Vue d'Ensemble

Combat **temps réel** inspiré de Genshin Impact :
- Attaques automatiques
- Le joueur contrôle les **swaps** et l'utilisation des **skills**
- Les réactions élémentaires sont le cœur de la stratégie

### 2.4.2 Jauges de Combat

Chaque monstre a 3 jauges :

| Jauge | Comportement |
|-------|-------------|
| **HP** | Points de vie, le monstre meurt à 0 |
| **ATK Bar** | Se remplit automatiquement (vitesse = SPD), déclenche une attaque à 100%, puis reset |
| **SKL Bar** | Se remplit à chaque attaque (+20%), skill disponible à 100% |

```
Vitesse ATK Bar:
- SPD 30 → ~1 attaque/seconde
- SPD 50 → ~1.5 attaque/seconde
- SPD 70 → ~2 attaques/seconde
```

### 2.4.3 Actions du Joueur

**3 boutons tactiles uniquement :**

| Bouton | Position | Action |
|--------|----------|--------|
| SWAP 1 | Gauche | Change vers le 2ème monstre |
| SWAP 2 | Centre | Change vers le 3ème monstre |
| SKILL | Droite | Utilise le skill du monstre actif |

### 2.4.4 Swap

- **Cooldown** : 3 secondes après un swap
- Le monstre swappé out **conserve ses jauges** (HP, SKL)
- Certains monstres ont des **talents de swap** (bonus à l'entrée)

| Talent de Swap | Effet |
|----------------|-------|
| Shield | Gagne un bouclier (absorbe 30 dégâts) |
| Quick | Première attaque instantanée |
| Regen | Soigne 10% HP en entrant |
| Dodge | Esquive la prochaine attaque |

### 2.4.5 Système d'Aura Élémentaire

Quand un monstre frappe, il **applique son élément** sur l'ennemi :

```
Durée de l'aura:
- Attaque auto : 2 secondes
- Skill : 4 secondes
```

L'aura est visible sur l'ennemi (icône de l'élément).

### 2.4.6 Réactions Élémentaires

Quand un élément **différent** touche un ennemi qui a déjà une aura :

| Aura | + Attaque | = Réaction | Effet |
|------|-----------|------------|-------|
| 💧 Wet | 🔥 Feu | **VAPORIZE** | Dégâts x2 |
| 💧 Wet | ⚡ Foudre | **ELECTROCUTE** | Dégâts + Stun 1 sec |
| 💧 Wet | 🌿 Terre | **BLOOM** | Heal équipe 15% |
| 🔥 Burn | 💧 Eau | **VAPORIZE** | Dégâts x2 |
| 🔥 Burn | 💨 Vent | **SWIRL FIRE** | Propage Burn à tous ennemis |
| 🔥 Burn | 🌿 Terre | **MELT** | Dégâts x1.5 |
| 🌿 Nature | 🔥 Feu | **BURNING** | Gros DoT 5 sec |
| ⚡ Shock | 💧 Eau | **SUPERCONDUCT** | DEF ennemi -30% pendant 5 sec |
| Tout | 💨 Vent | **SWIRL** | Propage l'aura aux autres ennemis |

### 2.4.7 Comportement Ennemi

L'ennemi a aussi ATK Bar et SKL Bar :
- Attaque automatiquement
- Un **indicateur** apparaît quand son skill est presque prêt (⚠️)
- Le joueur peut anticiper et swap vers un tank ou burst l'ennemi avant

### 2.4.8 Formules de Dégâts

```
Dégâts base = ATK_attaquant - (DEF_cible * 0.5)
Dégâts min = ATK_attaquant * 0.1

Multiplicateur élémentaire:
- Avantage : x1.5
- Neutre : x1.0
- Désavantage : x0.5

Dégâts finaux = Dégâts base * Multiplicateur * Bonus réaction
```

### 2.4.9 Déroulement d'un Combat

```
1. Affichage de l'ennemi (espèce, élément, HP)
2. Combat temps réel:
   - Barres ATK se remplissent
   - Attaques automatiques
   - Joueur tap SWAP ou SKILL quand opportun
3. Victoire quand HP ennemi = 0
4. Défaite si tous les monstres du joueur meurent
```

### 2.4.10 Entre les Étages

Entre chaque combat :
- Affichage des récompenses de l'étage
- État de l'équipe (HP restants)
- Aperçu du prochain étage (ennemis)
- Choix : **CONTINUER** ou **ABANDONNER**

Les HP ne sont **pas restaurés** entre les étages (sauf skills de heal).

---

## 2.5 Ressources

### 2.5.1 Types de Ressources

| Ressource | Icône | Obtention | Utilisation |
|-----------|-------|-----------|-------------|
| Cristaux | 💎 | Expéditions, Donjons | Améliorer stats |
| Essence Feu | 🔥 | Expéd. zones feu | Améliorations majeures |
| Essence Eau | 💧 | Expéd. zones eau | Améliorations majeures |
| Essence Terre | 🌿 | Expéd. zones terre | Améliorations majeures |
| Essence Vent | 💨 | Expéd. zones vent | Améliorations majeures |
| Essence Foudre | ⚡ | Expéd. zones foudre | Améliorations majeures |
| Essence Shadow | 🌑 | Expéd. zones shadow | Améliorations majeures |
| Essence Holy | ✨ | Expéd. zones holy | Améliorations majeures |

### 2.5.2 Amélioration des Monstres

Chaque stat peut être améliorée individuellement :

```
Coût amélioration = (stat_actuelle / 10) * 5 Cristaux

Exemple:
- ATK 50 → 51 : 25 Cristaux
- ATK 100 → 101 : 50 Cristaux
```

Les améliorations majeures (+10 stats d'un coup) nécessitent des **Essences** de l'élément du monstre.

---

## 2.6 Zones et Maps

### 2.6.1 Structure du Monde

```
Zone {
    id: string
    name: string
    maps: Vec<Map>              // Maps d'expédition
    dungeon: Dungeon            // Un donjon par zone
    unlock_condition: Option<UnlockCondition>
}

Map {
    id: string
    name: string
    zone_id: string
    level_range: (u8, u8)       // Ex: (5, 15)
    required_elements: Vec<Element>
    capturable_monsters: Vec<SpeciesId>
    resource_drops: Vec<(Resource, u8)>  // Type + quantité base
}
```

### 2.6.2 Zones du Jeu

| Zone | Maps | Donjon | Condition Déblocage |
|------|------|--------|---------------------|
| Prontera | 4 | Culvert | Début du jeu |
| Payon | 5 | Payon Cave | Culvert Ét.20 |
| Geffen | 4 | Geffen Tower | Payon Cave Ét.15 |
| Morroc | 5 | Pyramides | Geffen Tower Ét.15 |
| Aldebaran | 4 | Clock Tower | Pyramides Ét.15 |

### 2.6.3 Exemple : Zone Prontera

```
PRONTERA
├── Maps:
│   ├── Plaine Sud (Niv.1-5) - Requis: 🔥
│   ├── Forêt Ouest (Niv.5-10) - Requis: 💧🌿
│   ├── Égouts (Niv.8-12) - Requis: 🌑
│   └── Collines Nord (Niv.12-15) - Requis: ⚡💨
│
└── Donjon: Culvert
    ├── Éléments dominants: 💧🌑
    ├── Ennemis: Thief Bug, Familiar, Tarou, Plankton
    └── Boss (tous les 10 étages): Golden Thief Bug
```

---

# 3. INTERFACE UTILISATEUR

## 3.1 Navigation Globale

**100% tactile avec swipes :**

| Geste | Action |
|-------|--------|
| Swipe → | Retour (page précédente) |
| Swipe ↑↓ | Scroll dans les listes |
| Tap | Sélectionner / Confirmer |
| Long Press | Actions secondaires |

**Aucun bouton "Back" ou flèches de scroll.**

## 3.2 Arborescence des Écrans

```
🏠 ACCUEIL
├── 📍 CARTE DU MONDE
│   └── 📍 DÉTAIL ZONE
│       ├── 🗺️ EXPÉDITION MAP
│       │   └── 👥 SÉLECTION ÉQUIPE
│       └── ⚔️ DONJON
│           ├── 👥 SÉLECTION ÉQUIPE
│           ├── ⚔️ COMBAT
│           ├── 📊 ENTRE-ÉTAGES
│           └── 🏁 FIN DE RUN
│
├── 👹 MONSTRES
│   ├── 📋 DÉTAIL MONSTRE
│   │   └── 💪 AMÉLIORATION
│   └── 📖 COLLECTION
│
└── 📦 INVENTAIRE
```

---

## 3.3 Écrans Détaillés

### 3.3.1 🏠 ACCUEIL

```
╔═══════════════════════════════════════╗
║  ☀️ 14:32                     💎 1250 ║
╠═══════════════════════════════════════╣
║                                       ║
║  Expéditions:                         ║
║  1. 🗺️ Payon ████████░░ 23min        ║
║  2. 🗺️ Disponible                     ║
║                                       ║
║  Équipe active:                       ║
║  🔥 Niv.24   💧 Niv.22   🌿 Niv.20   ║
║                                       ║
╠═══════════════════════════════════════╣
║  [📍 Carte] [👹 Monstres] [📦 Invent.]║
╚═══════════════════════════════════════╝
```

**Interactions :**
| Élément | Tap | Résultat |
|---------|-----|----------|
| Expédition en cours | Tap | Voir détails / Récupérer |
| Expédition vide | Tap | → Carte |
| Monstre équipe | Tap | → Détail Monstre |
| Bouton Carte | Tap | → Carte du Monde |
| Bouton Monstres | Tap | → Liste Monstres |
| Bouton Inventaire | Tap | → Inventaire |

---

### 3.3.2 📍 CARTE DU MONDE

```
╔═══════════════════════════════════════╗
║  📍 CARTE                    swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  ▸ PRONTERA                           ║
║    🗺️ 4 maps (Niv.1-15)               ║
║    ⚔️ Culvert — Record: Ét.47         ║
║                                       ║
║  ▸ PAYON                              ║
║    🗺️ 5 maps (Niv.10-25)              ║
║    ⚔️ Payon Cave — Record: Ét.23      ║
║                                       ║
║  ▸ GEFFEN 🔒                          ║
║    Débloquer: Payon Cave Ét.15        ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

**Interactions :**
| Geste | Résultat |
|-------|----------|
| Tap zone | → Détail Zone |
| Swipe ↑↓ | Scroll zones |
| Swipe → | → Accueil |

---

### 3.3.3 📍 DÉTAIL ZONE

```
╔═══════════════════════════════════════╗
║  📍 PRONTERA                 swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  EXPÉDITIONS                          ║
║                                       ║
║  ▸ Plaine Sud                         ║
║    Niv.1-5  │  🔥 requis              ║
║                                       ║
║  ▸ Forêt Ouest                        ║
║    Niv.5-10  │  💧🌿 requis           ║
║                                       ║
║  ▸ Égouts                             ║
║    Niv.8-12  │  🌑 requis             ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  DONJON                               ║
║  ⚔️ Culvert — Record: Ét.47           ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

---

### 3.3.4 🗺️ EXPÉDITION MAP

```
╔═══════════════════════════════════════╗
║  🗺️ Forêt Ouest              swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  Niveau: 5-10                         ║
║  Requis: 💧 🌿                        ║
║                                       ║
║  Monstres:                            ║
║  🐺 Wolf  🦇 Familiar  🦊 Nine Tail   ║
║                                       ║
║  Ressources:                          ║
║  💎 Cristaux  🌿 Essence Terre        ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  Durée:                               ║
║  ▸[20min ★★★] [1h ★★☆] [4h ★☆☆]      ║
║                                       ║
║  Équipe: Non sélectionnée ⚠️          ║
║                                       ║
║  [ CHOISIR ÉQUIPE ]                   ║
╚═══════════════════════════════════════╝
```

---

### 3.3.5 👥 SÉLECTION ÉQUIPE

**État initial (requis non satisfaits) :**

```
╔═══════════════════════════════════════╗
║  👥 ÉQUIPE                   swipe →  ║
║  Forêt Ouest │ Requis: 💧 🌿          ║
╠═══════════════════════════════════════╣
║                                       ║
║  ÉQUIPE (0/3)              ⚠️ 💧🌿    ║
║  ┌─────┐ ┌─────┐ ┌─────┐              ║
║  │  +  │ │  +  │ │  +  │              ║
║  └─────┘ └─────┘ └─────┘              ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  DISPONIBLES                          ║
║                                       ║
║  🔥 Flame     Niv.24  ✗ élément       ║
║  💧 Marina    Niv.22  ✓               ║
║  🌿 Mandra    Niv.20  ✓               ║
║  👻 Whisper   Niv.15  ✗ en expéd.     ║
║  🐺 Wolf      Niv.18  ✗ élément       ║
║  ⚡ Deniro    Niv.12  ✗ élément       ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

**État après sélection (requis satisfaits) :**

```
╔═══════════════════════════════════════╗
║  👥 ÉQUIPE                   swipe →  ║
║  Forêt Ouest │ Requis: ✓ 💧🌿         ║
╠═══════════════════════════════════════╣
║                                       ║
║  ÉQUIPE (2/3)                         ║
║  ┌─────┐ ┌─────┐ ┌─────┐              ║
║  │ 💧  │ │ 🌿  │ │  +  │              ║
║  │Maria│ │Mandra│ │     │              ║
║  └─────┘ └─────┘ └─────┘              ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  DISPONIBLES                          ║
║                                       ║
║  🔥 Flame     Niv.24  ✓               ║
║  👻 Whisper   Niv.15  ✗ en expéd.     ║
║  🐺 Wolf      Niv.18  ✓               ║
║  ⚡ Deniro    Niv.12  ✓               ║
║                                       ║
║  [ CONFIRMER ]                        ║
╚═══════════════════════════════════════╝
```

**Logique :**
- Avant requis satisfaits : seuls les monstres du bon élément sont sélectionnables
- Après requis satisfaits : tous les monstres disponibles sont sélectionnables
- Monstres en expédition/donjon : jamais sélectionnables

**Interactions :**
| Geste | Résultat |
|-------|----------|
| Tap monstre dispo | Ajoute à l'équipe |
| Tap monstre équipe | Retire de l'équipe |
| Confirmer | Valide et retourne à la page précédente |
| Swipe → | Annule (ne sauve pas) |

---

### 3.3.6 🗺️ EXPÉDITION — Prête

```
╔═══════════════════════════════════════╗
║  🗺️ Forêt Ouest              swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  Niveau: 5-10                         ║
║  Requis: ✓ 💧🌿                       ║
║                                       ║
║  Monstres:                            ║
║  🐺 Wolf  🦇 Familiar  🦊 Nine Tail   ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  Durée: 20min ★★★                     ║
║                                       ║
║  Équipe:                              ║
║  💧 Marina  🌿 Mandra                 ║
║                        [modifier]     ║
║                                       ║
║  [ 🚀 LANCER ]                        ║
╚═══════════════════════════════════════╝
```

---

### 3.3.7 🗺️ RÉSULTAT EXPÉDITION

```
╔═══════════════════════════════════════╗
║  ✓ EXPÉDITION TERMINÉE       swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  Forêt Ouest — 20min                  ║
║                                       ║
║  Récompenses:                         ║
║  • +120 XP par monstre                ║
║  • +35 💎 Cristaux                    ║
║  • +12 🌿 Essence Terre               ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  CAPTURE: 🐺 Wolf !                   ║
║  (Déjà possédé → Fusion +1)           ║
║                                       ║
║  [ RÉCUPÉRER ]                        ║
╚═══════════════════════════════════════╝
```

---

### 3.3.8 ⚔️ DONJON — Sélection

```
╔═══════════════════════════════════════╗
║  ⚔️ CULVERT                  swipe →  ║
║  Record: Étage 47                     ║
╠═══════════════════════════════════════╣
║                                       ║
║  Commencer depuis:                    ║
║                                       ║
║  ▸ Étage 1    ★☆☆                     ║
║    Étage 10   ★★☆                     ║
║    Étage 20   ★★★                     ║
║    Étage 30   ★★★                     ║
║    Étage 40   ★★★                     ║
║    Étage 50   🔒                      ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  Équipe:                              ║
║  🔥 Flame  💧 Marina  🌿 Mandra       ║
║  Puissance: 2450   [modifier]         ║
║                                       ║
║  [ ⚔️ ENTRER ]                        ║
╚═══════════════════════════════════════╝
```

---

### 3.3.9 ⚔️ COMBAT (Temps Réel)

```
╔═══════════════════════════════════════╗
║  CULVERT Ét.12               +45 💎   ║
╠═══════════════════════════════════════╣
║                                       ║
║  🐸 Thief Bug 💧                      ║
║  HP ████████░░  SKL ██████░░ ⚠️       ║
║                                       ║
║       -15    💥 VAPORIZE -67          ║
║                                       ║
║  🔥 Flame [💧WET]                     ║
║  HP ██████░░░░  SKL ████████████ ✓    ║
║                                       ║
╠═══════════════════════════════════════╣
║   [🌿]       [💧]       [🔥 SKILL]    ║
║   OK         CD:2       METEOR        ║
╚═══════════════════════════════════════╝
```

**Éléments affichés :**
- Header : Donjon + Étage + Récompenses accumulées
- Ennemi : Sprite, Élément, HP bar, SKL bar (avec ⚠️ si presque prêt)
- Feedback : Dégâts, réactions
- Joueur : Monstre actif, Aura appliquée sur lui, HP bar, SKL bar
- Boutons : 2 swap + 1 skill

**Boutons :**
| Bouton | État | Affichage |
|--------|------|-----------|
| SWAP | Disponible | Icône monstre |
| SWAP | En cooldown | "CD:Xs" |
| SKILL | Disponible | Nom du skill |
| SKILL | Non prêt | Barre de progression |

---

### 3.3.10 📊 ENTRE-ÉTAGES

```
╔═══════════════════════════════════════╗
║  ÉTAGE 12 COMPLÉTÉ !                  ║
╠═══════════════════════════════════════╣
║                                       ║
║  +15 💎   Total: 60 💎                ║
║                                       ║
║  🔥 Flame   ██████░░░░  112/180       ║
║  💧 Marina  ████░░░░░░   78/156       ║
║  🌿 Mandra  █████████░  120/134       ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  ÉTAGE 13:                            ║
║  🐸🐸🦇 (3 ennemis)                   ║
║                                       ║
║  [ CONTINUER ]  [ ABANDONNER ]        ║
╚═══════════════════════════════════════╝
```

---

### 3.3.11 🏁 FIN DE RUN

**Victoire (abandon volontaire) :**

```
╔═══════════════════════════════════════╗
║  ✓ RUN TERMINÉE                       ║
╠═══════════════════════════════════════╣
║                                       ║
║  Étages complétés: 12                 ║
║                                       ║
║  Récompenses:                         ║
║  • 156 💎 Cristaux                    ║
║  • 340 XP par monstre                 ║
║                                       ║
║  [ QUITTER ]                          ║
╚═══════════════════════════════════════╝
```

**Défaite :**

```
╔═══════════════════════════════════════╗
║  💀 DÉFAITE — Étage 17                ║
╠═══════════════════════════════════════╣
║                                       ║
║  Récompenses obtenues:                ║
║  • 156 💎 Cristaux                    ║
║  • 340 XP par monstre                 ║
║                                       ║
║  🆕 Nouveau record: Étage 17 !        ║
║                                       ║
║  [ RÉESSAYER ]  [ QUITTER ]           ║
╚═══════════════════════════════════════╝
```

---

### 3.3.12 👹 LISTE MONSTRES

```
╔═══════════════════════════════════════╗
║  👹 MONSTRES                 swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  🔥 Flame+2    Niv.24  ⚔️ 850        ║
║     🟢 Disponible                     ║
║                                       ║
║  💧 Marina     Niv.22  ⚔️ 720        ║
║     🟡 Expédition (12min)             ║
║                                       ║
║  🌿 Mandra     Niv.20  ⚔️ 680        ║
║     🟢 Disponible                     ║
║                                       ║
║  👻 Whisper    Niv.15  ⚔️ 490        ║
║     🟢 Disponible                     ║
║                                       ║
║  🐺 Wolf+1     Niv.18  ⚔️ 580        ║
║     🟡 Expédition (12min)             ║
║                                       ║
║  ⚡ Deniro     Niv.12  ⚔️ 390        ║
║     🟢 Disponible                     ║
║                                       ║
║  ───────────────────────────────────  ║
║  [ 📖 Collection 24/50 ]              ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

**Statuts :**
| Couleur | Signification |
|---------|---------------|
| 🟢 | Disponible |
| 🟡 | En expédition (temps restant) |
| 🔴 | En donjon |

---

### 3.3.13 📋 DÉTAIL MONSTRE

```
╔═══════════════════════════════════════╗
║  🔥 FLAME+2                  swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║        [SPRITE]                       ║
║                                       ║
║  Niveau 24       XP ████████░░        ║
║  Élément: 🔥 Feu                      ║
║  Fusions: +2                          ║
║  Puissance: 850                       ║
║                                       ║
║  ATK  ████████████░░░░  67            ║
║  DEF  ████████░░░░░░░░  45            ║
║  SPD  ██████████░░░░░░  52            ║
║  HP   ██████████████░░  180           ║
║                                       ║
║  Skill: 🔥 METEOR                     ║
║  "Gros dégâts + Burn"                 ║
║                                       ║
║  [ 💪 AMÉLIORER ]                     ║
╚═══════════════════════════════════════╝
```

---

### 3.3.14 💪 AMÉLIORATION

```
╔═══════════════════════════════════════╗
║  💪 AMÉLIORER                swipe →  ║
║  🔥 Flame                             ║
╠═══════════════════════════════════════╣
║                                       ║
║  ATK  67 → 70    [+3]  💎 25          ║
║  DEF  45 → 48    [+3]  💎 20          ║
║  SPD  52 → 55    [+3]  💎 22          ║
║  HP  180 → 190   [+10] 💎 35          ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  Amélioration majeure:                ║
║  ATK +10         💎 80 + 🔥 x5        ║
║                                       ║
║  ───────────────────────────────────  ║
║                                       ║
║  Tes ressources:                      ║
║  💎 1250    🔥 45                     ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

---

### 3.3.15 📖 COLLECTION

```
╔═══════════════════════════════════════╗
║  📖 COLLECTION               swipe →  ║
║  24/50                                ║
╠═══════════════════════════════════════╣
║                                       ║
║  PRONTERA                       8/12  ║
║  ✓🐷 ✓🔥 ✓💧 ✓🌿 ✓🐸 ✓🦇 ✗?? ✗??    ║
║  ✗?? ✗?? ✗?? ✗??                      ║
║                                       ║
║  PAYON                          6/10  ║
║  ✓🐺 ✓👻 ✓⚡ ✓🦊 ✓🌸 ✓🐰 ✗?? ✗??    ║
║  ✗?? ✗??                              ║
║                                       ║
║  GEFFEN 🔒                      0/8   ║
║                                       ║
║  MORROC 🔒                      0/10  ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

---

### 3.3.16 📦 INVENTAIRE

```
╔═══════════════════════════════════════╗
║  📦 INVENTAIRE               swipe →  ║
╠═══════════════════════════════════════╣
║                                       ║
║  RESSOURCES                           ║
║  💎 Cristaux         1,250            ║
║                                       ║
║  ESSENCES                             ║
║  🔥 Feu              45               ║
║  💧 Eau              32               ║
║  🌿 Terre            67               ║
║  ⚡ Foudre           12               ║
║  💨 Vent             28               ║
║  🌑 Shadow            8               ║
║  ✨ Holy              3               ║
║  👻 Ghost             0               ║
║                                       ║
║               ↕️ scroll               ║
╚═══════════════════════════════════════╝
```

---

# 4. DONNÉES DE JEU

## 4.1 Liste des Monstres (Base)

| ID | Nom | Élément | ATK | DEF | SPD | HP | Skill | Zone |
|----|-----|---------|-----|-----|-----|-----|-------|------|
| poring | Poring | 💧 | 15 | 10 | 20 | 80 | Heal | Prontera |
| lunatic | Lunatic | 🌿 | 20 | 12 | 35 | 70 | Quick Hit | Prontera |
| fabre | Fabre | 🌿 | 18 | 15 | 18 | 90 | Poison Spore | Prontera |
| pupa | Pupa | 🌿 | 5 | 30 | 5 | 120 | Harden | Prontera |
| thief_bug | Thief Bug | 🌑 | 25 | 18 | 28 | 85 | Steal | Prontera |
| familiar | Familiar | 🌑 | 30 | 12 | 32 | 75 | Soul Strike | Prontera |
| wolf | Wolf | 🌿 | 35 | 20 | 30 | 100 | Fang | Payon |
| snake | Snake | 🌿 | 28 | 22 | 25 | 95 | Poison | Payon |
| nine_tail | Nine Tail | 👻 | 45 | 25 | 40 | 110 | Fire Bolt | Payon |
| whisper | Whisper | 👻 | 40 | 15 | 45 | 70 | Soul Strike | Payon |
| sohee | Sohee | 💧 | 35 | 28 | 30 | 120 | Heal | Payon |
| archer_skel | Archer Skeleton | 🌑 | 50 | 20 | 38 | 90 | Arrow Shot | Geffen |
| hunter_fly | Hunter Fly | 💨 | 42 | 18 | 48 | 80 | Gust | Geffen |
| marionette | Marionette | 👻 | 55 | 22 | 35 | 95 | Soul Strike | Geffen |
| isis | Isis | 🌑 | 60 | 35 | 32 | 130 | Dark Strike | Morroc |
| mummy | Mummy | 🌑 | 48 | 40 | 20 | 150 | Curse | Morroc |
| minorous | Minorous | 🔥 | 70 | 45 | 25 | 160 | Meteor | Morroc |

## 4.2 Données des Zones

### Prontera

**Maps :**
| Map | Niveau | Requis | Monstres | Ressources |
|-----|--------|--------|----------|------------|
| Plaine Sud | 1-5 | 🔥 | poring, lunatic, fabre | 💎, 🌿 |
| Forêt Ouest | 5-10 | 💧🌿 | lunatic, fabre, pupa | 💎, 🌿, 💧 |
| Égouts | 8-12 | 🌑 | thief_bug, familiar | 💎, 🌑 |
| Collines Nord | 10-15 | ⚡💨 | poring, lunatic | 💎, ⚡, 💨 |

**Donjon Culvert :**
| Étages | Ennemis | Boss (tous les 10) |
|--------|---------|-------------------|
| 1-10 | thief_bug, familiar, poring | Golden Thief Bug |
| 11-20 | thief_bug x2, familiar | Thief Bug Queen |
| 21-30 | thief_bug x2, familiar x2 | Giant Familiar |
| 31+ | Mix + scaling stats | Random boss |

### Payon

**Maps :**
| Map | Niveau | Requis | Monstres | Ressources |
|-----|--------|--------|----------|------------|
| Forêt | 10-15 | 🌿 | wolf, snake | 💎, 🌿 |
| Grotte | 12-18 | 🌑👻 | whisper, nine_tail | 💎, 🌑, 👻 |
| Temple | 15-22 | ✨💧 | sohee, whisper | 💎, ✨, 💧 |
| Sommet | 18-25 | 💨🔥 | nine_tail, wolf | 💎, 💨, 🔥 |

---

# 5. SPÉCIFICATIONS TECHNIQUES

## 5.1 Hardware

| Composant | Spec |
|-----------|------|
| MCU | ESP32-C6 |
| Écran | 1.8" TFT tactile (160x128 ou 128x160) |
| Input | Tactile capacitif |
| Stockage | Flash interne + potentiel SD card |

## 5.2 Contraintes Techniques

| Contrainte | Valeur | Impact |
|------------|--------|--------|
| RAM limitée | ~512KB | Limiter sprites chargés simultanément |
| CPU | Single core | Optimiser les calculs temps réel |
| Écran petit | 1.8" | UI minimaliste, gros éléments tactiles |
| Batterie | Limitée | Mode sleep, refresh rate adaptatif |

## 5.3 Structure des Données

### Save File

```
SaveData {
    // Progression
    zones_unlocked: Vec<ZoneId>
    dungeon_records: HashMap<DungeonId, u16>    // Étage max
    collection: HashSet<SpeciesId>              // Monstres vus/capturés
    
    // Monstres
    monsters: Vec<Monster>                       // Max 6
    
    // Ressources
    crystals: u32
    essences: HashMap<Element, u16>
    
    // État
    expeditions: [Option<Expedition>; 2]
    current_dungeon_run: Option<DungeonRun>
    
    // Meta
    play_time: Duration
    last_save: Timestamp
}
```

### État de Combat

```
CombatState {
    // Équipe joueur
    player_team: [Monster; 3]
    active_index: u8
    swap_cooldowns: [f32; 3]
    
    // Ennemi
    enemy: Enemy
    enemy_aura: Option<(Element, f32)>          // Élément + temps restant
    
    // Jauges
    player_atk_bar: f32
    player_skl_bar: f32
    enemy_atk_bar: f32
    enemy_skl_bar: f32
    
    // Run
    current_floor: u16
    rewards_accumulated: Rewards
}
```

## 5.4 Timing Combat

| Action | Durée |
|--------|-------|
| ATK bar fill (SPD 30) | ~1 seconde |
| ATK bar fill (SPD 50) | ~0.66 seconde |
| Swap cooldown | 3 secondes |
| Aura durée (auto) | 2 secondes |
| Aura durée (skill) | 4 secondes |
| Stun (Electrocute) | 1 seconde |
| DoT tick | 0.5 seconde |

## 5.5 Assets Requis

### Sprites Monstres (depuis RO)
- Format : PNG avec transparence
- Taille suggérée : 32x32 ou 48x48 pixels
- Animation : Idle (2-4 frames), Attack (2-4 frames)

### UI Elements
- Icônes éléments (7)
- Barres (HP, ATK, SKL)
- Boutons (Swap, Skill, Confirmer, etc.)
- Backgrounds (zones)

### Audio (optionnel)
- SFX : Hit, Skill, Swap, Victory, Defeat
- Pas de musique (batterie)

---

# 6. PRIORITÉS D'IMPLÉMENTATION

## Phase 1 : Core Loop

1. Structure de données monstres
2. Écran Accueil basique
3. Liste monstres + Détail
4. Combat temps réel basique (1v1, pas de swap)
5. Un donjon simple (10 étages)

## Phase 2 : Expéditions

1. Système de temps réel (timer)
2. Écran carte du monde
3. Lancement expédition
4. Résultats et récompenses
5. Système de capture

## Phase 3 : Combat Complet

1. Swap de monstres
2. Système d'aura
3. Réactions élémentaires
4. Skills
5. Cooldowns

## Phase 4 : Progression

1. Système d'XP et niveaux
2. Amélioration des stats
3. Fusion des doublons
4. Déblocage des zones
5. Checkpoints donjon

## Phase 5 : Polish

1. Animations sprites
2. Feedback visuel (dégâts, réactions)
3. Sons
4. Sauvegarde/chargement
5. Tutoriel

---

# 7. QUESTIONS OUVERTES

| Sujet | Status | Notes |
|-------|--------|-------|
| Tutoriel | À définir | Premiers étages guidés ? |
| Notifications | À définir | Alerter quand expédition finie ? |
| PvP | Futur | Arène asynchrone plus tard |
| Sauvegarde | À définir | Locale uniquement pour MVP |
| Monétisation | Aucune | Projet personnel |

---

# 8. APPENDICES

## A. Table des Réactions Élémentaires (Complète)

| Aura | + | = | Effet |
|------|---|---|-------|
| 💧 Wet | 🔥 | VAPORIZE | x2 dégâts |
| 💧 Wet | ⚡ | ELECTROCUTE | Dégâts + Stun 1s |
| 💧 Wet | 🌿 | BLOOM | Heal 15% équipe |
| 🔥 Burn | 💧 | VAPORIZE | x2 dégâts |
| 🔥 Burn | 💨 | SWIRL FIRE | Propage Burn |
| 🔥 Burn | 🌿 | MELT | x1.5 dégâts |
| 🌿 Nature | 🔥 | BURNING | DoT 5s |
| ⚡ Shock | 💧 | SUPERCONDUCT | DEF -30% 5s |
| Any | 💨 | SWIRL | Propage aura |
| 🌑 Shadow | ✨ | PURIFY | x2 dégâts |
| ✨ Holy | 🌑 | CORRUPT | x2 dégâts |

## B. Formules

### Dégâts
```
base = max(ATK - DEF*0.5, ATK*0.1)
final = base * element_mult * reaction_mult
```

### XP pour level up
```
xp_needed = level * 100
```

### Coût amélioration stat
```
cost = (current_stat / 10) * 5
```

### Puissance monstre (affichage)
```
power = ATK + DEF + SPD + (HP / 5)
```

---

**Fin du document.**
