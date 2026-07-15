# 05 — MVP

Objectif : **une v0.1 utilisable au quotidien sur Linux en ~6 semaines**, qui valide l'architecture (bus, collecteurs, IPC batché, widgets) sans les fonctionnalités longues (remote, plugins, alertes).

## Inclus dans le MVP

### Backend
- Workspace complet avec `openscope-core` (types, traits, bus) — **l'architecture est en place dès le MVP**, même si un seul OS est couvert.
- Collecteurs **Linux** :
  - CPU : usage global + par cœur, fréquence, température (hwmon), consommation RAPL si dispo
  - Mémoire : utilisée / libre / cache / swap
  - Disques : débit R/W, IOPS, occupation par point de montage
  - Réseau : up/down par interface
  - Processus : liste, CPU %, RAM, tri, recherche, kill, renice (sans élévation : ses propres processus seulement)
- Fenêtre chaude en mémoire (10 min) — **pas encore de SQLite**.
- `get_capabilities` + dégradation gracieuse (pas de température → pas de courbe température).

### Frontend
- Shell : sidebar + pages Dashboard, Processus, Paramètres.
- Dashboard : grille de widgets **repositionnables et redimensionnables**, disposition sauvegardée (fichier de config).
- Widgets : CPU (global + cœurs), fréquence/température, mémoire, débit disque, débit réseau, mini-liste top processus.
- Graphes uPlot fluides sur la fenêtre 10 min.
- Dark mode (le light peut attendre), design tokens en place.
- Intervalle de rafraîchissement configurable par collecteur.
- Page Processus complète : tri dynamique, recherche, kill avec confirmation, renice.

### Qualité
- CI 3 OS (le build Windows/macOS doit passer même si les collecteurs y sont partiels — `sysinfo` donne déjà CPU/RAM/processus « gratuits »).
- Budget perf vérifié : < 1 % CPU au repos à 1 Hz, mesuré par un bench en CI.

## Explicitement exclus du MVP (et pourquoi)

| Reporté | Raison |
|---------|--------|
| GPU | multi-vendeur = gros chantier ; arrive en Phase 2, NVIDIA d'abord |
| SMART, Docker | collecteurs additionnels — l'architecture les rend triviaux à ajouter ensuite |
| Historique SQLite, exports, snapshots | la fenêtre chaude suffit pour valider l'UX temps réel |
| Alertes | dépend de l'historique |
| Remote SSH, multi-machines | dépend d'une base locale stable |
| Plugins | dernière brique, l'API doit d'abord se stabiliser |
| Windows/macOS complets | le MVP cible Linux (machine de dev), la portabilité est structurelle dès le départ |

## Critères d'acceptation du MVP

1. `cargo tauri dev` lance l'app en < 1 s ; le dashboard affiche CPU/RAM/disque/réseau en temps réel.
2. Déplacer/redimensionner un widget, relancer l'app → la disposition est conservée.
3. Chercher un processus, le tuer depuis l'UI → il disparaît de la liste.
4. Passer le rafraîchissement CPU de 1 s à 250 ms → les graphes suivent sans jank.
5. Sur une machine sans capteur de température → le widget correspondant est absent, aucun message d'erreur.
6. L'app au repos consomme < 1 % CPU et < 150 Mo RAM.
